# Architecture

## Product contract
Every desktop can add files and play them. The iOS app is a native listener with the same paired
library. A desktop is not inherently the central server: each device owns its imported media,
and paired peers exchange catalogues and listening progress. The UI identifies file locations.
"Keep on this device" makes a verified copy. No automatic deletion or full-library replication.

## Components
- `hopper-core`: Rust node, SQLite catalogue, Iroh identity/transport, local authenticated HTTP API.
- `apps/desktop`: Tauri shell and bundled HTML/CSS/JavaScript UI; same code targets all desktops.
- `apps/ios`: SwiftUI UI and native AVQueuePlayer; links the Rust core as a static library.
- `hopper-node`: optional headless service on always-on desktops/Linux boxes.

## Identity and pairing
Each installation persists an Iroh key. One-time, expiring invitations authorize a second key.
Pair devices explicitly; this first release does not silently extend trust to friends of peers.
Pair the phone with each desktop, and pair desktops whose libraries should be visible together.
All peer requests except invitation redemption require a paired connection identity.
Revocation removes local trust and cached catalogue; it cannot erase files already downloaded.

## Media and state
Media IDs are BLAKE3 hashes of original file bytes; identical imports merge as one library item.
Local filesystem paths never cross the mesh. Logical folder paths are metadata only.
Metadata may be read with ffprobe on desktop; absence must not prevent a supported import.
Imported paths can be referenced in place; uploads and offline copies are managed by Hopper.
Source files must remain present and unchanged to be served.

Catalogues are cached for offline browsing. A catalogue item can have several sources; prefer a
local file, then reachable paired peers. Progress uses Lamport revisions plus device-ID tie-breaks,
so an intentional rewind after synchronization wins. Simultaneous offline listening resolves
deterministically; it cannot infer which human intent is newest. Queue order is device-local in
the first release. Transport is Iroh QUIC with default discovery/relay infrastructure.

## Security boundaries
The HTTP listener binds loopback only and requires a per-run bearer secret. Shells receive this
secret directly; browser startup carries it in the URL fragment, not an HTTP query or referrer.
Media uses separate random grants scoped to an item for native/browser playback. Peer media
requests resolve catalogue IDs and bounded byte ranges, never caller-provided filesystem paths.
Database, identity and the data directory are private to the local OS user.

## Scope and validation
The initial vertical slice is import → pair → browse → stream/cache → queue → resume. Podcast
subscriptions, AI processing, timeline remapping, full Podcasting 2.0 support, transcript/vector
search, watched-folder reconciliation and richer codec conversion remain tracked product work.
Cross-platform source/build workflows do not substitute for actual Windows/Linux/device testing.
