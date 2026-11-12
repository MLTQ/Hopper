mod apple;
mod index;

use super::fetch;
use crate::{Node, model::Peer, wire};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;

pub use index::Keys;

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Apple,
    PodcastIndex,
}

#[derive(Serialize, Deserialize)]
pub struct Query {
    pub provider: Provider,
    #[serde(default)]
    term: String,
    #[serde(default)]
    category: String,
}
impl Query {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.term.len() <= 240 && self.category.len() <= 100,
            "Search or category is too long"
        );
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Category {
    id: String,
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Listing {
    title: String,
    author: String,
    url: String,
    artwork: Option<String>,
}

impl Listing {
    fn new(title: &str, author: &str, url: &str, artwork: Option<&str>) -> Option<Self> {
        let title = title.trim();
        if title.is_empty() || url.len() > 8192 {
            return None;
        }
        Some(Self {
            title: title.chars().take(320).collect(),
            author: author.trim().chars().take(320).collect(),
            url: fetch::url(url).ok()?.to_string(),
            artwork: artwork
                .filter(|s| s.len() <= 8192)
                .and_then(|s| fetch::url(s).ok())
                .map(Into::into),
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct Page {
    shows: Vec<Listing>,
    categories: Vec<Category>,
}

impl Page {
    fn new(rows: impl Iterator<Item = Listing>, categories: Vec<Category>) -> Self {
        let mut seen = HashSet::new();
        let shows = rows
            .filter(|s| seen.insert(s.url.clone()))
            .take(40)
            .collect();
        Self { shows, categories }
    }
}

impl Node {
    pub async fn directory_page(&self, query: Query, owner: Option<&str>) -> Result<Page> {
        query.validate()?;
        if query.provider == Provider::Apple {
            return apple::browse(&query).await;
        }
        let home = self.home_device()?;
        let id = owner
            .or(home.as_ref().map(|d| d.id.as_str()))
            .context("Choose a Home desktop in Devices to use Podcast Index.")?;
        if id == self.0.device.id {
            return self.local_index_page(&query).await;
        }
        let peer = self
            .0
            .store
            .get::<Peer>("peer", id)?
            .context("Pair the desktop for Podcast Index first.")?;
        let (_, mut recv) = self.request(peer.address, wire::Request::Directory {query}).await
            .context("Could not reach the desktop for Podcast Index. Check that Hopper is running there.")?;
        match tokio::time::timeout(Duration::from_secs(55), wire::read_json(&mut recv))
            .await
            .context("The desktop took too long to reach Podcast Index.")?
            .context("Update Hopper on the desktop to browse Podcast Index.")?
        {
            wire::Response::Directory { page } => Ok(page),
            wire::Response::Error { message } => bail!("{message}"),
            _ => bail!("Update Hopper on the desktop to browse Podcast Index."),
        }
    }
    pub(crate) async fn local_index_page(&self, query: &Query) -> Result<Page> {
        ensure!(
            query.provider == Provider::PodcastIndex && !cfg!(target_os = "ios"),
            "Choose a desktop for Podcast Index."
        );
        query.validate()?;
        index::browse(self, query).await
    }
}
