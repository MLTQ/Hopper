# Validation record

This file records observed checks, not target-platform promises. Updated November 12, 2026.

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
- Windows runtime checks and Linux import, playback, codec and cross-device pairing checks.
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

## QR pairing and shared folders — November 12, 2026

- Installed and launched a signed debug build on a connected iPhone 15 Pro Max. On macOS,
  Hopper's native folder picker opened the temporary archive and its library showed the generated
  17-second spoken AAC episode. The desktop displayed a one-use QR. The user confirmed playback
  on the iPhone, the Mac displayed the paired phone online, and the phone's completed playback
  checkpoint reached the desktop at 17.14 seconds. The sample was hosted by reference.
- Two independent desktop nodes paired with one player in the integration suite. Tests cover
  nested logical folders, authenticated QR issuance, replay rejection, seeking with verified range
  bytes, indexing new files, unsharing without deleting originals, and playback from a second
  source. A second test covers restart and temporary archive disconnection.
- The macOS app bundle, arm64 iOS core builds, and a simulator build including the share
  extension passed. All 13 Rust tests passed across the suite and targeted rerun. Clippy with
  warnings denied, formatting, desktop JavaScript syntax and diff whitespace checks passed.
- The connected iPhone's development profiles lacked the App Group used by podcast share-sheet
  imports. The installed pairing build used a local signing override. The project retains both
  App Group entitlements. Provisioning and share-sheet testing are tracked in hopper-lky.13.
- This was a foreground listening check on one physical Mac and iPhone. Multiple physical
  desktops, Linux and Windows runtime behavior, long background playback, and network handoff
  still need device checks.

## Omarchy Linux desktop installation — November 12, 2026

- Built the x86-64 desktop release on Omarchy 4.5 with Rust 1.99.1, GTK 3.24.52 and
  WebKitGTK 2.54.1 using `cargo build --release --locked -p hopper-desktop`. All shared
  libraries resolved. Installed the executable, desktop entry and icon for the current user.
- The first launcher test exited with Wayland protocol error 71. With
  `WEBKIT_DISABLE_DMABUF_RENDERER=1` scoped to Hopper, the compositor mapped the native
  window and its WebKit accessibility tree showed the one-use QR image in the pairing dialog.
  The installed launcher applies this setting on this machine.
- Pair redemption, imports, media playback and codec coverage on Linux remain open in
  `hopper-lky.6`, along with Windows validation and distributable installers.

## Desktop folder removal — November 12, 2026

- All 15 Rust core tests passed on Linux. Regressions cover upgrading older reference and copied folder imports, pending Home deliveries across restart, authenticated removal, overlapping shares and independent reference roots with identical names and media.
- Headless Chromium drove the Folders view against a real, isolated Hopper node. An empty share, a nested reference import and two folder uploads appeared with their counts. Cancel preserved the entry. After removing overlapping entries, the remaining copy served the original bytes. Removing the final entry persisted across reload and preserved the source file.
- The existing Chromium UI checks, JavaScript material tests, Rust formatting and core Clippy checks passed. Other desktop operating systems and iOS were not exercised for this change.
- Installed the rebuilt release through the existing per-user launcher. Opened Folders in native WebKitGTK and verified the Remove action on the upgraded library. All removal tests used temporary media.

## Archive browsing and podcast presentation — November 12, 2026

- The desktop browser fixture contains 4,800 episodes in 20 plotlines, each with two 120-episode
  seasons. It checks drill-down, numeric ordering, paging, and queueing only that season.
- After combining the folder manager and browser, all 16 Rust checks passed and the macOS desktop
  app bundled successfully.
- In a temporary node, the publisher’s No Agenda episode “Spillover” returned its show cover and
  17 timed chapters with images. Verified valid bytes for the cover and the first chapter image,
  and repeated the chapter request from cache. The paired podcast fixture also verifies that
  images and hidden markers survive ad-cut time remapping.
- Installed the signed update on Iris (iPhone 15 Pro Max); launch was deferred by the lock screen.
  The archive itself is on a different desktop. Its actual import, on-phone chapter transitions,
  and listening through that archive were not observed in this check.
