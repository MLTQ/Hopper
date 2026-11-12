use super::{Category, Listing, Page, Query, fetch};
use anyhow::{Context, Result, ensure};
use reqwest::Url;
use serde::Deserialize;

const GENRES: &[(&str, &str)] = &[
    ("1301", "Arts"),
    ("1321", "Business"),
    ("1303", "Comedy"),
    ("1304", "Education"),
    ("1483", "Fiction"),
    ("1511", "Government"),
    ("1512", "Health & Fitness"),
    ("1487", "History"),
    ("1305", "Kids & Family"),
    ("1502", "Leisure"),
    ("1310", "Music"),
    ("1489", "News"),
    ("1314", "Religion & Spirituality"),
    ("1533", "Science"),
    ("1324", "Society & Culture"),
    ("1545", "Sports"),
    ("1318", "Technology"),
    ("1488", "True Crime"),
    ("1309", "TV & Film"),
];

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Show {
    collection_name: Option<String>,
    artist_name: Option<String>,
    feed_url: Option<String>,
    artwork_url100: Option<String>,
    artwork_url600: Option<String>,
    #[serde(default)]
    genre_ids: Vec<String>,
}

fn search_results(bytes: &[u8], category: &str) -> Result<Vec<Listing>> {
    #[derive(Deserialize)]
    struct Search {
        results: Vec<Show>,
    }
    let doc: Search =
        serde_json::from_slice(bytes).context("Apple returned an unreadable search response.")?;
    Ok(doc
        .results
        .iter()
        .filter(|s| category.is_empty() || s.genre_ids.iter().any(|id| id == category))
        .filter_map(|s| {
            Listing::new(
                s.collection_name.as_deref()?,
                s.artist_name.as_deref().unwrap_or_default(),
                s.feed_url.as_deref()?,
                s.artwork_url100.as_deref().or(s.artwork_url600.as_deref()),
            )
        })
        .collect())
}

fn chart_results(bytes: &[u8]) -> Result<Vec<Listing>> {
    let doc: serde_json::Value = serde_json::from_slice(bytes)?;
    let entries = doc["feed"]["entry"]
        .as_array()
        .context("Apple returned an unreadable podcast chart.")?;
    Ok(entries
        .iter()
        .filter_map(|s| {
            let id = s["id"]["attributes"]["im:id"].as_str()?;
            if id.is_empty() || id.len() > 20 || !id.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            let url = format!("https://podcasts.apple.com/us/podcast/id{id}");
            Listing::new(
                s["im:name"]["label"].as_str()?,
                s["im:artist"]["label"].as_str().unwrap_or_default(),
                &url,
                s["im:image"]
                    .as_array()
                    .and_then(|a| a.last())
                    .and_then(|v| v["label"].as_str()),
            )
        })
        .collect())
}

pub(super) async fn browse(query: &Query) -> Result<Page> {
    let category = query.category.as_str();
    ensure!(
        category.is_empty() || GENRES.iter().any(|(id, _)| *id == category),
        "Choose an Apple Podcasts category."
    );
    let term = query.term.trim();
    let shows = if term.is_empty() {
        let genre = if category.is_empty() {
            String::new()
        } else {
            format!("/genre={category}")
        };
        let url = format!("https://itunes.apple.com/us/rss/toppodcasts/limit=25{genre}/json");
        chart_results(&fetch::document(&url).await?.1)?
    } else {
        let mut url = Url::parse("https://itunes.apple.com/search")?;
        url.query_pairs_mut().extend_pairs([
            ("country", "US"),
            ("media", "podcast"),
            ("entity", "podcast"),
            ("term", term),
            ("limit", if category.is_empty() { "40" } else { "200" }),
        ]);
        search_results(&fetch::document(url.as_str()).await?.1, category)?
    };
    Ok(Page::new(
        shows.into_iter(),
        GENRES
            .iter()
            .map(|(id, name)| Category {
                id: id.to_string(),
                name: name.to_string(),
            })
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn search_normalizes_rss_and_filters_genres() {
        let bytes = serde_json::to_vec(&json!({"results":[
            {"collectionName":"Adventure","artistName":"Players","genreIds":["1483"],"feedUrl":"https://example.org/rss","artworkUrl100":"https://example.org/art.jpg"},
            {"collectionName":"News","genreIds":["1489"],"feedUrl":"https://example.org/news"},
            {"collectionName":"No public feed","genreIds":["1483"]}
        ]})).unwrap();
        let rows = search_results(&bytes, "1483").unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].title, "Adventure");
        assert_eq!(
            rows[0].artwork.as_deref(),
            Some("https://example.org/art.jpg")
        );
    }
    #[test]
    fn chart_retains_order_and_yields_resolvable_share_links() {
        let bytes=serde_json::to_vec(&json!({"feed":{"entry":[
            {"im:name":{"label":"Adventure"},"id":{"attributes":{"im:id":"42"}},"im:image":[{"label":"https://example.org/art.jpg"}]},
            {"im:name":{"label":"Invalid"},"id":{"attributes":{"im:id":"x"}}}
        ]}})).unwrap();
        let shows = chart_results(&bytes).unwrap();
        assert_eq!(shows.len(), 1);
        assert_eq!(
            crate::podcast::intake::apple_ids(&shows[0].url)
                .unwrap()
                .unwrap()
                .0,
            "42"
        );
    }
}
