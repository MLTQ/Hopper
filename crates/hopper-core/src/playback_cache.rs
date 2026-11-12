use crate::{Node, import, model::*};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{Arc, Mutex, Weak},
};

const KIND: &str = "playback_copy";
const LIFETIME: u64 = 30 * 86400;
type Transfer = tokio::sync::Mutex<()>;

#[derive(Default)]
pub(crate) struct CacheLocks {
    transfers: Mutex<HashMap<String, Weak<Transfer>>>,
    publication: Mutex<()>,
}

#[derive(Serialize, Deserialize)]
struct CopyRecord {
    media: Media,
    filename: String,
    modified: u64,
    saved_at: u64,
    prior_progress: Option<Progress>,
}

impl Node {
    fn cache_directory(&self) -> PathBuf {
        self.0.dir.join("playback-cache")
    }

    pub(crate) fn is_playback_copy(&self, local: &LocalMedia) -> bool {
        local.path.parent() == Some(self.cache_directory().as_path())
    }

    fn cached_file(&self, copy: &CopyRecord) -> Result<LocalMedia> {
        // Reconstruct sandbox-relative paths every launch, including after an iOS app update.
        ensure!(
            valid_id(&copy.media.id)
                && copy
                    .filename
                    .strip_prefix(&copy.media.id)
                    .and_then(|s| s.strip_prefix('.'))
                    .is_some_and(|ext| !ext.is_empty()
                        && ext.len() <= 12
                        && ext.bytes().all(|c| c.is_ascii_alphanumeric())),
            "Invalid cached filename"
        );
        Ok(LocalMedia {
            media: copy.media.clone(),
            path: self.cache_directory().join(&copy.filename),
            managed: true,
            modified: copy.modified,
        })
    }

    pub(crate) fn playback_copy(&self, id: &str) -> Result<Option<LocalMedia>> {
        self.0
            .store
            .get::<CopyRecord>(KIND, id)?
            .map(|r| self.cached_file(&r))
            .transpose()
            .map(|f| f.filter(import::available))
    }

    pub(crate) fn playback_copies(&self) -> Result<Vec<LocalMedia>> {
        self.0
            .store
            .all::<CopyRecord>(KIND)?
            .iter()
            .map(|r| self.cached_file(r))
            .filter(|f| f.as_ref().map(import::available).unwrap_or(true))
            .collect()
    }

    pub(crate) async fn download_for_playback(&self, id: &str) -> Result<PathBuf> {
        ensure!(valid_id(id), "Invalid media ID");
        let transfer = {
            let mut locks = self.0.playback_cache.transfers.lock().unwrap();
            locks.retain(|_, lock| lock.strong_count() > 0);
            if let Some(lock) = locks.get(id).and_then(Weak::upgrade) {
                lock
            } else {
                let lock = Arc::new(Transfer::new(()));
                locks.insert(id.to_owned(), Arc::downgrade(&lock));
                lock
            }
        };
        let _transfer = transfer.lock().await;
        self.prune_playback_copy(id, now())?;
        if let Some(file) = self.local_file(id)? {
            return Ok(file.path);
        }
        let item = self
            .library()?
            .items
            .into_iter()
            .find(|i| i.media.id == id)
            .context("Item is no longer in your library")?;
        let directory = self.cache_directory();
        crate::private_dir(&directory)?;
        let file = self.download_verified(&item.media, &directory).await?;
        let _guard = self.0.playback_cache.publication.lock().unwrap();
        let record = CopyRecord {
            media: item.media,
            filename: file
                .path
                .file_name()
                .and_then(|f| f.to_str())
                .context("Invalid cached filename")?
                .to_owned(),
            modified: file.modified,
            saved_at: now(),
            prior_progress: item.progress,
        };
        if self.finished_since(&record)? {
            std::fs::remove_file(&file.path)?;
            anyhow::bail!("Playback already finished");
        }
        self.0.store.put(KIND, id, &record)?;
        Ok(file.path)
    }

    fn finished_since(&self, copy: &CopyRecord) -> Result<bool> {
        Ok(self
            .0
            .store
            .get::<Progress>("progress", &copy.media.id)?
            .is_some_and(|p| {
                p.completed
                    && copy
                        .prior_progress
                        .as_ref()
                        .is_none_or(|prior| p.newer_than(prior))
            }))
    }

