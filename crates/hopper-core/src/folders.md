# folders.rs

A desktop stores each designated root and its index as one SQLite record. A native shell may
supply an asynchronous folder picker. Only authenticated local HTTP calls manage roots.
Periodic and manual scans serialize with add/remove, reuse unchanged hashes, skip symlinks and
Hopper's private data, and publish each complete index atomically. Peer snapshots and media reads
resolve indexed content IDs only. Original paths never enter snapshots. Unsharing removes only
the selected registration; independent copies and other shared roots remain available.

Source lookup includes one-time imported folders. Identical content remains playable from an available independent file, another import or a registered share.

A known shared-folder path can restore the folder label of an existing loose import with identical bytes.

Automatic playback copies supply local items when no permanent source is available.
