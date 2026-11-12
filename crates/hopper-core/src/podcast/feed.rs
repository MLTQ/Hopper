use super::types::*;
use anyhow::{Context, Result, ensure};
use reqwest::Url;
use roxmltree::Node;

pub struct ParsedFeed {
    pub title: String,
    pub author: String,
    pub description: String,
    pub artwork: Option<String>,
    pub episodes: Vec<Episode>,
}
fn child<'a, 'i>(n: Node<'a, 'i>, tag: &str) -> Option<Node<'a, 'i>> {
    n.children()
        .find(|c| c.is_element() && c.tag_name().name() == tag)
}
fn value(n: Node<'_, '_>, tag: &str) -> String {
    child(n, tag)
        .and_then(|n| n.text())
        .unwrap_or("")
        .trim()
        .chars()
        .take(16000)
        .collect()
}
fn resolved(base: &Url, value: Option<&str>) -> Option<String> {
    let raw = value?.trim();
    if raw.is_empty() {
        return None;
    }
    let u = base.join(raw).ok()?;
    matches!(u.scheme(), "http" | "https").then(|| u.to_string())
}
fn cover(node: Node<'_, '_>, base: &Url) -> Option<String> {
    node.children()
        .filter(|n| n.is_element() && n.tag_name().name() == "image")
        .find_map(|n| {
            resolved(base, n.attribute("href"))
                .or_else(|| resolved(base, child(n, "url").and_then(|n| n.text())))
        })
        .or_else(|| {
            let images = child(node, "images")?.attribute("srcset")?;
            images
                .split(',')
                .find_map(|entry| resolved(base, entry.split_whitespace().next()))
        })
}