    pub(crate) fn prune_playback_copy(&self, id: &str, at: u64) -> Result<()> {
        let _guard = self.0.playback_cache.publication.lock().unwrap();
        let Some(copy) = self.0.store.get::<CopyRecord>(KIND, id)? else {
            return Ok(());
        };
        if at.saturating_sub(copy.saved_at) >= LIFETIME || self.finished_since(&copy)? {
            let path = self.cached_file(&copy)?.path;
            match std::fs::remove_file(path) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e.into()),
                _ => self.0.store.remove(KIND, id)?,
            }
        }
        Ok(())
    }

    /// Only run before serving requests. Incomplete transfers and interrupted publications have no index.
    pub(crate) fn recover_playback_cache(&self) -> Result<()> {
        self.prune_playback_cache(now())?;
        let directory = self.cache_directory();
        if !directory.exists() {
            return Ok(());
        }
        let mut live = HashSet::new();
        for record in self.0.store.all::<CopyRecord>(KIND)? {
            let file = self.cached_file(&record)?;
            if import::available(&file) {
                live.insert(file.path);
            } else {
                self.0.store.remove(KIND, &record.media.id)?;
            }
        }
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            if entry.file_type()?.is_file() && !live.contains(&entry.path()) {
                std::fs::remove_file(entry.path())?;
            }
        }
        Ok(())
    }

    pub(crate) fn prune_playback_cache(&self, at: u64) -> Result<()> {
        for copy in self.0.store.all::<CopyRecord>(KIND)? {
            self.prune_playback_copy(&copy.media.id, at)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn retained_playback_survives_restart_releases_on_completion_and_expires() {
        let tmp = tempfile::tempdir().unwrap();
        let config = |name: &str| Config {
            data_dir: tmp.path().join(name),
            name: name.to_owned(),
            local_only: true,
        };
        let (desktop, _) = Node::start(config("archive")).await.unwrap();
        let (phone, _) = Node::start(config("phone")).await.unwrap();
        let original = tmp.path().join("Episode.mp3");
        let bytes = vec![17u8; 96_000];
        std::fs::write(&original, &bytes).unwrap();
        let id = desktop
            .import(vec![original.clone()])
            .await
            .unwrap()
            .item_ids
            .remove(0);
        phone.pair(&desktop.invite().await.unwrap()).await.unwrap();
        let (one, two) = tokio::join!(
            phone.download_for_playback(&id),
            phone.download_for_playback(&id)
        );
        let path = one.unwrap();
        assert_eq!(path, two.unwrap());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert!(phone.library().unwrap().items[0].automatic_download);
        let interrupted = phone.cache_directory().join(".tmp-interrupted");
        std::fs::write(&interrupted, b"partial").unwrap();
        phone.close().await;
        drop(phone);
        let (phone, _) = Node::start(config("phone")).await.unwrap();
        assert!(!interrupted.exists());
        assert_eq!(phone.local_file(&id).unwrap().unwrap().path, path);
        phone.set_progress(&id, 4.0, Some(10.0), false).unwrap();
        assert!(path.exists());
        phone.set_progress(&id, 10.0, Some(10.0), true).unwrap();
        assert!(!path.exists());
        // Replaying a previously finished episode earns a new retention period.
        phone.download_for_playback(&id).await.unwrap();
        let saved_at = phone
            .0
            .store
            .get::<CopyRecord>(KIND, &id)
            .unwrap()
            .unwrap()
            .saved_at;
        phone.prune_playback_cache(saved_at + LIFETIME - 1).unwrap();
        assert!(path.exists());
        phone.keep_local(&id).await.unwrap();
        phone.prune_playback_cache(saved_at + LIFETIME).unwrap();
        assert!(!path.exists());
        let pinned = phone.local_file(&id).unwrap().unwrap();
        assert!(!phone.is_playback_copy(&pinned));
        assert_eq!(std::fs::read(pinned.path).unwrap(), bytes);
        assert_eq!(std::fs::read(original).unwrap(), bytes);
        phone.close().await;
        desktop.close().await;
    }
}
