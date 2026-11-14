# http.rs

## Purpose
Loopback-only shell API and byte-range media gateway used by web/native players.

## Contracts
- /api requires a per-run bearer token and validates Host/Origin. No CORS is enabled.
- /media requires a separate item-scoped grant and supports GET/HEAD and single HTTP ranges.
- Import paths refer to the local desktop. Multipart uploads copy into managed storage.
- UI assets contain no authorization secrets; shells pass the secret via URL fragment.
- APIs return JSON errors; unavailable media returns 503 and invalid ranges return 416.
- Long imports/downloads currently keep the request pending; background job UI is follow-up work.
- The HTTP listener shuts down when its owning node closes.
- /api/podcasts/command forwards a typed action to an explicitly paired owner; its body includes
  owner_id plus the action fields. Podcast UI is served as a local static ES module.

GET/POST /api/podcasts/agent read redacted local status or save desktop configuration. This route
is never forwarded over Iroh. The agents.js module supplies the desktop setup dialog.

POST /home selects a paired desktop; /add-podcasts queues Apple/RSS/OPML imports. /deliveries/:id/retry retries failed/offline additions. File import/upload routes successful item IDs to Home with a managed staging copy.

The loopback UI also serves material.css, glass.js and glass-lens.js through the same asset headers.
These are presentation-only assets; no appearance preferences or backdrop pixels enter the API.

The authenticated folder routes manage local registrations and invoke a picker supplied by the desktop shell. The QR route renders a one-use invitation locally. The pairing and folder UI modules are bundled assets.

GET /api/folders includes imported roots. DELETE /api/imported-folders/:id removes one local import group. Multipart folder names come from the browser’s relative filename.

The native folder-copy endpoint invokes the shell picker. Original chapter lookup is restricted to catalogued episode IDs. A bearer-protected binary endpoint fetches bounded artwork without forwarding credentials.

The authenticated native-play route resolves a media ID to an available local file or scoped stream. Playback-download returns the verified local path.

POST /api/podcasts/directory reads an Apple chart/search or routes Podcast Index to a selected desktop. POST /api/podcasts/directory/keys accepts a new local desktop key pair and returns only an acknowledgment.

The desktop Cutting room module renders persisted job state through existing podcast commands.
