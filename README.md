# Hopper

Put something in the hopper. Listen wherever you are.

Hopper is a standalone paired media library: **macOS, Linux and Windows desktops can each add
and play media; a native iOS app connects directly to those desktops**. There is no mandatory
central library server. An optional headless node suits an always-on Linux box.

This is an early implementation, not a finished release. See [validation](docs/validation.md)
for the checks actually performed and remaining device tests.

The local macOS development bundle is `target/debug/bundle/macos/Hopper.app`. The generated iOS
project is `apps/ios/Hopper.xcodeproj`. Both have been built; macOS and simulator launch/playback
checks are recorded below. Linux/Windows runtime verification remains outstanding.

## Working vertical slice

- Import media files or whole folders on any desktop, by reference or managed copy.
- Pair installations using expiring, one-use invitations over encrypted Iroh connections.
- Browse a combined catalogue with source-device status and cached offline metadata.
- Stream with HTTP byte-range seeking; download a verified copy for offline playback.
- Play audio/video on desktop and through AVQueuePlayer on iOS, with per-device queues.
- Sync playback progress, including intentional rewinds; persist native iOS checkpoints.
- Extract titles, creators, duration and M4B chapters when desktop ffprobe is available.
- Subscribe to RSS podcasts on a desktop and browse/control them from paired desktop/iOS apps.
- Set per-show ad cuts, banter/promos, fades, confidence and automatic preparation of future episodes.
- Process through a built-in audio LLM connection, Hermes/custom command, or external MCP inbox.
- Preserve originals and remap chapters, supported transcripts, soundbites and listening position.

Configure **Podcasts → Audio agent** on the processing desktop before starting cuts. Audio processing
requires FFmpeg and FFprobe. See [audio-agent setup](docs/agents.md) for provider requirements,
external-agent integration and current limits. No paid model or API key is selected automatically.

Identical file bytes merge into one library item with multiple available sources. Your source
files are preserved. Folder paths are retained as library metadata. New files in imported folders
are picked up by reimporting; automatic watching/mirroring is not implemented yet.

## Run a desktop

Prerequisites: current Rust toolchain, the native Tauri build dependencies for your OS, and
optionally FFmpeg (`ffprobe`) on PATH for rich metadata. There is no Node/npm build step.

```sh
cargo run -p hopper-desktop
```

To make a platform bundle, install Tauri CLI 2 (`cargo install tauri-cli --version '^2' --locked`),
then run `cargo tauri build` inside `apps/desktop`. macOS debug development builds can use
`cargo tauri build --debug --bundles app`. Signed public installers are not configured.

On Ubuntu/Debian, install Tauri's GTK/WebKit dependencies (including `libwebkit2gtk-4.1-dev`,
`build-essential`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `patchelf`) before building.
Windows needs the MSVC C++ build tools and WebView2. macOS needs Xcode command-line tools.

## Run a headless desktop/node

```sh
cargo run -p hopper-core --bin hopper-node -- --name "Living room Linux" --import /path/to/media
```

The command prints a **private local UI URL**. Open it on that machine. Use Pair a device in the
UI to connect other installations; do not share the local UI URL or its authorization fragment.
`--data-dir PATH` selects a library; `--local-only` disables public discovery/relays for testing.
Desktop shell builds accept `HOPPER_DATA_DIR` for an isolated development library.

## Build iOS

Requires macOS/Xcode, XcodeGen (`brew install xcodegen`), and Rust's Apple device/simulator targets.

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
cargo fetch --locked
bash apps/ios/build-core.sh
open apps/ios/Hopper.xcodeproj
```

Choose your development team in Xcode before running on your iPhone. To keep it across project
regeneration, set `DEVELOPMENT_TEAM` in `apps/ios/project.yml`. The project is generated; edit
that YAML rather than committing hand-edited `.xcodeproj` files. No signing identity is assumed.

The phone embeds the same Rust node and uses a native SwiftUI interface. AVQueuePlayer owns
playback and upcoming items. Background audio is declared; lock-screen commands and interruption
handling are implemented. Physical-device behavior still needs validation, especially Wi-Fi to
cellular transitions, long pauses, and extended locked playback.

## Pair and listen

1. Open Hopper on a desktop. Add a few files/folders.
2. Select **Pair a device → Create invitation**.
3. On iPhone, open **Devices**, paste the invitation, and pair.
4. Repeat with the other desktops. Pair desktops to each other if they should browse one another.
5. Play from the combined library, add items to Up next, or choose **Keep on this device**.

Original-owning desktops must stay online for streaming. Downloaded copies work offline and are
also available from that paired device while it is serving. Closing a desktop app stops its node.
Pairing grants that device library access; unpairing cannot delete copies already downloaded.

## Formats and next steps

Import recognizes MP3, M4A/M4B, AAC, FLAC, Ogg/Opus, WAV, AIFF, MP4/M4V, MOV, WebM and MKV.
Actual playback depends on the native player/system webview's codec support. Broad format
conversion, embedded artwork, resumable downloads and automatic folder replication are follow-ups.

Remaining product work includes publishing private cleaned RSS feeds, full Podcasting 2.0 support,
chapter-art display, semantic transcript search, and evaluating real-world ad-detection accuracy.
See [the roadmap](docs/roadmap.md) and [agent limitations](docs/agents.md).

```sh
cargo test -p hopper-core
cargo check -p hopper-desktop
```

See [architecture](docs/architecture.md) and [local API](docs/api.md). Each source module has a
companion Markdown contract. Beads (`bd ready --no-daemon`) tracks ongoing implementation.
