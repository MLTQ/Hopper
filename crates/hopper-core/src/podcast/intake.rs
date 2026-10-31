use super::fetch;
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct FeedList {
    pub urls: Vec<String>,
    pub errors: Vec<String>,
}

pub fn opml(xml: &str) -> Result<FeedList> {
    ensure!(
        xml.len() <= 2 * 1024 * 1024,
        "subscription file exceeds 2 MB"
    );
    let doc = roxmltree::Document::parse(xml).context("choose an OPML subscription export")?;
    ensure!(
        doc.root_element()
            .tag_name()
            .name()
            .eq_ignore_ascii_case("opml"),
        "choose an OPML subscription export"
    );
    let mut result = FeedList::default();
    let mut seen = std::collections::HashSet::new();
    for entry in doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "outline")
    {
        let Some(raw) = entry
            .attribute("xmlUrl")
            .or_else(|| entry.attribute("xmlurl"))
        else {
            continue;
        };
        match normalize(raw) {
            Ok(url) => {
                if seen.insert(url.clone()) {
                    result.urls.push(url);
                }
            }
            Err(_) => result
                .errors
                .push("Skipped a feed with an invalid or unsupported address".into()),
        }
        ensure!(
            result.urls.len() <= 500,
            "import at most 500 subscriptions at a time"
        );
    }
    ensure!(
        !result.urls.is_empty(),
        "no RSS subscriptions found in this OPML file"
    );
    Ok(result)
}

pub fn normalize(raw: &str) -> Result<String> {
    ensure!(raw.len() <= 8192, "podcast link is too long");
    let raw = raw.trim();
    let raw = raw
        .strip_prefix("podcast://")
        .map(|s| format!("https://{s}"))
        .unwrap_or_else(|| raw.to_owned());
    let mut url = fetch::url(&raw)?;
    ensure!(
        !matches!(
            url.host_str(),
            Some("open.spotify.com" | "spotify.com" | "spotify.link")
        ),
        "Spotify import is not supported yet; use an RSS or Apple Podcasts link"
    );
    url.set_fragment(None);
    Ok(url.into())
}

pub fn apple_ids(raw: &str) -> Result<Option<(String, Option<String>)>> {
    let url = fetch::url(raw)?;
    if !matches!(
        url.host_str(),
        Some("podcasts.apple.com" | "itunes.apple.com")
    ) {
        return Ok(None);
    }
    let id = url
        .path_segments()
        .into_iter()
        .flatten()
        .filter_map(|s| s.strip_prefix("id"))
        .find(|id| !id.is_empty() && id.len() <= 20 && id.bytes().all(|b| b.is_ascii_digit()))
        .context("Apple Podcasts link is missing its show ID")?
        .to_owned();
    let episode = url
        .query_pairs()
        .find(|(k, _)| k == "i")
        .map(|(_, v)| v.into_owned());
    ensure!(
        episode.as_ref().is_none_or(|id| !id.is_empty()
            && id.len() <= 20
            && id.bytes().all(|b| b.is_ascii_digit())),
        "invalid Apple episode ID"
    );
    Ok(Some((id, episode)))
}

#[derive(Default)]
pub struct Resolved {
    pub feed_url: String,
    pub episode_url: Option<String>,
    pub episode_guid: Option<String>,
    pub episode_requested: bool,
}

pub async fn resolve(raw: &str) -> Result<Resolved> {
    let raw = normalize(raw)?;
    let Some((show, episode)) = apple_ids(&raw)? else {
        return Ok(Resolved {
            feed_url: raw,
            ..Default::default()
        });
    };
    // Apple exposes episodes through the parent show's lookup. Looking up an episode ID
    // directly returns no result for valid shares, so match within the bounded show listing.
    let entity = if episode.is_some() {
        "podcastEpisode&limit=200"
    } else {
        "podcast"
    };
    let (_, bytes) = fetch::document(&format!(
        "https://itunes.apple.com/lookup?id={show}&entity={entity}"
    ))
    .await?;
    let mut result = from_lookup(&bytes, &show)?;
    result.episode_requested = episode.is_some();
    if let Some(episode) = episode {
        attach_episode(&mut result, &bytes, &show, &episode);
    }
    Ok(result)
}

fn attach_episode(result: &mut Resolved, bytes: &[u8], show: &str, episode: &str) {
    if let Ok(doc) = serde_json::from_slice::<serde_json::Value>(bytes)
        && let Some(entry) = doc["results"].as_array().into_iter().flatten().find(|e| {
            e["trackId"].as_u64().map(|i| i.to_string()).as_deref() == Some(episode)
                && e["collectionId"].as_u64().map(|i| i.to_string()).as_deref() == Some(show)
        })
    {
        result.episode_url = entry["episodeUrl"].as_str().map(str::to_owned);
        result.episode_guid = entry["episodeGuid"].as_str().map(str::to_owned);
    }
}

fn from_lookup(bytes: &[u8], id: &str) -> Result<Resolved> {
    let doc: serde_json::Value = serde_json::from_slice(bytes)
        .context("Apple podcast lookup returned an invalid response")?;
    let feed = doc["results"].as_array().into_iter().flatten().find(|e| {
        e["collectionId"].as_u64().or_else(||e["trackId"].as_u64()).map(|n|n.to_string()).as_deref()==Some(id) && e["feedUrl"].is_string()
    }).and_then(|e|e["feedUrl"].as_str()).context("Apple did not provide a public RSS feed for this show; use the publisher's feed link if available")?;
    Ok(Resolved {
        feed_url: normalize(feed)?,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exports_and_apple_links_are_validated_without_guessing() {
        let feeds = opml(r#"<opml version="2.0"><body><outline text="Folder"><outline xmlUrl="https://example.org/rss?a=1&amp;b=2"/><outline xmlUrl="https://example.org/rss?a=1&amp;b=2#fragment"/><outline xmlUrl="file:///private/audio"/></outline></body></opml>"#).unwrap();
        assert_eq!(feeds.urls, vec!["https://example.org/rss?a=1&b=2"]);
        assert_eq!(feeds.errors.len(), 1);
        assert_eq!(
            apple_ids("https://podcasts.apple.com/us/podcast/ideas-for-today/id123?i=456").unwrap(),
            Some(("123".into(), Some("456".into())))
        );
        assert!(
            apple_ids("https://podcasts.apple.com.evil.test/id123")
                .unwrap()
                .is_none()
        );
        assert!(normalize("https://open.spotify.com/show/123").is_err());
        assert!(opml("<rss/>").is_err());
        let result = from_lookup(
            br#"{"results":[{"collectionId":123,"feedUrl":"https://example.org/rss"}]}"#,
            "123",
        )
        .unwrap();
        assert_eq!(result.feed_url, "https://example.org/rss");
        assert!(from_lookup(br#"{"results":[]}"#, "123").is_err());
        let mut resolved = Resolved::default();
        let episodes = br#"{"results":[{"collectionId":123,"trackId":456,"episodeGuid":"guid-1","episodeUrl":"https://example.org/audio.mp3"}]}"#;
        attach_episode(&mut resolved, episodes, "789", "456");
        assert!(resolved.episode_guid.is_none());
        attach_episode(&mut resolved, episodes, "123", "456");
        assert_eq!(resolved.episode_guid.as_deref(), Some("guid-1"));
    }
}
