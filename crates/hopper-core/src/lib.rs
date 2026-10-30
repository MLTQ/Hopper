mod ffi;
mod http;
pub mod import;
mod media;
pub mod model;
mod peer;
mod store;
mod wire;

use anyhow::{Context, Result};
use fs2::FileExt;
use iroh::{Endpoint, SecretKey, endpoint::presets};
use model::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fs::{File, OpenOptions},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use store::Store;

#[derive(Clone)]
pub struct Config {
    pub data_dir: PathBuf,
    pub name: String,
    pub local_only: bool,
}

impl Config {
    pub fn desktop(name: String) -> Result<Self> {
        let dir = directories::ProjectDirs::from("app", "Hopper", "Hopper")
            .context("could not locate application support directory")?;
        Ok(Self {
            data_dir: dir.data_local_dir().into(),
            name,
            local_only: false,
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Hello {
    pub port: u16,
    pub token: String,
    pub device: Device,
}

impl Hello {
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}/#{}", self.port, self.token)
    }
}

#[derive(Clone)]
pub struct Node(Arc<Inner>);

struct Inner {
    store: Store,
    endpoint: Endpoint,
    device: Device,
    dir: PathBuf,
    token: String,
    invites: Mutex<HashMap<String, Instant>>,
    grants: Mutex<HashMap<String, (String, Instant)>>,
    statuses: Mutex<HashMap<String, Option<String>>>,
    connections: tokio::sync::Mutex<HashMap<iroh::EndpointId, iroh::endpoint::Connection>>,
    refreshing: tokio::sync::Mutex<()>,
    downloads: tokio::sync::Mutex<()>,
    shutdown: tokio_util::sync::CancellationToken,
    _lock: File,
}

pub fn random_secret() -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(rand::random::<[u8; 32]>())
}

pub fn private_dir(path: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

impl Node {
    pub async fn start(config: Config) -> Result<(Self, Hello)> {
        anyhow::ensure!(
            !config.name.trim().is_empty() && config.name.len() <= 100,
            "device name must be 1–100 characters"
        );
        private_dir(&config.data_dir)?;
        private_dir(&config.data_dir.join("media"))?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(config.data_dir.join("node.lock"))?;
        lock.try_lock_exclusive()
            .context("this Hopper library is already open in another process")?;
        let key_path = config.data_dir.join("identity.key");
        let key = if key_path.exists() {
            let bytes = std::fs::read(&key_path)?;
            SecretKey::from_bytes(
                &bytes
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("invalid identity key; refusing to replace it"))?,
            )
        } else {
            use std::io::Write;
            let key = SecretKey::generate();
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&key_path)?;
            file.write_all(&key.to_bytes())?;
            file.sync_all()?;
            key
        };
        let builder = if config.local_only {
            Endpoint::builder(presets::Minimal)
        } else {
            Endpoint::builder(presets::N0)
        };
        let endpoint = builder
            .secret_key(key)
            .alpns(vec![wire::ALPN.to_vec()])
            .bind()
            .await?;
        let device = Device {
            id: endpoint.id().to_string(),
            name: config.name,
        };
        let node = Self(Arc::new(Inner {
            store: Store::open(&config.data_dir.join("library.sqlite"))?,
            endpoint,
            device: device.clone(),
            dir: config.data_dir,
            token: random_secret(),
            invites: Mutex::new(HashMap::new()),
            grants: Mutex::new(HashMap::new()),
            statuses: Mutex::new(HashMap::new()),
            connections: Default::default(),
            refreshing: Default::default(),
            downloads: Default::default(),
            shutdown: Default::default(),
            _lock: lock,
        }));
        node.start_accepting();
        let port = http::serve(node.clone()).await?;
        let hello = Hello {
            port,
            token: node.0.token.clone(),
            device,
        };
        let weak = Arc::downgrade(&node.0);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(15)).await;
                let Some(inner) = weak.upgrade() else { break };
                if inner.endpoint.is_closed() {
                    break;
                }
                let _ = Node(inner).refresh().await;
            }
        });
        Ok((node, hello))
    }

    pub async fn close(&self) {
        self.0.shutdown.cancel();
        self.0.endpoint.close().await;
    }

    pub fn snapshot(&self) -> Result<Snapshot> {
        let items = self
            .0
            .store
            .all::<LocalMedia>("local")?
            .into_iter()
            .filter(import::available)
            .map(|l| l.media)
            .collect();
        Ok(Snapshot {
            device: self.0.device.clone(),
            items,
            progress: self.0.store.all("progress")?,
        })
    }

    pub fn library(&self) -> Result<Library> {
        let statuses = self.0.statuses.lock().unwrap().clone();
        let mut items = BTreeMap::<String, LibraryItem>::new();
        for local in self.0.store.all::<LocalMedia>("local")? {
            let online = import::available(&local);
            let source = Source {
                device_id: self.0.device.id.clone(),
                name: self.0.device.name.clone(),
                local: true,
                online,
            };
            items.insert(
                local.media.id.clone(),
                LibraryItem {
                    media: local.media,
                    sources: vec![source],
                    progress: None,
                },
            );
        }
        let peers: Vec<Peer> = self.0.store.all("peer")?;
        for peer in &peers {
            if let Some(snapshot) = self.0.store.get::<Snapshot>("snapshot", &peer.device.id)? {
                for media in snapshot.items {
                    let source = Source {
                        device_id: peer.device.id.clone(),
                        name: peer.device.name.clone(),
                        local: false,
                        online: statuses.get(&peer.device.id).is_some_and(Option::is_none),
                    };
                    items
                        .entry(media.id.clone())
                        .or_insert_with(|| LibraryItem {
                            media,
                            sources: vec![],
                            progress: None,
                        })
                        .sources
                        .push(source);
                }
            }
        }
        for p in self.0.store.all::<Progress>("progress")? {
            if let Some(item) = items.get_mut(&p.item_id) {
                item.progress = Some(p);
            }
        }
        let mut items: Vec<_> = items.into_values().collect();
        items.sort_by(|a, b| {
            a.media.folder.cmp(&b.media.folder).then(
                a.media
                    .title
                    .to_lowercase()
                    .cmp(&b.media.title.to_lowercase()),
            )
        });
        Ok(Library {
            device: self.0.device.clone(),
            items,
            queue: self.0.store.get("queue", "current")?.unwrap_or_default(),
            peers: peers
                .into_iter()
                .map(|p| PeerStatus {
                    online: statuses.get(&p.device.id).is_some_and(Option::is_none),
                    error: statuses.get(&p.device.id).cloned().flatten(),
                    device: p.device,
                })
                .collect(),
        })
    }

    pub async fn import(&self, paths: Vec<PathBuf>) -> Result<ImportReport> {
        let node = self.clone();
        tokio::task::spawn_blocking(move || import::import_paths(&node.0.store, &paths))
            .await
            .map_err(Into::into)
    }

    pub fn set_queue(&self, queue: Queue) -> Result<()> {
        anyhow::ensure!(
            queue.items.len() <= 10000 && queue.items.iter().all(|id| valid_id(id)),
            "invalid queue"
        );
        anyhow::ensure!(
            queue
                .current
                .as_ref()
                .is_none_or(|id| queue.items.contains(id)),
            "current item must be in queue"
        );
        self.0.store.put("queue", "current", &queue)
    }

    pub fn set_progress(
        &self,
        id: &str,
        position: f64,
        duration: Option<f64>,
        completed: bool,
    ) -> Result<Progress> {
        self.0
            .store
            .progress(id, &self.0.device.id, position, duration, completed)
    }
}
