# PlayerView.swift

## Purpose
Queue editing, persistent mini player, full transport, video and chapter presentation.

## Contracts
Views delegate all operations to HopperPlayer. Dismissing a view does not destroy playback.
Slider commits a seek only on release. Queue supports explicit reorder/delete. Codec errors
surface through the client. User-selected video shares the same native player as audio.

The mini-player and speed control use the shared hopperGlass modifier without changing size
or radius. Queue/full-player backgrounds use hopperScene. Primary transport keeps a contrast-safe
accent fill. Playback, scrubbing and queue contracts are unchanged.
