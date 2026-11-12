use super::fetch;
use crate::model::Chapter;
use anyhow::{Context, Result, ensure};
use serde_json::Value;

pub(crate) async fn download(url: &str) -> Result<(Value, Vec<Chapter>)> {
    let result = fetch::document(url).await;
    // Reflex also serves its own item timeline when the publisher's upstream chapter host fails.
    let (base, bytes) = match (result, reflex_item_url(url)) {
        (Err(_), Some(item)) => fetch::document(&item).await?,
        (result, _) => result?,
    };
    let mut document: Value = serde_json::from_slice(&bytes)?;
    let base = fetch::url(&base)?;
    let entries = document["chapters"]
        .as_array_mut()
        .context("Invalid chapter document")?;
    let mut chapters = Vec::new();
    for entry in entries {
        let Some(start) = entry["startTime"]
            .as_f64()
            .filter(|t| t.is_finite() && *t >= 0.)
        else {
            continue;
        };
        let artwork_url = entry["img"]
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .and_then(|s| base.join(s.trim()).ok())
            .filter(|u| matches!(u.scheme(), "https" | "http"))
            .map(String::from);
        if let Some(url) = &artwork_url {
            entry["img"] = Value::String(url.clone());
        }
        chapters.push(Chapter {
            start,
            end: entry["endTime"].as_f64().filter(|end| *end > start),
            title: entry["title"].as_str().unwrap_or("Chapter").into(),
            artwork_url,
            hidden: entry["toc"].as_bool() == Some(false),
        });
    }
    ensure!(!chapters.is_empty(), "No timed chapters in document");
    chapters.sort_by(|a, b| a.start.total_cmp(&b.start));
    Ok((document, chapters))
}

fn reflex_item_url(raw: &str) -> Option<String> {
    let url = fetch::url(raw).ok()?;
    if url.scheme() != "https" || url.host_str()? != "reflex.livewire.io" || url.query().is_some() {
        return None;
    }
    let path = url.path().strip_prefix("/chapters/podcast/")?;
    let (podcast, item) = path.split_once("/item/")?;
    let (item, _) = item.split_once("/chapters/")?;
    if podcast.contains('/') || podcast.is_empty() || item.is_empty() {
        return None;
    }
    Some(format!(
        "https://reflex.livewire.io/chapters/podcast/{podcast}/item/{item}"
    ))
}
