# Local API v1

The node binds an ephemeral loopback port and returns `Hello { port, token, device }` to its shell.
All `/api` routes require `Authorization: Bearer <token>` and a loopback Host header. Browser
Origin must match the node origin. JSON uses snake_case. No peer receives this control secret.

| Method | Path | Contract |
| --- | --- | --- |
| GET | /api/library | Device, merged items/sources/progress, paired status, local queue |
| POST | /api/refresh | Exchange catalogue/progress with paired nodes, return library |
| POST | /api/invite | `{code, expires_in}`; one-use, ten-minute invitation |
| POST | /api/invite/qr | Return the invitation, its ten-minute lifetime and a locally rendered SVG data URL |
| GET | /api/folders | Local share registrations in `folders`, one-time import groups in `imports` (id, name, files), and native picker availability |
| POST | /api/folders | Register and index an absolute desktop folder supplied as `{path}` |
| POST | /api/folders/choose | Ask the desktop shell to pick and share a folder. Cancellation returns null |
| POST | /api/folders/:id/rescan | Reconcile a registered folder immediately |
| DELETE | /api/folders/:id | Stop sharing this folder and keep the original files |
| DELETE | /api/imported-folders/:id | Remove a one-time local folder import and preserve files on disk |
| POST | /api/pair | `{code}`; persist mutual trust and fetch peer library |
| DELETE | /api/peers/:id | Remove local trust and peer catalogue; retain downloaded copies |
| POST | /api/import/folder | Pick and copy a directory tree with the desktop’s native chooser, or supply a local path. Returns an import report or null on cancellation. |
| POST | /api/podcasts/artwork | Fetch a bounded HTTP(S) image document for the authenticated native shell. The publisher receives no Hopper credentials. |
| GET | /api/podcasts/chapters/:id | Fetch and cache an original episode’s publisher chapter timeline by catalogued episode ID. Cleaned variants use their remapped media chapters. |
| POST | /api/import | `{paths:[...]}` local paths; return `{added,skipped,errors}` |
| POST | /api/upload | Multipart files; copy into managed storage; same report |
| POST | /api/play/:id | `{path}` relative media URL with scoped grant |
| POST | /api/keep/:id | Stream, verify hash, publish local copy; `{ok:true}` |
| POST | /api/progress | `{item_id,position,duration?,completed?}`; return versioned progress |
| POST | /api/podcasts/command | `{owner_id,action,...}`; paired owner-directed show/job commands |
| GET | /api/podcasts/agent | Local processing mode, endpoint/model/argv, key presence; never the key |
| POST | /api/podcasts/agent | Local-only `{mode,endpoint,model,command,api_key?}`; no peer forwarding |
| POST | /api/queue | `{items:[media IDs],current:ID or null}`; persist per-device queue |

`GET/HEAD /media/:id?grant=…` accepts one item-specific playback grant, never the control secret.
Byte-range requests support open, closed and suffix ranges, with proper 206/416/Content-Range.
Media is streamed, not loaded wholesale into memory. A missing remote source returns 503.

Peer protocol is separate (`hopper/library/1` over Iroh): paired identities exchange bounded
snapshots/progress and media ranges. One-time invitation redemption is the only unpaired request.
Local paths, import/delete controls and shell secrets are never exposed to remote peers.

## Progress semantics
Seconds are floating point and must be finite/nonnegative. Each local update gets a new Lamport
revision. Incoming newer `(revision,device_id)` values win; equal/replayed values do nothing.
After synchronization, intentional rewinds win over older further-ahead positions. Simultaneous
offline edits resolve deterministically and cannot infer the user's most recent real-world intent.

## Current limits
Queue is per device; progress is synchronized every 15 seconds and on explicit refresh. Imports
and downloads hold their request open. Download retry restarts the transfer. Catalogue snapshots
are capped by a 16 MiB protocol frame. Playback grants live for seven days or until process exit.

## Podcast contracts
Library/snapshot `podcasts` includes shows, episodes, jobs and desktop processing availability.
Commands: subscribe(url), refresh(show_id), settings(show_id,newest_first,cut_options),
unsubscribe(show_id), download(episode_id), cut(show_id,episode_ids,options), retry(job_id), cancel(job_id).
Results include optional show_id/media_id and job_ids. A Cut request enqueues durable work and returns;
its statuses progress through waiting_for_agent/queued/downloading/analysing/rendering/ready, or
failed/cancelled. Options: keep_banter, keep_promos, fade_ms, minimum_confidence, auto_process.

Agent modes are builtin, hermes, command, inbox. Configuration requires the local control token;
paired clients cannot change executables or credentials. The source-bound Analysis and MCP bridge
contracts are documented in [agents](agents.md). Backwards-compatible snapshots default podcasts to
empty; older peers cannot forward the new podcast commands until upgraded.

Player/peer catalogues carry job statuses, not full decision/transcript documents. Those remain on
the owner in job manifests. Playable library media retains its original/remapped chapter list.
