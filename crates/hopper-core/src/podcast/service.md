# podcast/service.rs

Authoritative show-owner commands: subscribe/refresh, settings, unsubscribe, download, queue/cancel/
retry processing. Paired clients forward typed commands to that owner and refresh the catalogue.
Subscriptions refresh periodically; enabling automatic processing applies to newly discovered
episodes, never silently the entire existing backlog. Explicit cut requests are bounded to 500.

Feed GUIDs retain download/cleaned associations across URL changes. Download originals once into
managed storage, preserving byte hashes, chapter documents and transcript references. A show
unsubscribe cancels pending work but retains downloaded media. Download/refresh mutations are
serialized to avoid losing variants. Unsupported transcript formats produce metadata warnings.
Jobs and media live in SQLite and private managed storage; private local paths stay off the mesh.

Batch cuts check existing jobs/options before creating any jobs, avoiding partial batches on conflicts.

Publishing a downloaded episode reuses an existing identical managed file so overlapping library folder sources keep their valid timestamps.

External chapter parsing retains image URLs and silent markers on playable original media.

CutEpisode queues a single episode of a followed show on its owner and leaves saved show preferences and automatic processing intact.
