# player.js

## Purpose
Persistent desktop audio/video element, device queue, media controls and progress checkpoints.

## Contracts
- Initial state restores selection without autoplay. Explicit play resolves a fresh media grant.
- Progress saves are serialized to preserve seek/rewind intent; snapshots every ten seconds,
  plus pauses, seeks and completion. Queue transitions are driven by the media element.
- System Media Session controls are best-effort where supported by the desktop webview.
- Browser playback supports the codecs installed in that webview. Native iOS uses its own player.
- No library rerender replaces the media element. Every asynchronous load has a generation guard.
- Progress is attached to the loaded media ID, not a pending selection; queue writes are serialized.
- Removing the current item or clearing the queue cancels any outstanding playback load.

A folder queue can start at a selected episode and follows the ordered items in that subtree.
