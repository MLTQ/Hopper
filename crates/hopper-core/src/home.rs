use crate::{Node, model::*, podcast, wire};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    pub device_id: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Payload {
    Podcast { url: String },
    Media { id: String },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Delivery {
    pub id: String,
    pub owner: Device,
    pub label: String,
    pub payload: Payload,
    pub state: String,
    pub message: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub attempts: u32,
    pub next_attempt: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub message: String,
    pub show_id: Option<String>,
    pub media_id: Option<String>,
}
#[derive(Deserialize)]
pub struct AddPodcasts {
    pub owner_id: Option<String>,
    pub url: Option<String>,
    pub opml: Option<String>,
    pub batch_id: Option<String>,
}
#[derive(Serialize)]
pub struct Added {
    pub delivery_ids: Vec<String>,
    pub warnings: Vec<String>,
}

impl Node {
    pub fn home_device(&self) -> Result<Option<Device>> {
        let settings: Settings = self.0.store.get("settings", "home")?.unwrap_or_default();
        let Some(id) = settings.device_id else {
            return Ok((!cfg!(target_os = "ios")).then(|| self.0.device.clone()));
        };
        if id == self.0.device.id {
            return Ok(Some(self.0.device.clone()));
        }
        Ok(self.0.store.get::<Peer>("peer", &id)?.map(|p| p.device))
    }
    pub fn set_home(&self, settings: Settings) -> Result<()> {
        if let Some(id) = &settings.device_id {
            self.delivery_owner(Some(id))?;
        }
        self.0.store.put("settings", "home", &settings)
    }
    fn delivery_owner(&self, id: Option<&str>) -> Result<Device> {
        let id = id
            .map(str::to_owned)
            .or(self.home_device()?.map(|d| d.id))
            .context("choose a Home desktop in Devices first")?;
        if id == self.0.device.id {
            ensure!(!cfg!(target_os = "ios"), "choose a desktop as Home");
            return Ok(self.0.device.clone());
        }
        let peer = self
            .0
            .store
            .get::<Peer>("peer", &id)?
            .context("pair the Home desktop first")?;
        ensure!(
            self.0
                .store
                .get::<Snapshot>("snapshot", &id)?
                .is_some_and(|s| s.podcasts.desktop),
            "refresh this paired desktop before choosing it as Home"
        );
        Ok(peer.device)
    }
    pub fn deliveries(&self) -> Result<Vec<Delivery>> {
        let mut jobs = self.0.store.all::<Delivery>("delivery")?;
        jobs.sort_by_key(|j| std::cmp::Reverse(j.created_at));
        // Keep every outstanding item visible, plus recent completed imports.
        let mut complete = 0;
        jobs.retain(|j| {
            if j.state == "delivered" {
                complete += 1;
                complete <= 30
            } else {
                true
            }
        });
        Ok(jobs)
    }
    fn enqueue(
        &self,
        owner: Device,
        payload: Payload,
        label: String,
        batch: &str,
    ) -> Result<String> {
        let id = blake3::hash(
            format!(
                "{}|{}|{}",
                owner.id,
                batch,
                serde_json::to_string(&payload)?
            )
            .as_bytes(),
        )
        .to_hex()
        .to_string();
        if self.0.store.get::<Delivery>("delivery", &id)?.is_none() {
            let count = self
                .0
                .store
                .all::<Delivery>("delivery")?
                .iter()
                .filter(|j| matches!(j.state.as_str(), "waiting" | "sending"))
                .count();
            ensure!(
                count < 2000,
                "too many pending additions; let Home receive them first"
            );
            self.0.store.put(
                "delivery",
                &id,
                &Delivery {
                    id: id.clone(),
                    owner,
                    label,
                    payload,
                    state: "waiting".into(),
                    message: "Waiting to send to Home".into(),
                    created_at: now(),
                    updated_at: now(),
                    attempts: 0,
                    next_attempt: 0,
                },
            )?;
        }
        self.0.delivery_wake.notify_one();
        Ok(id)
    }
    pub fn add_podcasts(&self, input: AddPodcasts) -> Result<Added> {
        let owner = self.delivery_owner(input.owner_id.as_deref())?;
        ensure!(
            input.url.is_some() != input.opml.is_some(),
            "provide a podcast link or an OPML file"
        );
        let list = if let Some(xml) = input.opml {
            podcast::intake::opml(&xml)?
        } else {
            podcast::intake::FeedList {
                urls: vec![podcast::intake::normalize(input.url.as_deref().unwrap())?],
                errors: vec![],
            }
        };
        let batch = input.batch_id.unwrap_or_else(crate::random_secret);
        ensure!(batch.len() <= 128, "invalid import identifier");
        let mut ids = vec![];
        for url in list.urls {
            ids.push(self.enqueue(
                owner.clone(),
                Payload::Podcast { url },
                "Podcast import".into(),
                &batch,
            )?);
        }
        Ok(Added {
            delivery_ids: ids,
            warnings: list.errors,
        })
    }
    pub async fn send_imports_home(&self, ids: &[String]) -> Result<()> {
        let Some(owner) = self.home_device()? else {
            return Ok(());
        };
        if owner.id == self.0.device.id {
            return Ok(());
        }
        self.delivery_owner(Some(&owner.id))?;
        let batch = crate::random_secret();
        for id in ids {
            let local = self.stage_media(id).await?;
            self.enqueue(
                owner.clone(),
                Payload::Media { id: id.clone() },
                local.media.title,
                &batch,
            )?;
        }
        Ok(())
    }
    async fn stage_media(&self, id: &str) -> Result<LocalMedia> {
        let _guard = self.0.downloads.lock().await;
        let (import_root, mut local) = match self
            .0
            .store
            .get::<LocalMedia>("local", id)?
            .filter(crate::import::available)
        {
            Some(file) => (None, file),
            None => {
                let (name, file) = self
                    .0
                    .store
                    .imported_file(id)?
                    .context("imported media missing")?;
                (Some(name), file)
            }
        };
        ensure!(
            crate::import::available(&local),
            "source changed; import it again"
        );
        if local.managed {
            return Ok(local);
        }
        let temp = tempfile::NamedTempFile::new_in(self.0.dir.join("media"))?;
        tokio::fs::copy(&local.path, temp.path()).await?;
        let path = temp.path().to_owned();
        let filename = local.media.filename.clone();
        let folder = local.media.folder.clone();
        let checked = tokio::task::spawn_blocking(move || {
            crate::import::inspect(&path, &filename, &folder, true)
        })
        .await??;
        ensure!(
            checked.media.id == id,
            "source changed while preparing delivery; import it again"
        );
        let ext = std::path::Path::new(&local.media.filename)
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or("media");
        ensure!(
            ext.len() <= 12 && ext.bytes().all(|b| b.is_ascii_alphanumeric()),
            "invalid file extension"
        );
        temp.as_file().sync_all()?;
        let dest = self.0.dir.join("media").join(format!("{id}.{ext}"));
        crate::import::persist_managed(temp, &dest, id).await?;
        local.path = dest;
        local.managed = true;
        local.modified = crate::import::modified(&std::fs::metadata(&local.path)?);
        self.0.store.put_import(import_root.as_deref(), &local)?;
        Ok(local)
    }
    pub fn retry_delivery(&self, id: &str) -> Result<()> {
        self.0.store.update::<Delivery>("delivery", id, |old| {
            let mut j = old.context("addition not found")?;
            ensure!(j.state != "sending", "addition is being sent");
            j.state = "waiting".into();
            j.next_attempt = 0;
            j.message = "Waiting to send to Home".into();
            Ok(j)
        })?;
        self.0.delivery_wake.notify_one();
        Ok(())
    }
    pub(crate) fn receive_delivery<'a>(
        &'a self,
        source: &'a str,
        id: &'a str,
        payload: Payload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Receipt>> + Send + 'a>> {
        Box::pin(async move {
            ensure!(
                !cfg!(target_os = "ios"),
                "a desktop must receive Home additions"
            );
            ensure!(valid_id(id), "invalid delivery ID");
            ensure!(
                source == self.0.device.id || self.0.store.get::<Peer>("peer", source)?.is_some(),
                "sender is no longer paired"
            );
            let _guard = self.0.deliveries.lock().await;
            let key = format!("{source}:{id}");
            if let Some(receipt) = self.0.store.get::<Receipt>("received_delivery", &key)? {
                return Ok(receipt);
            }
            let receipt = match payload {
                Payload::Podcast { url } => {
                    let resolved = podcast::intake::resolve(&url).await?;
                    let r = self
                        .apply_podcast_command(podcast::Command::Subscribe {
                            url: resolved.feed_url,
                        })
                        .await?;
                    let show = r.show_id.context("podcast was not imported")?;
                    let episode = self.podcast_catalogue()?.episodes.into_iter().find(|e| {
                        e.show_id == show
                            && (resolved.episode_guid.as_ref().is_some_and(|g| g == &e.guid)
                                || resolved
                                    .episode_url
                                    .as_ref()
                                    .is_some_and(|u| u == &e.enclosure_url))
                    });
                    let mut message = "Subscribed on Home".to_owned();
                    let mut media_id = None;
                    if let Some(episode) = episode {
                        let r = self
                            .apply_podcast_command(podcast::Command::Download {
                                episode_id: episode.id,
                            })
                            .await?;
                        media_id = r.media_id;
                        message = "Subscribed and saved the shared episode on Home".into();
                    } else if resolved.episode_requested {
                        message="Show subscribed. The shared episode could not be matched; choose it from the show's episode list.".into();
                    }
                    Receipt {
                        message,
                        show_id: Some(show),
                        media_id,
                    }
                }
                Payload::Media { id } => {
                    ensure!(valid_id(&id), "invalid media ID");
                    self.refresh().await?;
                    ensure!(
                        self.0
                            .store
                            .get::<Snapshot>("snapshot", source)?
                            .is_some_and(|s| s.items.iter().any(|m| m.id == id)),
                        "sender has not shared this media in its Hopper library"
                    );
                    self.keep_local(&id).await?;
                    self.stage_media(&id).await?;
                    Receipt {
                        message: "Saved and verified on Home".into(),
                        show_id: None,
                        media_id: Some(id),
                    }
                }
            };
            self.0.store.put("received_delivery", &key, &receipt)?;
            Ok(receipt)
        })
    }
    pub(crate) async fn deliver(
        &self,
        job: &Delivery,
    ) -> Result<std::result::Result<Receipt, String>> {
        if job.owner.id == self.0.device.id {
            return Ok(self
                .receive_delivery(&self.0.device.id, &job.id, job.payload.clone())
                .await
                .map_err(|e| e.to_string()));
        }
        let Some(peer) = self.0.store.get::<Peer>("peer", &job.owner.id)? else {
            return Ok(Err(
                "Home is no longer paired; pair it again and retry".into()
            ));
        };
        let (_, mut recv) = self
            .request(
                peer.address,
                wire::Request::Deliver {
                    id: job.id.clone(),
                    payload: job.payload.clone(),
                },
            )
            .await?;
        match tokio::time::timeout(
            std::time::Duration::from_secs(3700),
            wire::read_json(&mut recv),
        )
        .await??
        {
            wire::Response::Delivered { receipt } => Ok(Ok(receipt)),
            wire::Response::Error { message } => Ok(Err(message)),
            _ => Ok(Err("Update Hopper on Home to receive additions".into())),
        }
    }
}
