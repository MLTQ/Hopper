# podcast/mod.rs

Podcast subscriptions and processing are a core capability independent of either UI. Modules
separate public contracts, RSS parsing, bounded downloads, durable jobs, edit arithmetic and rendering.
The show owner is authoritative. Paired clients send commands to that owner; originals remain intact.
iOS can browse/play and submit jobs to a desktop, but never launches an agent or FFmpeg process.

Agent configuration is separate from analysis adapters; llm handles actual bounded audio input.

intake parses OPML and resolves supported share links for the Home delivery receiver.

The chapter reader is shared by owner preparation and on-demand original chapter recovery.

The directory module normalizes Apple and Podcast Index listings for both shells.

downloads schedules original episode transfers independently of ad analysis.
