# HopperPlayer.swift

## Purpose
Native AVQueuePlayer session, remote commands, background queue advancement and durable progress.

## Contracts
- Native owns current item and upcoming three entries; no web UI/timer drives transitions.
- Play is explicit. Restore selects the last item without autoplay. Queue persists immediately.
- Periodic progress, seek/pause and completion write the native journal before HTTP delivery.
- Call/Siri interruptions respect shouldResume; headphone disconnection pauses.
- AVAudioSession playback category plus UIBackgroundModes audio enable legitimate background use.
- Imported metadata supplies chapters; system decoding determines actual format support.
- Physical-device lock/route/network tests are required; compiling is not proof of background play.
- Upcoming-entry loads are serialized and invalidated by queue edits; unavailable catalogue IDs
  are retained safely when editing visible rows. Explicit play flushes the journal and reads fresh
  local progress before seeking, preserving immediate rewind/replay intent.

Folder playback replaces the queue with the selected subtree and can start at a particular episode. Artwork follows playback, seeks and item changes through PlaybackArtwork.
