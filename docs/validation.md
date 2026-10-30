# Validation record

This file records observed checks, not target-platform promises. Updated October 30, 2026.

## Automated checks already passed
- Rust core unit tests: byte-range edge cases and Lamport progress convergence/rewind semantics.
- Real local Iroh node integration: one-use pairing, remote catalogue, range stream bytes,
  HEAD/416, denied HTTP authorization/origin, progress sync, verified download, playback from a
  local copy after peer revocation, deduplication, and preservation of original source bytes.
- Reverse catalogue and media requests on the same paired connection, independent of forward
  progress pushes. This caught and fixed missing request handling on outbound connections.
- Desktop JavaScript syntax checks.
- macOS Tauri executable and app bundle builds; native app launches and loads its local library.
- Rust arm64 iPhone/device and simulator static libraries compile with iOS 16 deployment targets.
- Native SwiftUI simulator app builds, installs and launches (iOS 26.5 simulator).

## Interactive checks already passed
- Desktop folder import using generated WAV, MP3 and chaptered M4B files; real metadata and chapters.
- Desktop playback advances through all three queued files. M4B chapters are available for seeking.
- Simulator pairs with an isolated desktop node, receives all three files and existing progress,
  streams through the native player and advances automatically to subsequent queue entries.
- Simulator downloads the M4B, plays the local copy, resumes its saved position and seeks to
  an embedded chapter. Both devices report online after the two-way connection fix.
- App relaunch restores the queue, current audiobook and saved progress without autoplay.
- Light appearance was checked with the simulator itself in dark mode and corrected for contrast.

Tests use generated audio in an isolated preview library; no personal media was imported.
The checked-in build workflow has not been run on a remote CI service.

## Required before a dependable release
- Actual Windows and Linux runtime behavior and codec coverage.
- Physical iPhone: at least 30 minutes locked with three queue transitions; lock-screen/headset
  controls; long M4B seeks; calls/Siri; Bluetooth disconnection; Wi-Fi/cellular handoff; host sleep;
  offline playback; paused-app resumption; relaunch restoring a saved position.
- WAN pairing on different networks, relay fallback, and reconnect after mobile suspension.
- Power/battery measurements and larger-library/import/download stress cases.

No physical-iPhone background-playback result is claimed by compilation or simulator checks.
