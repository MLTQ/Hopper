use iroh::EndpointAddr;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Chapter {
    pub title: String,
    pub start: f64,
    pub end: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Media {
    pub id: String,
    pub title: String,
    pub creator: String,
    pub album: String,
    pub kind: String,
    pub filename: String,
    pub folder: String,
    pub mime: String,
    pub bytes: u64,
    pub duration: Option<f64>,
    pub chapters: Vec<Chapter>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalMedia {
    pub media: Media,
    pub path: PathBuf,
    pub modified: u64,
    pub managed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Peer {
    pub device: Device,
    pub address: EndpointAddr,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Progress {
    pub item_id: String,
    pub position: f64,
    pub duration: Option<f64>,
    pub completed: bool,
    pub revision: u64,
    pub device_id: String,
    pub updated_at: u64,
}

impl Progress {
    pub fn newer_than(&self, other: &Self) -> bool {
        (self.revision, &self.device_id) > (other.revision, &other.device_id)
    }
    pub fn validate(&self) -> bool {
        valid_id(&self.item_id)
            && self.position.is_finite()
            && self.position >= 0.0
            && self.duration.is_none_or(|v| v.is_finite() && v >= 0.0)
            && self.device_id.len() <= 128
            && self.revision < u64::MAX
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Queue {
    pub items: Vec<String>,
    pub current: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub device: Device,
    pub items: Vec<Media>,
    pub progress: Vec<Progress>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Source {
    pub device_id: String,
    pub name: String,
    pub local: bool,
    pub online: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct LibraryItem {
    #[serde(flatten)]
    pub media: Media,
    pub sources: Vec<Source>,
    pub progress: Option<Progress>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PeerStatus {
    #[serde(flatten)]
    pub device: Device,
    pub online: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Library {
    pub device: Device,
    pub items: Vec<LibraryItem>,
    pub peers: Vec<PeerStatus>,
    pub queue: Queue,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct ImportReport {
    pub added: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

pub fn valid_id(id: &str) -> bool {
    id.len() == 64
        && id
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
