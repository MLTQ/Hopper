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