pub fn seconds(text: &str) -> Option<f64> {
    let mut total = 0.0;
    for p in text.trim().replace(',', ".").split(':') {
        total = total * 60.0 + p.parse::<f64>().ok()?;
    }
    (total.is_finite() && total >= 0.0).then_some(total)
}
pub fn parse(xml: &str, base: &str, show_id: &str) -> Result<ParsedFeed> {
    let document =
        roxmltree::Document::parse(xml).context("the address did not return a valid RSS feed")?;
    let channel = document
        .descendants()
        .find(|n| n.has_tag_name("channel"))
        .context("an RSS podcast feed is required")?;
    let base = Url::parse(base)?;
    let title = value(channel, "title");
    ensure!(!title.is_empty(), "podcast feed has no title");
    let artwork = cover(channel, &base);
    let mut episodes = vec![];
    let mut seen = std::collections::HashSet::new();
    for (index, item) in channel
        .children()
        .filter(|n| n.has_tag_name("item"))
        .take(5000)
        .enumerate()
    {
        let Some(enclosure) = child(item, "enclosure") else {
            continue;
        };
        let Some(url) = resolved(&base, enclosure.attribute("url")) else {
            continue;
        };
        let mime = enclosure
            .attribute("type")
            .unwrap_or("audio/mpeg")
            .to_string();
        if !mime.starts_with("audio/") && !mime.starts_with("video/") {
            continue;
        }
        let guid = value(item, "guid");
        let guid = if guid.is_empty() { url.clone() } else { guid };
        let id = blake3::hash(format!("{show_id}\0{guid}").as_bytes())
            .to_hex()
            .to_string();
        if !seen.insert(id.clone()) {
            continue;
        }
        let chapter = item.children().find(|n| {
            n.is_element() && n.tag_name().name() == "chapters" && n.attribute("url").is_some()
        });
        let transcript = item.children().find(|n| {
            n.is_element() && n.tag_name().name() == "transcript" && n.attribute("url").is_some()
        });
        let published = chrono::DateTime::parse_from_rfc2822(&value(item, "pubDate"))
            .map(|d| d.timestamp())
            .or_else(|_| {
                chrono::DateTime::parse_from_rfc3339(&value(item, "pubDate")).map(|d| d.timestamp())
            })
            .unwrap_or(-(index as i64));
        let chapters = item
            .descendants()
            .filter(|n| n.is_element() && n.tag_name().name() == "chapter")
            .filter_map(|n| {
                Some(crate::model::Chapter {
                    title: n.attribute("title").unwrap_or("Chapter").into(),
                    start: seconds(n.attribute("start")?)?,
                    end: None,
                    artwork_url: resolved(&base, n.attribute("image")),
                    hidden: false,
                })
            })
            .collect();
        let soundbites = item
            .children()
            .filter(|n| n.is_element() && n.tag_name().name() == "soundbite")
            .filter_map(|n| {
                Some(Soundbite {
                    start: seconds(n.attribute("startTime")?)?,
                    duration: seconds(n.attribute("duration")?)?,
                    title: n.text().unwrap_or("").into(),
                })
            })
            .collect();
        let title = value(item, "title");
        episodes.push(Episode {
            id,
            show_id: show_id.into(),
            guid,
            title: if title.is_empty() {
                "Untitled episode".into()
            } else {
                title
            },
            description: value(item, "description"),
            enclosure_url: url,
            mime,
            published,
            duration: seconds(&value(item, "duration")),
            artwork_url: cover(item, &base),
            chapters_url: chapter.and_then(|n| resolved(&base, n.attribute("url"))),
            transcript_url: transcript.and_then(|n| resolved(&base, n.attribute("url"))),
            transcript_type: transcript
                .and_then(|n| n.attribute("type"))
                .map(String::from),
            original_id: None,
            cleaned_id: None,
            chapters,
            chapter_document: None,
            transcript: vec![],
            soundbites,
            metadata_warnings: vec![],
        });
    }
    Ok(ParsedFeed {
        title,
        author: value(channel, "author"),
        description: value(channel, "description"),
        artwork,
        episodes,
    })
}
pub fn transcript(text: &str) -> Result<Vec<Cue>> {
    let text = text.replace("\r\n", "\n");
    let mut cues = vec![];
    for block in text.split("\n\n") {
        let lines: Vec<_> = block.lines().collect();
        let Some(i) = lines.iter().position(|l| l.contains(" --> ")) else {
            continue;
        };
        let (start, end) = lines[i].split_once(" --> ").unwrap();
        let start = seconds(start).context("invalid transcript start")?;
        let end = seconds(end.split_whitespace().next().unwrap_or(""))
            .context("invalid transcript end")?;
        ensure!(end >= start, "invalid transcript range");
        cues.push(Cue {
            start,
            end,
            text: lines[i + 1..].join("\n"),
        });
    }
    ensure!(
        !cues.is_empty(),
        "transcript is not supported WebVTT or SRT"
    );
    Ok(cues)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rss_namespace_guid_and_relative_links() {
        let xml = r#"<rss xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd" xmlns:podcast="https://podcastindex.org/namespace/1.0"><channel><title>A &amp; B</title><itunes:author>Hosts</itunes:author><item><title>First</title><guid isPermaLink="false">ep-1</guid><itunes:duration>1:02:03</itunes:duration><enclosure url="episode.mp3" type="audio/mpeg"/><podcast:chapters url="chapters.json" type="application/json+chapters"/></item></channel></rss>"#;
        let p = parse(xml, "https://example.org/feed.xml", "show").unwrap();
        assert_eq!(p.title, "A & B");
        assert_eq!(p.episodes[0].duration, Some(3723.0));
        assert_eq!(
            p.episodes[0].enclosure_url,
            "https://example.org/episode.mp3"
        );
        assert_eq!(
            p.episodes[0].chapters_url.as_deref(),
            Some("https://example.org/chapters.json")
        );
        assert_eq!(
            transcript("WEBVTT\n\n00:00:01.000 --> 00:00:02.000\nHi").unwrap()[0].start,
            1.0
        );
        assert_eq!(
            transcript("1\n00:01,000 --> 00:02,000\nHi").unwrap()[0].end,
            2.0
        );
    }
}
