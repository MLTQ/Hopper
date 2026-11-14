use crate::{
    Node,
    model::{LocalMedia, now},
};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Download {
    pub episode_id: String,
    pub show_id: String,
    pub state: String,
    pub message: String,
    pub attempts: u32,
    pub next_attempt: u64,
    pub created_at: u64,
}

impl Node {
    pub(super) fn queue_episode_download(&self, id: &str) -> Result<()> {
        ensure!(
            !cfg!(target_os = "ios"),
            "Download podcast episodes on their desktop."
        );
        let episode = self.episode(id)?;
        ensure!(
            self.show(&episode.show_id)?.followed,
            "Subscribe to this show before downloading."
        );
        let available = match &episode.original_id {
            Some(id) => self
                .0
                .store
                .get::<LocalMedia>("local", id)?
                .is_some_and(|l| crate::import::available(&l)),
            None => false,
        };
        self.0
            .store
            .update::<Download>("episode_download", id, |old| {
                if let Some(old) = old
                    && matches!(old.state.as_str(), "queued" | "downloading")
                {
                    return Ok(old);
                }
                Ok(Download {
                    episode_id: episode.id,
                    show_id: episode.show_id,
                    state: if available { "ready" } else { "queued" }.into(),
                    message: if available {
                        "Downloaded"
                    } else {
                        "Waiting to download"
                    }
                    .into(),
                    attempts: 0,
                    next_attempt: 0,
                    created_at: now(),
                })
            })?;
        self.0.episode_download_wake.notify_one();
        Ok(())
    }

    pub(crate) fn start_episode_downloads(&self) {
        if cfg!(target_os = "ios") {
            return;
        }
        let weak = std::sync::Arc::downgrade(&self.0);
        tokio::spawn(async move {
            if let Some(inner) = weak.upgrade() {
                for mut job in inner
                    .store
                    .all::<Download>("episode_download")
                    .unwrap_or_default()
                {
                    if job.state == "downloading" {
                        job.state = "queued".into();
                        job.message = "Resuming download".into();
                        job.next_attempt = 0;
                        let _ = inner.store.put("episode_download", &job.episode_id, &job);
                    }
                }
                inner.episode_download_wake.notify_one();
            }
            loop {
                let Some(inner) = weak.upgrade() else { break };
                tokio::select! {
                    _=inner.shutdown.cancelled()=>break,
                    _=inner.episode_download_wake.notified()=>{},
                    _=tokio::time::sleep(Duration::from_secs(5))=>{}
                }
                let node = Node(inner);
                let mut jobs = node
                    .0
                    .store
                    .all::<Download>("episode_download")
                    .unwrap_or_default();
                jobs.sort_by_key(|j| j.created_at);
                let Some(mut job) = jobs
                    .into_iter()
                    .find(|j| j.state == "queued" && j.next_attempt <= now())
                else {
                    continue;
                };
                if !node.show(&job.show_id).is_ok_and(|s| s.followed) {
                    let _ = node.0.store.remove("episode_download", &job.episode_id);
                    continue;
                }
                job.state = "downloading".into();
                job.message = "Downloading episode".into();
                job.attempts = job.attempts.saturating_add(1);
                if node
                    .0
                    .store
                    .put("episode_download", &job.episode_id, &job)
                    .is_err()
                {
                    continue;
                }
                let result = tokio::select! {
                    _=node.0.shutdown.cancelled()=>break,
                    r=node.download_episode(&job.episode_id)=>r
                };
                match result {
                    Ok(_) => {
                        job.state = "ready".into();
                        job.message = "Downloaded".into();
                    }
                    Err(error) => {
                        job.state = if job.attempts < 3 { "queued" } else { "failed" }.into();
                        job.next_attempt = now() + 30 * u64::from(job.attempts);
                        job.message = if job.attempts < 3 {
                            format!("Download interrupted. Retrying shortly. {error}")
                        } else {
                            format!("Download failed. {error}")
                        };
                    }
                }
                let _ = node.0.store.put("episode_download", &job.episode_id, &job);
                node.0.episode_download_wake.notify_one();
            }
        });
    }
}
