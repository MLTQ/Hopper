# Validation record

This file records observed checks, not target-platform promises. Updated November 4, 2026.

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

## Podcast and agent checks — October 30, 2026
- Eight Rust tests passed: five unit tests plus paired-library, full podcast render, and agent-mode
  integration tests. Real FFmpeg rendered generated WAV audio; a fake LLM verified actual audio
  input and a fixture command verified the external executable protocol. No real model was called.
- Tested GUID deduplication, remote subscribe/download/cut/unsubscribe, byte-exact original retention,
  edited duration, chapter JSON/art fields, VTT cues, soundbites, mapped listening progress and
  paired access to cleaned media. Conflicting/uncertain overlapping classifications preserve content.
- Built-in mode waits for configuration, sends WAV/model/auth as specified, redacts credential status,
  drops saved credentials on endpoint change, and rejects invalid ranges without replacing a ready
  episode. The actual Python MCP bridge rejects traversal/wrong hashes, accepts valid inbox output,
  and cancellation remains terminal.
- Clippy with warnings denied, JavaScript syntax checks, and diff whitespace checks passed.
- macOS Tauri development bundle rebuilt. Rust iPhone/device and simulator libraries rebuilt;
  native SwiftUI simulator build passed with the podcast tab, show menus and cut preferences.
- Desktop UI walkthrough against an isolated synthetic feed: subscribe, show settings, audio-agent
  setup, fade selection, cut submission, ready status, queue, cleaned playback, original access and
  remapped chapter display. A 35-second source became 30 seconds; a chapter at 10 seconds moved to 5.
  Evidence: artifacts/podcast-ready.png (local, ignored).
- Updated native simulator visual walkthrough was blocked by the Mac being locked. Compilation is
  verified; no new native UI or physical-device execution result is claimed for this feature.

Real ad-recognition precision, long-episode costs, Hermes audio tools and Dot/Muse client integration
remain unvalidated. Cleaned RSS export and full Podcasting 2.0 compliance remain follow-up work.
CI now installs FFmpeg/Python for these tests, but no remote CI run has been performed.

## Imports and Home checks — October 31, 2026
- Eleven Rust tests passed: six unit tests and five integration tests. New fixtures exercise three
  independent Iroh nodes, OPML deduplication, Home ownership, receipt replay, managed file delivery,
  unchanged/moved originals, playback-copy availability after laptop shutdown, and persistent
  pending additions after sender restart while Home is offline. Import-routing changes were retested.
- Live public Apple lookup checked against Apple Events: the parent-show response includes feedUrl,
  episodeGuid and episodeUrl. Direct episode-ID lookup returned no results; the implementation now
  queries the parent show with entity=podcastEpisode, bounds results to 200 and verifies both IDs.
  Parser/matching unit tests were rerun after this correction. No actual podcast audio was fetched.
- Clippy with warnings denied and desktop JavaScript syntax/whitespace checks passed.
- macOS Tauri app bundle rebuilt. Rust arm64 iPhone/device and simulator libraries compiled;
  the SwiftUI app and embedded HopperShare extension compiled for the simulator.
- This feature has not been walked through in the native UI or shared into from the actual Apple
  Podcasts/Castamatic apps. Physical App Group signing, real OPML exports, share-extension lifecycle,
  mobile suspension, WAN delivery and Linux/Windows runtime behavior remain unverified. The
  generated OPML fixture is not represented as a real Castamatic export. See hopper-lky.13.

## Liquid glass appearance — November 4, 2026
- Four desktop material tests passed: ink hysteresis, trimmed outliers, texture/chroma response,
  secondary text at 4.5:1 against synthetic black/white/saturated extremes including highlights,
  tint scaling and dense fallback. This is a numerical material check, not a measured native render.
- Isolated headless Chromium checks passed under the production CSP: decoded card lenses,
  shared podcast ink, unchanged light/dark geometry, saved independent tint preferences, dialog
  lens exclusion, error color, compact layout, rerender/filter cleanup and no browser exceptions.
- Reduced transparency, increased contrast and forced colors were emulated and verified to use
  solid surfaces without SVG lenses. A non-Chromium user-agent guard retained CSS glass; this
  checks the guard in Chromium, not Firefox/WebKit's actual rendering engine.
- A regression check suspends animation frames before startup and verifies the initial sidebar,
  queue and player material still initializes synchronously for background native windows.
- Light/dark library, podcast and import/appearance dialog screenshots were visually reviewed.
  Synthetic catalogue evidence is local and ignored: `artifacts/glass/`. No user media was imported.
- All 11 Rust core unit/integration tests passed. Desktop check, Clippy with warnings denied,
  JavaScript syntax, Rust formatting and whitespace checks passed. macOS Tauri debug app bundle
  rebuilt; the native iOS simulator app and share extension build passed with the new palette.
- Native macOS inspection exposed deferred animation-frame initialization and prompted the
  synchronous startup fix. Further app UI automation timed out; full native interaction and the
  corrected WebKit rendering are not claimed as verified. No iOS simulator walkthrough or physical
  iPhone, Linux, Windows or Firefox runtime check was performed for this appearance change.

See [material scope and changed files](liquid-glass.md). Native material contrast/performance and
accessibility walkthroughs are tracked in hopper-tqd; existing platform/device release gates remain.
