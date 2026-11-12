use super::{feed, fetch, types::*};
use crate::{Node, model::*, wire};
use anyhow::{Context, Result, bail, ensure};

impl Node {
    pub fn podcast_catalogue(&self) -> Result<Catalogue> {
        let shows: Vec<Show> = self.0.store.all("podcast_show")?;
        let ids: std::collections::HashSet<_> = shows
            .iter()
            .filter(|s| s.followed)
            .map(|s| s.id.clone())
            .collect();
        let mut jobs: Vec<Job> = self.0.store.all("podcast_job")?;
        jobs.sort_by_key(|j| std::cmp::Reverse(j.created_at));
        jobs.retain(|j| ids.contains(&j.show_id));
        jobs.truncate(200);
        Ok(Catalogue {
            shows: shows.into_iter().filter(|s| s.followed).collect(),
            episodes: self
                .0
                .store
                .all::<Episode>("podcast_episode")?
                .into_iter()
                .filter(|e| ids.contains(&e.show_id))
                .collect(),
            jobs,
            agent_available: self.agent_available(),
            desktop: !cfg!(target_os = "ios"),
            hosts: vec![],
        })
    }
    pub async fn podcast_command(&self, owner: &str, command: Command) -> Result<CommandResult> {
        if owner == self.0.device.id {
            return self.apply_podcast_command(command).await;
        }
        let peer = self
            .0
            .store
            .get::<Peer>("peer", owner)?
            .context("pair this podcast's desktop first")?;
        let (_, mut recv) = self
            .request(peer.address, wire::Request::Podcast { command })
            .await?;
        let result = match tokio::time::timeout(
            std::time::Duration::from_secs(3700),
            wire::read_json(&mut recv),
        )
        .await??
        {
            wire::Response::Podcast { result } => result,
            wire::Response::Error { message } => bail!("{message}"),
            _ => bail!("this device does not support podcast commands"),
        };
        self.refresh().await?;
        Ok(result)
    }
    pub(crate) async fn apply_podcast_command(&self, command: Command) -> Result<CommandResult> {
        let _guard = self.0.podcast_commands.lock().await;
        match command {
            Command::Subscribe { url } => {
                let url = fetch::url(&url)?.to_string();
                let id = blake3::hash(format!("{}\0{url}", self.0.device.id).as_bytes())
                    .to_hex()
                    .to_string();
                let mut show = self
                    .0
                    .store
                    .get::<Show>("podcast_show", &id)?
                    .unwrap_or(Show {
                        id: id.clone(),
                        owner: self.0.device.clone(),
                        feed_url: url,
                        title: String::new(),
                        author: String::new(),
                        description: String::new(),
                        artwork_url: None,
                        newest_first: false,
                        cut_options: CutOptions::default(),
                        followed: true,
                        updated_at: now(),
                        last_checked: 0,
                        error: None,
                    });
                show.followed = true;
                self.refresh_show(show).await?;
                Ok(CommandResult {
                    show_id: Some(id),
                    ..Default::default()
                })
            }
            Command::Refresh { show_id } => {
                self.refresh_show(self.show(&show_id)?).await?;
                Ok(CommandResult::default())
            }
            Command::Settings {
                show_id,
                newest_first,
                cut_options,
            } => {
                cut_options.validate()?;
                let mut show = self.show(&show_id)?;
                show.newest_first = newest_first;
                show.cut_options = cut_options;
                show.updated_at = now();
                self.0.store.put("podcast_show", &show.id, &show)?;
                Ok(CommandResult::default())
            }
            Command::Unsubscribe { show_id } => {
                let mut show = self.show(&show_id)?;
                show.followed = false;
                show.updated_at = now();
                self.0.store.put("podcast_show", &show.id, &show)?;
                for mut job in self.0.store.all::<Job>("podcast_job")? {
                    if job.show_id == show_id && job.active() {
                        job.state = "cancelled".into();
                        job.message = "Show unsubscribed; original files retained".into();
                        self.0.store.put("podcast_job", &job.id, &job)?
                    }
                }
                Ok(CommandResult::default())
            }
            Command::Download { episode_id } => {
                let episode = self.download_episode(&episode_id).await?;
                Ok(CommandResult {
                    media_id: episode.cleaned_id.or(episode.original_id),
                    ..Default::default()
                })
            }
            Command::CutEpisode {
                episode_id,
                options,
            } => {
                ensure!(
                    !cfg!(target_os = "ios"),
                    "Cut ads runs on the show's desktop."
                );
                options.validate()?;
                ensure!(
                    self.show(&self.episode(&episode_id)?.show_id)?.followed,
                    "Subscribe to this show before cutting an episode."
                );
                let id = self.queue_cut(&episode_id, &options)?;
                self.0.podcast_wake.notify_one();
                Ok(CommandResult {
                    job_ids: vec![id],
                    ..Default::default()
                })
            }
            Command::Cut {
                show_id,
                episode_ids,
                options,
            } => {
                ensure!(
                    !cfg!(target_os = "ios"),
                    "Cut ads runs on a desktop. Subscribe to this show on a paired desktop."
                );
                options.validate()?;
                ensure!(
                    !episode_ids.is_empty() && episode_ids.len() <= 500,
                    "choose between 1 and 500 episodes"
                );
                let mut show = self.show(&show_id)?;
                // Validate the entire request before creating any jobs.
                for id in &episode_ids {
                    ensure!(
                        self.episode(id)?.show_id == show_id,
                        "episode does not belong to this show"
                    )
                }
                for job in self.0.store.all::<Job>("podcast_job")? {
                    ensure!(
                        !job.active()
                            || !episode_ids.contains(&job.episode_id)
                            || job.options == options,
                        "an episode is already processing with different options; cancel it first"
                    );
                }
                show.cut_options = options.clone();
                show.updated_at = now();
                self.0.store.put("podcast_show", &show.id, &show)?;
                let mut job_ids = vec![];
                for id in episode_ids {
                    job_ids.push(self.queue_cut(&id, &options)?)
                }
                self.0.podcast_wake.notify_one();
                Ok(CommandResult {
                    job_ids,
                    ..Default::default()
                })
            }
            Command::Retry { job_id } => {
                let mut job = self
                    .0
                    .store
                    .get::<Job>("podcast_job", &job_id)?
                    .context("job not found")?;
                ensure!(
                    !job.active() || job.state == "waiting_for_agent",
                    "job is already running"
                );
                ensure!(
                    self.show(&job.show_id)?.followed,
                    "subscribe to this show before retrying"
                );
                job.state = "queued".into();
                job.message = "Queued".into();
                job.updated_at = now();
                self.0.store.put("podcast_job", &job.id, &job)?;
                self.0.podcast_wake.notify_one();
                Ok(CommandResult {
                    job_ids: vec![job_id],
                    ..Default::default()
                })
            }
            Command::Cancel { job_id } => {
                let mut job = self
                    .0
                    .store
                    .get::<Job>("podcast_job", &job_id)?
                    .context("job not found")?;
                ensure!(job.active(), "job is no longer running");
                job.state = "cancelled".into();
                job.message = "Cancelled; original retained".into();
                job.updated_at = now();
                self.0.store.put("podcast_job", &job.id, &job)?;
                Ok(CommandResult::default())
            }
        }
    }
    pub(super) fn show(&self, id: &str) -> Result<Show> {
        self.0
            .store
            .get("podcast_show", id)?
            .context("podcast not found on this device")
    }
    pub(super) fn episode(&self, id: &str) -> Result<Episode> {
        self.0
            .store
            .get("podcast_episode", id)?
            .context("episode not found on this device")
    }
    pub(super) async fn refresh_show(&self, mut show: Show) -> Result<()> {
        let (base, bytes) = fetch::document(&show.feed_url).await?;
        let xml = String::from_utf8(bytes).context("podcast feed must be UTF-8")?;
        let parsed = feed::parse(&xml, &base, &show.id)?;
        self.0.store.put("podcast_feed_xml", &show.id, &xml)?;
        let _guard = self.0.podcast_downloads.lock().await;
        let mut new_ids = vec![];
        for mut episode in parsed.episodes {
            if let Some(old) = self
                .0
                .store
                .get::<Episode>("podcast_episode", &episode.id)?
            {
                // Downloaded media is pinned to its exact bytes even when dynamic ad URLs change.
                episode.original_id = old.original_id;
                episode.cleaned_id = old.cleaned_id;
                if episode.original_id.is_some() {
                    episode.chapters = old.chapters;
                    episode.chapter_document = old.chapter_document;
                    episode.transcript = old.transcript;
                    episode.duration = old.duration;
                    episode.metadata_warnings = old.metadata_warnings;
                }
            } else {
                new_ids.push(episode.id.clone())
            }
            self.0.store.put("podcast_episode", &episode.id, &episode)?;
        }
        let first_refresh = show.last_checked == 0;
        show.title = parsed.title;
        show.author = parsed.author;
        show.description = parsed.description;
        show.artwork_url = parsed.artwork;
        show.last_checked = now();
        show.updated_at = now();
        show.error = None;
        self.0.store.put("podcast_show", &show.id, &show)?;
        if show.cut_options.auto_process && !first_refresh && !cfg!(target_os = "ios") {
            for id in new_ids {
                self.queue_cut(&id, &show.cut_options)?;
            }
            self.0.podcast_wake.notify_one();
        }
        Ok(())
    }
    pub(super) fn queue_cut(&self, id: &str, options: &CutOptions) -> Result<String> {
        let episode = self.episode(id)?;
        for job in self.0.store.all::<Job>("podcast_job")? {
            if job.episode_id == id && job.active() {
                ensure!(
                    job.options == *options,
                    "this episode already has a job with different options; cancel it first"
                );
                return Ok(job.id);
            }
        }
        let id = crate::random_secret();
        let job = Job {
            id: id.clone(),
            episode_id: episode.id,
            show_id: episode.show_id,
            options: options.clone(),
            state: if self.agent_available() {
                "queued"
            } else {
                "waiting_for_agent"
            }
            .into(),
            message: if self.agent_available() {
                "Queued for your agent"
            } else {
                "Waiting for the ad-cutting agent on this desktop"
            }
            .into(),
            created_at: now(),
            updated_at: now(),
            source_hash: None,
            output_id: None,
            removed_seconds: 0.0,
            analysis: None,
            timeline: vec![],
            cleaned_chapter_document: None,
            cleaned_transcript: vec![],
            cleaned_soundbites: vec![],
        };
        self.0.store.put("podcast_job", &id, &job)?;
        Ok(id)
    }
    pub(super) async fn download_episode(&self, id: &str) -> Result<Episode> {
        let _guard = self.0.podcast_downloads.lock().await;
        let mut episode = self.episode(id)?;
        if let Some(id) = &episode.original_id
            && self
                .0
                .store
                .get::<LocalMedia>("local", id)?
                .is_some_and(|l| crate::import::available(&l))
        {
            return Ok(episode);
        }
        let show = self.show(&episode.show_id)?;
        let temp = tempfile::NamedTempFile::new_in(self.0.dir.join("media"))?;
        fetch::download(&episode.enclosure_url, temp.path()).await?;
        let ext = match episode.mime.as_str() {
            "audio/mp4" | "audio/x-m4a" => "m4a",
            "audio/ogg" => "ogg",
            "audio/flac" => "flac",
            "audio/wav" | "audio/x-wav" => "wav",
            "video/mp4" => "mp4",
            _ => "mp3",
        };
        let filename = format!("{}.{}", episode.id, ext);
        let path = temp.path().to_owned();
        let folder = show.title.clone();
        let mut local = tokio::task::spawn_blocking(move || {
            crate::import::inspect(&path, &filename, &folder, true)
        })
        .await??;
        if let Some(url) = &episode.chapters_url {
            match super::chapters::download(url).await {
                Ok((doc, chapters)) => {
                    episode.chapters = chapters;
                    episode.chapter_document = Some(doc);
                }
                Err(_) => episode
                    .metadata_warnings
                    .push("External chapters could not be loaded".into()),
            }
        }
        if episode.chapters.is_empty() {
            episode.chapters = local.media.chapters.clone()
        }
        episode.chapters.sort_by(|a, b| a.start.total_cmp(&b.start));
        if let Some(url) = &episode.transcript_url {
            match fetch::document(url)
                .await
                .and_then(|(_, b)| feed::transcript(&String::from_utf8(b)?))
            {
                Ok(cues) => episode.transcript = cues,
                Err(_) => episode.metadata_warnings.push(
                    "Transcript unavailable or not WebVTT/SRT; original reference retained".into(),
                ),
            }
        }
        local.media.title = episode.title.clone();
        local.media.creator = show.author;
        local.media.album = show.title;
        local.media.chapters = episode.chapters.clone();
        local.media.duration = local.media.duration.or(episode.duration);
        episode.duration = local.media.duration;
        episode.original_id = Some(local.media.id.clone());
        let dest = self
            .0
            .dir
            .join("media")
            .join(format!("{}.{}", local.media.id, ext));
        crate::import::persist_managed(temp, &dest, &local.media.id).await?;
        local.path = dest;
        local.modified = crate::import::modified(&std::fs::metadata(&local.path)?);
        self.0.store.put("local", &local.media.id, &local)?;
        self.0.store.put("podcast_episode", &episode.id, &episode)?;
        Ok(episode)
    }
}
