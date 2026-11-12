# podcast/types.rs

Shared show, episode, preferences and durable processing-job contracts. IDs include the owning
device/feed and GUID; downloaded variants separately use immutable byte hashes. Feed URLs may
contain private feed tokens, so they must not be logged or embedded in public error messages.

Cut defaults keep banter/promos and low-confidence segments. A positive fade is a non-overlapping
edge fade, not a crossfade. Jobs retain the exact options and source hash used, analysis decisions,
source-to-output spans, original/edited metadata and errors. Only ready jobs label media cleaned.
Owner commands are typed and contain no executable commands or filesystem paths.

Player/peer catalogues omit heavy analysis/transcript/chapter documents and truncate episode
descriptions. Full processing records stay on the owner; playable Media still includes chapters.

Episode artwork is optional. Compact catalogues keep episode covers and chapter document links.
