use super::types::*;
use crate::model::Chapter;
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Span {
    pub source_start: f64,
    pub source_end: f64,
    pub output_start: f64,
}

pub fn plan(
    analysis: &Analysis,
    options: &CutOptions,
    hash: &str,
    duration: f64,
) -> Result<Vec<Span>> {
    options.validate()?;
    ensure!(
        analysis.source_hash == hash,
        "agent result belongs to a different source file"
    );
    ensure!(
        duration.is_finite()
            && duration > 0.0
            && analysis.duration.is_finite()
            && (analysis.duration - duration).abs() < 0.25,
        "agent duration does not match the source"
    );
    ensure!(analysis.segments.len() <= 1000, "too many edit segments");
    let mut cuts = Vec::<(f64, f64)>::new();
    let mut protected = Vec::<(f64, f64)>::new();
    for s in &analysis.segments {
        ensure!(
            s.start.is_finite()
                && s.end.is_finite()
                && s.start >= 0.0
                && s.end > s.start
                && s.end <= duration + 0.01,
            "agent returned an invalid time range"
        );
        ensure!(
            s.confidence.is_finite()
                && (0.0..=1.0).contains(&s.confidence)
                && matches!(
                    s.kind.as_str(),
                    "ad" | "banter" | "promo" | "content" | "uncertain"
                ),
            "invalid agent classification"
        );
        if s.confidence >= options.minimum_confidence
            && (s.kind == "ad"
                || s.kind == "banter" && !options.keep_banter
                || s.kind == "promo" && !options.keep_promos)
        {
            cuts.push((s.start, s.end.min(duration)));
        } else {
            protected.push((s.start, s.end.min(duration)));
        }
    }
    // Conflicting classifications resolve toward preservation, including kept banter/promos.
    for (start, end) in protected {
        cuts = cuts
            .into_iter()
            .flat_map(|(a, b)| {
                if end <= a || start >= b {
                    return vec![(a, b)];
                }
                let mut pieces = vec![];
                if a < start {
                    pieces.push((a, start));
                }
                if end < b {
                    pieces.push((end, b));
                }
                pieces
            })
            .collect();
    }
    cuts.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut merged = Vec::<(f64, f64)>::new();
    for (start, end) in cuts {
        if let Some(last) = merged.last_mut().filter(|last| start <= last.1) {
            last.1 = last.1.max(end);
        } else {
            merged.push((start, end));
        }
    }
    let (mut cursor, mut output) = (0.0, 0.0);
    let mut spans = vec![];
    for (start, end) in merged
        .into_iter()
        .chain(std::iter::once((duration, duration)))
    {
        if start > cursor {
            spans.push(Span {
                source_start: cursor,
                source_end: start,
                output_start: output,
            });
            output += start - cursor;
        }
        cursor = end;
    }
    ensure!(
        output >= 0.25,
        "cuts would remove the whole episode; original preserved"
    );
    Ok(spans)
}
pub fn duration(spans: &[Span]) -> f64 {
    spans
        .last()
        .map_or(0.0, |s| s.output_start + s.source_end - s.source_start)
}
pub fn map_time(t: f64, spans: &[Span]) -> f64 {
    for s in spans {
        if t < s.source_start {
            return s.output_start;
        }
        if t <= s.source_end {
            return s.output_start + (t - s.source_start);
        }
    }
    duration(spans)
}
pub fn chapters(chapters: &[Chapter], spans: &[Span], source_duration: f64) -> Vec<Chapter> {
    chapters
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let end = c
                .end
                .unwrap_or_else(|| chapters.get(i + 1).map_or(source_duration, |n| n.start));
            let start = map_time(c.start, spans);
            let end = map_time(end, spans);
            (end > start && start < duration(spans)).then(|| Chapter {
                title: c.title.clone(),
                artwork_url: c.artwork_url.clone(),
                hidden: c.hidden,
                start,
                end: Some(end),
            })
        })
        .collect()
}
pub fn cues(cues: &[Cue], spans: &[Span]) -> Vec<Cue> {
    // Split cues crossing a cut; never leave a cue active across an excised interval.
    cues.iter()
        .flat_map(|c| {
            spans.iter().filter_map(move |s| {
                let start = c.start.max(s.source_start);
                let end = c.end.min(s.source_end);
                (end > start).then(|| Cue {
                    start: s.output_start + start - s.source_start,
                    end: s.output_start + end - s.source_start,
                    text: c.text.clone(),
                })
            })
        })
        .collect()
}
pub fn chapter_document(
    doc: &serde_json::Value,
    spans: &[Span],
    source_duration: f64,
) -> serde_json::Value {
    let mut out = doc.clone();
    if let Some(entries) = doc.get("chapters").and_then(|v| v.as_array()) {
        let mut entries: Vec<_> = entries
            .iter()
            .filter(|entry| {
                entry
                    .get("startTime")
                    .and_then(|v| v.as_f64())
                    .is_some_and(|t| t >= 0.0 && t < source_duration)
            })
            .collect();
        entries.sort_by(|a, b| {
            a["startTime"]
                .as_f64()
                .unwrap()
                .total_cmp(&b["startTime"].as_f64().unwrap())
        });
        let mut result = vec![];
        for (i, entry) in entries.iter().enumerate() {
            let Some(start) = entry.get("startTime").and_then(|v| v.as_f64()) else {
                continue;
            };
            let end = entry
                .get("endTime")
                .and_then(|v| v.as_f64())
                .or_else(|| entries.get(i + 1)?.get("startTime")?.as_f64())
                .unwrap_or(source_duration);
            let mapped = map_time(start, spans);
            let mapped_end = map_time(end, spans);
            if mapped_end <= mapped {
                continue;
            }
            let mut entry = (*entry).clone();
            entry["startTime"] = mapped.into();
            entry["endTime"] = mapped_end.into();
            result.push(entry);
        }
        out["chapters"] = result.into();
    }
    out
}
pub fn soundbites(items: &[Soundbite], spans: &[Span]) -> Vec<Soundbite> {
    items
        .iter()
        .filter_map(|s| {
            let start = map_time(s.start, spans);
            let end = map_time(s.start + s.duration, spans);
            (end > start).then(|| Soundbite {
                start,
                duration: end - start,
                title: s.title.clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overlaps_preferences_and_metadata_share_one_timeline() {
        let a = Analysis {
            source_hash: "x".into(),
            duration: 100.0,
            segments: vec![
                Candidate {
                    start: 10.0,
                    end: 20.0,
                    kind: "ad".into(),
                    confidence: 0.99,
                    reason: "sponsor".into(),
                },
                Candidate {
                    start: 18.0,
                    end: 30.0,
                    kind: "ad".into(),
                    confidence: 0.95,
                    reason: "sponsor".into(),
                },
                Candidate {
                    start: 40.0,
                    end: 50.0,
                    kind: "banter".into(),
                    confidence: 0.99,
                    reason: "conversation".into(),
                },
                Candidate {
                    start: 60.0,
                    end: 70.0,
                    kind: "ad".into(),
                    confidence: 0.6,
                    reason: "uncertain".into(),
                },
            ],
        };
        let s = plan(&a, &CutOptions::default(), "x", 100.0).unwrap();
        assert_eq!(duration(&s), 80.0);
        assert_eq!(map_time(45.0, &s), 25.0);
        let doc = serde_json::json!({"version":"1.2.0","chapters":[{"startTime":0,"title":"Intro"},{"startTime":10,"title":"Ad"},{"startTime":30,"title":"Story","img":"https://example.com/art.jpg"}]});
        let edited = chapter_document(&doc, &s, 100.0);
        assert_eq!(edited["chapters"].as_array().unwrap().len(), 2);
        assert_eq!(edited["chapters"][1]["startTime"], 10.0);
        assert!(edited["chapters"][1]["img"].is_string());
        let t = cues(
            &[Cue {
                start: 8.0,
                end: 32.0,
                text: "across cut".into(),
            }],
            &s,
        );
        assert_eq!(t.len(), 2);
        assert_eq!(t[1].start, 10.0);
        assert_eq!(t[1].end, 12.0);
        assert!(plan(&a, &CutOptions::default(), "wrong", 100.0).is_err());
        let mut overlapping = a.clone();
        overlapping.segments.push(Candidate {
            start: 15.0,
            end: 17.0,
            kind: "uncertain".into(),
            confidence: 0.99,
            reason: "Boundary conflict".into(),
        });
        assert_eq!(
            duration(&plan(&overlapping, &CutOptions::default(), "x", 100.0).unwrap()),
            82.0
        );
        let options = CutOptions {
            keep_banter: false,
            ..Default::default()
        };
        assert_eq!(duration(&plan(&a, &options, "x", 100.0).unwrap()), 70.0);
    }
}
