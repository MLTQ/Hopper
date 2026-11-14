use crate::model::{Chapter, Device};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct CutOptions {
    pub keep_banter: bool,
    pub keep_promos: bool,
    pub fade_ms: u32,
    pub minimum_confidence: f64,
    pub auto_process: bool,
}
impl Default for CutOptions {
    fn default() -> Self {
        Self {
            keep_banter: true,
            keep_promos: true,
            fade_ms: 20,
            minimum_confidence: 0.9,
            auto_process: false,
        }
    }
}
impl CutOptions {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.fade_ms <= 500
                && self.minimum_confidence.is_finite()
                && (0.5..=1.0).contains(&self.minimum_confidence),
            "invalid cut options"
        );
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Show {
    pub id: String,
    pub owner: Device,
    pub feed_url: String,
    pub title: String,
    pub author: String,
    pub description: String,
    pub artwork_url: Option<String>,
    pub newest_first: bool,
    pub cut_options: CutOptions,
    pub followed: bool,
    pub updated_at: u64,
    pub last_checked: u64,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Episode {
    pub id: String,
    pub show_id: String,
    pub guid: String,
    pub title: String,
    pub description: String,
    pub enclosure_url: String,
    pub mime: String,
    pub published: i64,
    pub duration: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artwork_url: Option<String>,
    pub chapters_url: Option<String>,
    pub transcript_url: Option<String>,
    pub transcript_type: Option<String>,
    pub original_id: Option<String>,
    pub cleaned_id: Option<String>,
    #[serde(default)]
    pub chapters: Vec<Chapter>,
    #[serde(default)]
    pub chapter_document: Option<serde_json::Value>,
    #[serde(default)]
    pub transcript: Vec<Cue>,
    #[serde(default)]
    pub soundbites: Vec<Soundbite>,
    #[serde(default)]
    pub metadata_warnings: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Cue {
    pub start: f64,
    pub end: f64,
    pub text: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Soundbite {
    pub start: f64,
    pub duration: f64,
    pub title: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Candidate {
    pub start: f64,
    pub end: f64,
    pub kind: String,
    pub confidence: f64,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Analysis {
    pub source_hash: String,
    pub duration: f64,
    pub segments: Vec<Candidate>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub episode_id: String,
    pub show_id: String,
    pub options: CutOptions,
    pub state: String,
    pub message: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub source_hash: Option<String>,
    pub output_id: Option<String>,
    pub removed_seconds: f64,
    pub analysis: Option<Analysis>,
    pub timeline: Vec<super::timeline::Span>,
    pub cleaned_chapter_document: Option<serde_json::Value>,
    pub cleaned_transcript: Vec<Cue>,
    pub cleaned_soundbites: Vec<Soundbite>,
}
impl Job {
    pub fn active(&self) -> bool {
        matches!(
            self.state.as_str(),
            "queued" | "downloading" | "analysing" | "rendering" | "waiting_for_agent"
        )
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Catalogue {
    pub shows: Vec<Show>,
    pub episodes: Vec<Episode>,
    pub jobs: Vec<Job>,
    #[serde(default)]
    pub downloads: Vec<super::downloads::Download>,
    pub agent_available: bool,
    pub desktop: bool,
    #[serde(default)]
    pub hosts: Vec<Host>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Host {
    pub device: Device,
    pub online: bool,
    pub desktop: bool,
    pub agent_available: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Command {
    Subscribe {
        url: String,
    },
    Refresh {
        show_id: String,
    },
    Settings {
        show_id: String,
        newest_first: bool,
        cut_options: CutOptions,
    },
    Unsubscribe {
        show_id: String,
    },
    Download {
        episode_id: String,
    },
    QueueDownload {
        episode_id: String,
    },
    CutEpisode {
        episode_id: String,
        options: CutOptions,
    },
    Cut {
        show_id: String,
        episode_ids: Vec<String>,
        options: CutOptions,
    },
    Retry {
        job_id: String,
    },
    Cancel {
        job_id: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CommandResult {
    pub show_id: Option<String>,
    pub media_id: Option<String>,
    pub job_ids: Vec<String>,
}

impl Catalogue {
    pub(crate) fn sort_by_latest_episode(&mut self) {
        let mut newest = std::collections::HashMap::<&str, i64>::new();
        for episode in &self.episodes {
            if episode.published > 0 {
                let date = newest.entry(&episode.show_id).or_default();
                *date = (*date).max(episode.published);
            }
        }
        self.shows.sort_by_cached_key(|show| {
            (
                std::cmp::Reverse(newest.get(show.id.as_str()).copied()),
                show.title.to_lowercase(),
                show.id.clone(),
            )
        });
    }

    /// Player catalogues carry status and references; full editing documents remain on the owner.
    pub(crate) fn for_sharing(mut self) -> Self {
        for episode in &mut self.episodes {
            episode.description = episode.description.chars().take(500).collect();
            episode.transcript.clear();
            episode.chapter_document = None;
            episode.chapters.clear(); // Playable media carries its mapped chapter list.
        }
        for job in &mut self.jobs {
            job.analysis = None;
            job.timeline.clear();
            job.cleaned_chapter_document = None;
            job.cleaned_transcript.clear();
            job.cleaned_soundbites.clear();
        }
        self
    }
}

#[cfg(test)]
mod recency_tests {
    use super::*;
    fn show(id: &str, title: &str, last_checked: u64) -> Show {
        Show {
            id: id.into(),
            title: title.into(),
            last_checked,
            updated_at: last_checked,
            owner: Device {
                id: id.into(),
                name: id.into(),
            },
            feed_url: format!("https://example.org/{id}"),
            author: String::new(),
            description: String::new(),
            artwork_url: None,
            newest_first: false,
            cut_options: Default::default(),
            followed: true,
            error: None,
        }
    }
    #[test]
    fn merged_shows_sort_by_max_episode_date_across_owners() {
        let mut catalogue = Catalogue {
            shows: vec![
                show("old", "Old", 9900),
                show("undated", "A", 9999),
                show("new", "Z", 10),
                show("tie", "B", 30),
            ],
            ..Default::default()
        };
        for (id, date) in [("new", 300), ("old", 100), ("new", 7), ("tie", 300)] {
            let feed = r#"<rss><channel><title>X</title><item><title>Episode</title><guid>x</guid><enclosure url="https://example.org/episode.mp3" type="audio/mpeg"/></item></channel></rss>"#;
            let mut episode = crate::podcast::feed::parse(feed, "https://example.org/rss", id)
                .unwrap()
                .episodes
                .remove(0);
            episode.published = date;
            catalogue.episodes.push(episode);
        }
        catalogue.sort_by_latest_episode();
        assert_eq!(
            catalogue
                .shows
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            ["tie", "new", "old", "undated"]
        );
    }
}
