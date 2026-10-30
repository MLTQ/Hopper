# Product roadmap

The standalone direction retains the original Hopper vision. Current work implements the
paired-library playback foundation. Beads epic `hopper-lky` holds implementation tasks.

## Next: make everyday listening dependable
- Real iPhone locked playback across multiple episodes, calls, Bluetooth and cellular handoff.
- Linux/Windows installers and runtime validation, macOS signing/distribution.
- Resumable offline downloads with progress, storage limits, pinning and automatic prefetch.
- Better file-change handling, watched folders and explicit selective replication policies.
- Embedded artwork, complete M4B metadata, codec capability reporting and conversion fallback.
- Device naming/settings, pairing QR codes and easier multi-device trust setup.
- Queue portability/handoff, sleep timers and bookmarks with notes.

## Podcast preparation
- Subscribe to RSS/private feeds, preserve stable source identity, seasons and episode order.
- Durable processing jobs prioritize the next queued episodes and expose failures/costs.
- Agent skill orchestrates multimodal audio analysis; classify ads conservatively with boundaries,
  evidence and confidence. Per-show preferences distinguish paid ads, promos, intros and recaps.
- Preserve original media plus a reversible edit list bound to the exact source file version.
- Produce edited media and a source-to-edited timeline map for progress, chapters, transcripts,
  artwork changes and soundbites. Corrections can restore wrongly removed content.
- Publish private per-show cleaned feeds. Processed files remain streamable; live sources initially
  pass through without edits. Do not label an unprocessed file as cleaned.

## Podcasting 2.0 and search
- Track ingest/preserve/export/display support for each namespace feature explicitly.
- Support chapters/artwork, transcripts, alternate enclosures, live items and creator metadata;
  retime all time-based metadata when generating an edited variant.
- Exact and vector search over titles and timestamped transcript chunks with playable results.
- Limit search to listened content as an optional spoiler guard for narrative shows/books.

## Maintainability
Core APIs and media identities are independent of the GUI and of Gruve. Optional Gruve discovery
integration can follow without making it a runtime dependency. Keep original files and exportable
state, document conflict semantics, and preserve meaningful regression tests.
