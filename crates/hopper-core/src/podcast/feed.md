# podcast/feed.rs

Bounded RSS parsing with namespace-aware XML tokenization, stable GUID-based identity, dates,
duration, artwork, enclosure links, Podcasting 2.0 chapter/transcript references and soundbites.
roxmltree does not fetch external entities; no feed content executes. Relative URLs resolve
against the final feed URL. HTTP(S) enclosures only. Feeds are capped at 5,000 entries and duplicate
GUIDs collapse. The service archives original XML for future namespace support.

WebVTT and SRT transcripts become timestamped cues. Unsupported formats are recorded as warnings,
never exported with stale timestamps as if they had been remapped. Podcast JSON chapter documents
retain unknown fields when the service downloads them. Tests exercise namespaces and time parsing.
