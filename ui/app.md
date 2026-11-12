# app.js

## Purpose
Desktop library interaction: filtering, folders, uploads, pairing, source availability and details.

## Contracts
- All imported/remote text is escaped. Event delegation uses media IDs, not executable metadata.
- Files selected/dropped are copied; path import explicitly references originals.
- Pairing explains trust and requires a one-use invitation from the other device.
- Device/library refresh never restarts playback. The player owns its local queue.
- Empty libraries are real empty states; no fabricated media or simulated online devices.
- Native file chooser supports recursive folders; dropping folders is not yet recursive.
- Podcast browsing delegates to podcasts.js and keeps the existing playback/queue instance alive.

Devices includes a preferred Home picker and durable delivery statuses. New file imports are
forwarded by the core when Home is remote; referenced originals are copied before delivery.

Appearance delegates to `Glass` in glass.js. It owns only device-local presentation preferences;
library refreshes and dialogs retain the existing player instance.

The Add and Folders screens open the shared-folder manager. The Pair a device dialog uses pairing.js for QR generation, expiry and text invitations.

Folders refreshes the added-folder list on entry and library reload. If a selected logical folder disappears, the collection returns to All folders.
