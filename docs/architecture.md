# Architecture

## Product contract
Every desktop can add files and play them. The iOS app is a native listener with the same paired
library. A desktop is not inherently the central server: each device owns its imported media,
and paired peers exchange catalogues and listening progress. The UI identifies file locations.
"Keep on this device" makes a verified copy. Automatic iPhone playback copies have separate storage and expire on completion or after 30 days.

## Components
- `hopper-core`: Rust node, SQLite catalogue, Iroh identity/transport, local authenticated HTTP API.
- `apps/desktop`: Tauri shell and bundled HTML/CSS/JavaScript UI; same code targets all desktops.
- `apps/ios`: SwiftUI UI and native AVQueuePlayer; links the Rust core as a static library.
- `hopper-node`: optional headless service on always-on desktops/Linux boxes.

## Identity and pairing
Each installation persists an Iroh key. One-time, expiring invitations authorize a second key.
The desktop QR contains the same invitation accepted by the text field. The phone decodes it
on device and redeems it over the existing encrypted pairing connection.
Pair devices explicitly; this first release does not silently extend trust to friends of peers.
Pair the phone with each desktop, and pair desktops whose libraries should be visible together.
All peer requests except invitation redemption require a paired connection identity.
Revocation removes local trust and cached catalogue; it cannot erase files already downloaded.

## Media and state
Media IDs are BLAKE3 hashes of original file bytes; identical imports merge as one library item.
Local filesystem paths never cross the mesh. Logical folder paths are metadata only.
Metadata may be read with ffprobe on desktop; absence must not prevent a supported import.
Imported paths can be referenced in place; uploads and offline copies are managed by Hopper.
Source files must remain present and unchanged to be served. Desktop shared folders persist
as local registrations. A minute scan reconciles files by content ID, using size and modification
time to reuse unchanged metadata. Each shared source is checked against its canonical path.
Paired nodes exchange current endpoint addresses during catalogue refresh and try a stable identity-derived UDP port at known LAN addresses after restart. A peer gets logical folder names and media IDs. Stopping a share removes that source, and
independent imports, other desktops’ sources and downloaded copies keep their own lifetimes.

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

## Podcasts and processing
A show has one desktop owner. Peers forward typed subscribe/settings/download/cut/cancel commands
only to explicitly paired owners. The owner fetches RSS and enclosure bytes, preserves GUID identity,
archives source metadata, and serially processes durable jobs. Feed refresh is periodic while running.

Local-only agent configuration chooses actual audio LLM inference, Hermes/custom argv or an MCP
inbox. Credentials and executable configuration never enter the mesh. Providers return source-bound
segment decisions; the core validates them, computes a retained timeline, renders a new managed
file, and maps chapters, supported transcripts, soundbites and progress through the same spans.
Failed/cancelled jobs cannot replace a ready episode. Original bytes remain available. iOS embeds
the same catalogue/command core but launches neither agents nor FFmpeg.

## Home and durable intake
Home is a per-device preference, defaulting to self on desktop and unset on iOS. It must identify
self or an explicitly paired desktop whose cached catalogue advertises desktop capability. A
Home can be offline. New subscriptions and imports target it; old show owners and pending targets
never silently change. These defaults preserve the distributed library model.

Apple links resolve to public RSS on the receiving desktop. OPML is bounded and traversed locally;
each feed gets an outbox job. File imports stage managed bytes before enqueue. Typed Deliver
requests are authenticated like other Iroh operations; Home only fetches media IDs present in the
sender's catalogue and verifies/copies before acknowledging. Source files are untouched. Sender
restarts recover interrupted jobs; transport errors back off and explicit errors require retry.
Receipts key on sender/job ID. The operation layer is idempotent across acknowledgement loss.
No new service or public endpoint is exposed. Catalogue snapshots exclude outbox/settings.

The iOS share extension only writes an App Group inbox. Main-app startup/foreground drains it into
the core outbox, using stable batch IDs and removing manifests only after enqueue acknowledgement.
The main app and extension have separate lifetimes; neither this inbox nor Iroh promises background
execution after the phone app is suspended or terminated. See [setup](imports-and-home.md).

## Browsing folders and episode artwork
Clients derive folder levels from the logical paths in the catalogue; a phone does not need the
desktop’s filesystem paths. A folder lists its immediate children. Playing it queues the media
under that branch in natural folder and title order. Shared roots index in place; the desktop’s
native copy action can instead make a managed copy while retaining the same directory hierarchy.

A chapter can carry a publisher image and a hidden table-of-contents flag. Cuts remap chapter
times while keeping these fields. When playing an original from an older catalogue, clients can
request the publisher’s chapter document using the catalogued episode ID. This fallback never
places original timings onto a cleaned recording. Players prefer current chapter art, then episode
and show art; iOS uses the same selection for its system playback controls.

## Scope and validation
The current slice includes RSS subscriptions, pluggable audio processing and timeline remapping.
Full Podcasting 2.0 support, cleaned RSS publishing, transcript/vector search, automatic offline folder
replication and richer codec conversion remain tracked product work. Real inference accuracy
and third-party client integrations require evaluation separately from deterministic contract tests.
Cross-platform source/build workflows do not substitute for actual Windows/Linux/device testing.

One-time desktop folder imports have separate local source memberships, keyed by the referenced root path or the browser-supplied top-level name. Removing a membership changes the local catalogue and keeps the physical files. Shared roots, other imports, offline copies and peer sources can continue providing the same content ID.

## Podcast discovery

Both shells use the same directory adapters and durable subscription intake. Apple requests originate on the browsing device. Podcast Index queries travel over the paired connection to the selected desktop, which signs HTTPS requests using private local credentials. Directory browsing is read-only. Subscribing explicitly queues one URL for that desktop. After the owner imports the feed, it persists a download of the latest published episode. A separate worker completes episode transfers and resumes interruptions independently of agent analysis. Explicit downloads share that queue.

The merged subscription library orders shows by latest episode publication, including cached peer episodes. A single-episode cut takes its own job options and leaves the default policy for future episodes intact.
