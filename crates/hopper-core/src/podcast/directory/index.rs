use super::{Category, Listing, Page, Query};
use crate::{Node, model::now};
use anyhow::{Context, Result, ensure};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub struct Keys {
    key: String,
    secret: String,
}

impl Node {
    pub fn save_index_keys(&self, keys: Keys) -> Result<()> {
        ensure!(
            !cfg!(target_os = "ios"),
            "Set up Podcast Index on your desktop."
        );
        ensure!(
            [&keys.key, &keys.secret]
                .iter()
                .all(|s| (1..=512).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_graphic())),
            "Enter the API key and secret from Podcast Index."
        );
        let file = tempfile::NamedTempFile::new_in(&self.0.dir)?;
        serde_json::to_writer(file.as_file(), &keys)?;
        file.as_file().sync_all()?;
        file.persist(self.0.dir.join("podcast-index.json"))?;
        Ok(())
    }
    fn index_keys(&self) -> Result<Keys> {
        match std::fs::read(self.0.dir.join("podcast-index.json")) {
            Ok(raw) => Ok(serde_json::from_slice(&raw)
                .context("Re-enter the Podcast Index credentials on the desktop.")?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let key = std::env::var("PODCAST_INDEX_API_KEY").unwrap_or_default();
                let secret = std::env::var("PODCAST_INDEX_API_SECRET").unwrap_or_default();
                ensure!(
                    !key.is_empty() && !secret.is_empty(),
                    "Set up Podcast Index on {} in Podcasts → Browse → Podcast Index.",
                    self.0.device.name
                );
                Ok(Keys { key, secret })
            }
            Err(e) => Err(e.into()),
        }
    }
}

fn signature(keys: &Keys, date: &str) -> String {
    let mut h = ring::digest::Context::new(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY);
    for part in [keys.key.as_str(), keys.secret.as_str(), date] {
        h.update(part.as_bytes());
    }
    h.finish()
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

async fn get(
    client: &Client,
    keys: &Keys,
    endpoint: &str,
    params: &[(&str, &str)],
) -> Result<Value> {
    let mut url = Url::parse(&format!("https://api.podcastindex.org/api/1.0/{endpoint}"))?;
    url.query_pairs_mut().extend_pairs(params.iter().copied());
    let date = now().to_string();
    let mut resp = client
        .get(url)
        .header("X-Auth-Date", &date)
        .header("X-Auth-Key", &keys.key)
        .header("Authorization", signature(keys, &date))
        .send()
        .await
        .map_err(|_| anyhow::anyhow!("Could not reach Podcast Index."))?;
    ensure!(
        !matches!(resp.status().as_u16(), 401 | 403),
        "Podcast Index rejected the credentials. Check the API key, secret and desktop clock."
    );
    ensure!(
        resp.status().is_success(),
        "Podcast Index returned HTTP {}.",
        resp.status().as_u16()
    );
    let mut bytes = Vec::new();
    while let Some(chunk) = resp
        .chunk()
        .await
        .context("Podcast Index transfer interrupted.")?
    {
        ensure!(
            bytes.len() + chunk.len() <= 4 * 1024 * 1024,
            "Podcast Index response too large."
        );
        bytes.extend_from_slice(&chunk);
    }
    let doc: Value =
        serde_json::from_slice(&bytes).context("Podcast Index returned an unreadable response.")?;
    ensure!(
        doc["status"] != false && doc["status"] != "false",
        "Podcast Index could not complete this search."
    );
    Ok(doc)
}

fn categories(doc: &Value) -> Result<Vec<Category>> {
    let mut cats: Vec<_> = doc["feeds"]
        .as_array()
        .context("Podcast Index returned an unreadable category list.")?
        .iter()
        .filter_map(|row| {
            let name = row["name"].as_str()?;
            Some(Category {
                id: row["id"].as_u64()?.to_string(),
                name: name.chars().take(100).collect(),
            })
        })
        .collect();
    cats.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(cats)
}

fn listings(doc: &Value, filter: Option<&str>) -> Result<Vec<Listing>> {
    Ok(doc["feeds"]
        .as_array()
        .context("Podcast Index returned an unreadable list of shows.")?
        .iter()
        .filter(|s| filter.is_none_or(|id| s["categories"].get(id).is_some()))
        .filter_map(|s| {
            Listing::new(
                s["title"].as_str()?,
                s["author"].as_str().unwrap_or_default(),
                s["url"].as_str()?,
                s["artwork"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .or(s["image"].as_str()),
            )
        })
        .collect())
}

pub(super) async fn browse(node: &Node, query: &Query) -> Result<Page> {
    let keys = node.index_keys()?;
    let client = Client::builder()
        .timeout(Duration::from_secs(22))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("Hopper/0.1 podcast discovery")
        .build()?;
    let cats = node
        .0
        .index_categories
        .get_or_try_init(|| async {
            categories(&get(&client, &keys, "categories/list", &[]).await?)
        })
        .await?;
    let category = query.category.as_str();
    ensure!(
        category.is_empty() || cats.iter().any(|c| c.id == category),
        "Choose a Podcast Index category."
    );
    let term = query.term.trim();
    let doc = if term.is_empty() {
        let mut params = vec![("max", "40")];
        if !category.is_empty() {
            params.push(("cat", category));
        }
        get(&client, &keys, "podcasts/trending", &params).await?
    } else {
        get(
            &client,
            &keys,
            "search/byterm",
            &[
                ("q", term),
                ("max", if category.is_empty() { "40" } else { "200" }),
            ],
        )
        .await?
    };
    let filter = (!term.is_empty() && !category.is_empty()).then_some(category);
    Ok(Page::new(listings(&doc, filter)?.into_iter(), cats.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn signs_the_key_secret_and_timestamp_in_provider_order() {
        let keys = Keys {
            key: "key".into(),
            secret: "secret".into(),
        };
        assert_eq!(
            signature(&keys, "1700000000"),
            "abaf71c02050c31e4d4e6b08c1625173af0445ba"
        );
    }
    #[test]
    fn categories_filter_search_and_ignore_broken_art() {
        let cats =
            categories(&json!({"feeds":[{"id":65,"name":"News"},{"id":1,"name":"Arts"}]})).unwrap();
        assert_eq!(cats[0].id, "1");
        let shows=listings(&json!({"feeds":[
            {"title":"Players","url":"https://example.org/show","image":"file:///bad","categories":{"1":"Arts"}},
            {"title":"Scores","url":"https://example.org/sports","categories":{"65":"News"}}
        ]}),Some("1")).unwrap();
        assert_eq!(shows.len(), 1);
        assert_eq!(shows[0].title, "Players");
        assert!(shows[0].artwork.is_none());
    }
}
