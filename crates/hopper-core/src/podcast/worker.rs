#[cfg(not(target_os = "ios"))]
use super::timeline;
use super::types::*;
use crate::{Node, model::*};
#[cfg(not(target_os = "ios"))]
use anyhow::{Context, Result, ensure};
use std::time::Duration;

impl Node {
    pub fn agent_available(&self) -> bool {
        !cfg!(target_os = "ios") && self.agent_config().is_ok_and(|c| c.configured())
    }
    pub(crate) fn start_podcasts(&self) {
        let weak = std::sync::Arc::downgrade(&self.0);
        tokio::spawn(async move {
            // A process interruption never makes a partial render ready. Saved analysis is reused.
            if let Some(inner) = weak.upgrade() {
                for mut job in inner.store.all::<Job>("podcast_job").unwrap_or_default() {
                    if matches!(
                        job.state.as_str(),
                        "downloading" | "analysing" | "rendering"
                    ) {
                        job.state = "queued".into();
                        job.message = "Resuming after restart".into();
                        let _ = inner.store.put("podcast_job", &job.id, &job);
                    }
                }
            }
            loop {
                let Some(inner) = weak.upgrade() else { break };
                tokio::select! {_=inner.shutdown.cancelled()=>break,_=inner.podcast_wake.notified()=>{},_=tokio::time::sleep(Duration::from_secs(5))=>{}}
                let node = Node(inner);
                #[cfg(not(target_os = "ios"))]
                {
                    let mut jobs = node.0.store.all::<Job>("podcast_job").unwrap_or_default();
                    jobs.sort_by_key(|j| j.created_at);
                    if let Some(job) = jobs.into_iter().find(|j| {
                        j.state == "queued"
                            || j.state == "waiting_for_agent" && node.agent_available()
                    }) {
                        if let Err(error) = node.run_cut(&job.id).await {
                            let _ = node.0.store.update::<Job>("podcast_job", &job.id, |saved| {
                                let mut saved = saved.context("job missing")?;
                                if saved.state != "cancelled" {
                                    saved.state = "failed".into();
                                    saved.message = error.to_string();
                                    saved.updated_at = now();
                                }
                                Ok(saved)
                            });
                        }
                        node.0.podcast_wake.notify_one();
                    }
                }
                for mut show in node.0.store.all::<Show>("podcast_show").unwrap_or_default() {
                    if show.followed && now().saturating_sub(show.last_checked) > 1800 {
                        let Ok(_guard) = node.0.podcast_commands.try_lock() else {
                            continue;
                        };
                        if let Err(error) = node.refresh_show(show.clone()).await {
                            show.last_checked = now();
                            show.error = Some(error.to_string());
                            let _ = node.0.store.put("podcast_show", &show.id, &show);
                        }
                    }
                }
            }
        });
    }
    #[cfg(not(target_os = "ios"))]
    fn save_job(&self, job: &mut Job, state: &str, message: &str) -> Result<()> {
        job.state = state.into();
        job.message = message.into();
        job.updated_at = now();
        self.0.store.update::<Job>("podcast_job", &job.id, |old| {
            ensure!(old.is_some_and(|j| j.state != "cancelled"), "job cancelled");
            Ok(job.clone())
        })?;
        Ok(())
    }
    #[cfg(not(target_os = "ios"))]
    async fn run_cut(&self, id: &str) -> Result<()> {
        let mut job = self
            .0
            .store
            .get::<Job>("podcast_job", id)?
            .context("job not found")?;
        let config = self.agent_config()?;
        if !config.configured() {
            self.save_job(
                &mut job,
                "waiting_for_agent",
                "Waiting for the ad-cutting agent on this desktop",
            )?;
            return Ok(());
        }
        self.save_job(
            &mut job,
            "downloading",
            "Downloading original episode and metadata",
        )?;
        let episode = self.download_episode(&job.episode_id).await?;
        let source = self
            .0
            .store
            .get::<LocalMedia>(
                "local",
                episode.original_id.as_ref().context("original missing")?,
            )?
            .context("original file missing")?;
        ensure!(
            source.media.kind != "video",
            "Cut ads currently supports audio episodes; original video retained"
        );
        let measured = super::render::probe_duration(&source.path).await?;
        job.source_hash = Some(source.media.id.clone());
        let work = self.0.dir.join("jobs").join(&job.id);
        crate::private_dir(&work)?;
        let output = work.join("analysis.json");
        if job
            .analysis
            .as_ref()
            .is_none_or(|a| a.source_hash != source.media.id)
        {
            self.save_job(&mut job, "analysing", "Agent is listening for ads")?;
            let analysis = super::analyzer::analyze(
                self,
                &config,
                &job,
                &source,
                measured,
                &episode.title,
                &work,
                &output,
            )
            .await?;
            job.timeline = timeline::plan(&analysis, &job.options, &source.media.id, measured)?;
            job.analysis = Some(analysis);
        } else {
            job.timeline = timeline::plan(
                job.analysis.as_ref().unwrap(),
                &job.options,
                &source.media.id,
                measured,
            )?;
        }
        let clean_duration = timeline::duration(&job.timeline);
        job.removed_seconds = measured - clean_duration;
        self.save_job(
            &mut job,
            "rendering",
            "Stitching audio and updating timestamps",
        )?;
        let mut cleaned = if job.removed_seconds < 0.001 {
            source.clone()
        } else {
            super::render::render(&source, &job.timeline, job.options.fade_ms, &work).await?
        };
        ensure!(
            crate::import::available(&source),
            "source file changed while the agent was processing it"
        );
        cleaned.media.chapters = timeline::chapters(&episode.chapters, &job.timeline, measured);
        cleaned.media.duration = Some(clean_duration);
        job.cleaned_chapter_document = episode
            .chapter_document
            .as_ref()
            .map(|d| timeline::chapter_document(d, &job.timeline, measured));
        job.cleaned_transcript = timeline::cues(&episode.transcript, &job.timeline);
        job.cleaned_soundbites = timeline::soundbites(&episode.soundbites, &job.timeline);
        if job.removed_seconds >= 0.001 {
            let dest = self
                .0
                .dir
                .join("media")
                .join(format!("{}.m4a", cleaned.media.id));
            if !dest.exists() {
                tokio::fs::copy(&cleaned.path, &dest).await?;
                tokio::fs::File::open(&dest).await?.sync_all().await?;
            }
            cleaned.path = dest;
            cleaned.modified = crate::import::modified(&std::fs::metadata(&cleaned.path)?);
        }
        // Cancellation can arrive during rendering. Publish only after checking it again.
        let _guard = self.0.podcast_commands.lock().await;
        ensure!(
            self.0
                .store
                .get::<Job>("podcast_job", id)?
                .is_some_and(|j| j.state != "cancelled"),
            "job cancelled"
        );
        self.0.store.put("local", &cleaned.media.id, &cleaned)?;
        let _download_guard = self.0.podcast_downloads.lock().await;
        let mut episode = self.episode(&episode.id)?;
        episode.cleaned_id = Some(cleaned.media.id.clone());
        self.0.store.put("podcast_episode", &episode.id, &episode)?;
        if cleaned.media.id != source.media.id
            && let Some(p) = self.0.store.get::<Progress>("progress", &source.media.id)?
            && self
                .0
                .store
                .get::<Progress>("progress", &cleaned.media.id)?
                .is_none()
        {
            self.set_progress(
                &cleaned.media.id,
                timeline::map_time(p.position, &job.timeline),
                Some(clean_duration),
                p.completed,
            )?;
        }
        job.output_id = Some(cleaned.media.id);
        let message = if job.removed_seconds < 0.001 {
            "No confident cuts found; original kept"
        } else {
            "Ready — original preserved and timestamps updated"
        };
        self.save_job(&mut job, "ready", message)?;
        std::fs::write(
            work.join("edit-manifest.json"),
            serde_json::to_vec_pretty(&job)?,
        )?;
        Ok(())
    }
}