- On the Omarchy archive desktop, built revision `8a3a8ee` in release mode and updated the
  executable used by the existing Wayland launcher. Verified that the mapped WebKitGTK window
  used the new executable and served the updated folder browser. The existing catalogue contained
  563 archive items in two subfolders (488 and 75), matching the two directories on disk.
  Opened the archive and its 75-item branch in the native UI and verified that it showed episode
  playback buttons. No playback or phone streaming was started during this deployment check.

## Phone playback and automatic downloads — November 12, 2026

- The paired iPhone’s saved LAN port differed from the live Omarchy endpoint after the desktop
  restarted. The updated desktop and phone reused their existing pairing for the listening check.
- A signed diagnostic build on the physical iPhone started a 6.4 MB archive episode through the
  native player, advanced beyond two seconds, and paused. The library reported a verified automatic
  copy. A second play resolved to a local file URL and advanced beyond three seconds.
- All 18 Rust tests passed. Paired-node coverage exercised an obsolete LAN port, concurrent requests,
  restart, completion, replay, the 30-day boundary and preservation of an explicit keep and the
  desktop original. The interrupted-transfer recovery check also passed. Workspace Clippy and
  both Apple core builds passed. A macOS bundle and a signed regular iPhone app were built.
- The handset check covered foreground playback. The long background and network handoff matrix
  remains tracked separately. The expiry boundary was exercised with a controlled clock.

## Podcast discovery and single-episode cuts — November 12, 2026

- An isolated Hopper node fetched 25 live US Fiction chart entries from Apple, 40 results for “actual play”, and 36 results with Fiction selected. Every returned listing had a cover URL.
- The desktop browser test switched between Apple and Podcast Index, submitted a search with a category and queued the chosen feed for its desktop using fixture responses. Verified Podcast Index’s documented response shapes, signature construction, and missing-credentials state. An authenticated live Podcast Index account was unavailable.
- Core tests check merged show recency with undated and tied shows. A paired client requested a single-episode cut with its own fade setting. The owner rendered with FFmpeg, retained the original and show defaults, and remapped metadata and playback position. Classification used a fixture agent.
- Built the macOS app and both iOS core targets. The signed iPhone app was installed on Iris with the existing development signing configuration. Physical interaction with the discovery screen and Windows builds were not checked.

## Hermes on the Omarchy desktop — November 12, 2026

- Installed the current desktop build and verified its running window. Configured the installed Hermes CLI as the owner’s external agent. There were zero existing jobs queued when it was enabled.
- An isolated Hopper library on the same machine subscribed to a temporary 20.184-second spoken episode with a fictional host-read sponsor between two stretches of roleplay. Hopper downloaded the source, launched the actual Hermes agent with the shipped skill and rendered a ready edited file. Hermes returned original-time decisions with spoken evidence for all three stretches. Its ad spanned 4.36–15.37 seconds. Hopper removed 11.01 seconds. FFprobe measured the copied-back AAC edit at 9.17375 seconds and FFmpeg decoded it successfully. The full source and its separate edited variant remained available.
- The stdio bridge integration test returned an actual WAV clip ending at the source boundary and rejected an oversized request. Skill validation, all 23 core tests, desktop browser checks, workspace Clippy and the macOS bundle build passed. Timing and detection on full-length unscripted shows remain a separate evaluation.

## Subscription downloads and Cutting room — November 13, 2026

A paired listener subscribed to a feed that listed its older episode first. The owner queued only the latest publication, resumed its transfer after an interrupted restart, and completed a manual download while an external-agent job was waiting. Duplicate requests retained a single download record and reused available media. Owner download state reached the paired catalogue.

The native and desktop interfaces expose owner download actions and a Cutting room containing actual processing status, cancellation and retry. iOS decodes download state optionally to retain compatibility with earlier desktop snapshots.

All 24 core tests, the desktop browser interaction checks, workspace Clippy, both embedded iOS targets and the signed iPhone build passed. The browser check clicked Download, verified its owner-directed queue command, then opened Cutting room and cancelled a waiting fixture job. Windows execution was not checked.
