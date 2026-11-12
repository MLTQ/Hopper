# playback_cache.rs

A native playback download verifies the full media hash and length before publishing an offline
copy under the node’s private playback-cache directory. Downloads of the same ID share a transfer
lock. Unrelated items can transfer independently. Records store relative filenames for iOS
container relocation and the progress revision when the download began.

Cleanup expires a copy 30 days after saving or when a newer checkpoint completes the item.
Completion racing a download is checked before publication. A replay of an already finished
item gets a new retention period. Startup removes interrupted transfers in the private cache.
Explicit keeps and original files have independent storage and survive cleanup.

A real paired-node regression exercises duplicate requests, restart, replay, pause, the expiry
boundary, and preserving explicit keeps and source bytes.
