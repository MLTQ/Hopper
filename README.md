# Hopper

Put something in the hopper. Listen wherever you are.

Hopper is a standalone paired media library: **macOS, Linux and Windows desktops can each add
and play media; a native iOS app connects directly to those desktops**. There is no mandatory
central library server. An optional headless node suits an always-on Linux box.

This is an early implementation, not a finished release. See [validation](docs/validation.md)
for the checks actually performed and remaining device tests.

The local macOS development bundle is `target/debug/bundle/macos/Hopper.app`. The generated iOS
project is `apps/ios/Hopper.xcodeproj`. The macOS bundle and a physical iPhone have paired by QR and streamed a shared
episode end to end. On Omarchy Linux, the installed desktop app opened its library UI and displayed
a pairing QR code. Linux media playback and Windows runtime checks remain open.

## Working vertical slice

- Share a desktop folder in place, or import individual files and managed copies.
- Scan a desktop’s one-use QR code with Hopper on iPhone to pair over encrypted Iroh connections.
- Browse a combined catalogue with source-device status and cached offline metadata.
- Stream with HTTP byte-range seeking; download a verified copy for offline playback.
- Play audio/video on desktop and through AVQueuePlayer on iOS, with per-device queues.
- Sync playback progress, including intentional rewinds; persist native iOS checkpoints.
- Extract titles, creators, duration and M4B chapters when desktop ffprobe is available.
- Import Apple Podcasts links, RSS feeds, or Castamatic OPML subscription lists.
- Choose a Home desktop; laptop/phone additions wait for delivery there, with visible receipts.
- Browse/control desktop-owned podcasts from paired desktop/iOS apps.
- Set per-show ad cuts, banter/promos, fades, confidence and automatic preparation of future episodes.
- Process through a built-in audio LLM connection, Hermes/custom command, or external MCP inbox.
- Preserve originals and remap chapters, supported transcripts, soundbites and listening position.

See [imports and Home setup](docs/imports-and-home.md) for Apple/Castamatic migration and the iOS
share extension. Spotify is deferred. Subscription imports do not include listening history.

Configure **Podcasts → Audio agent** on the processing desktop before starting cuts. Audio processing
requires FFmpeg and FFprobe. See [audio-agent setup](docs/agents.md) for provider requirements,
external-agent integration and current limits. No paid model or API key is selected automatically.

Identical file bytes merge into one library item with multiple available sources. Your source
files are preserved. Shared folders retain their names and subfolders on paired devices. Hopper
checks them when the desktop starts and every minute while it runs. Open **Folders → Added folders** to see this desktop’s shared folders and one-time
folder imports. Choose **Remove** on a row and confirm. Removing a shared folder stops its scans.
Removing an import drops its items from that folder in the library. Other sources and files on
disk are preserved.

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

On Arch-based Linux with GTK 3 and WebKitGTK 4.1 installed, build the desktop executable with
`cargo build --release --locked -p hopper-desktop`. If Hopper exits at startup with
`Error 71 (Protocol error) dispatching to Wayland display`, try
`WEBKIT_DISABLE_DMABUF_RENDERER=1 ./target/release/hopper-desktop`. This worked on Omarchy 4.5
with WebKitGTK 2.54.1. The setting can be applied in Hopper's launcher.

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

1. On the desktop, open **Put something in → Share a desktop folder → Choose folder…** and pick your podcast archive.
2. Open **Pair a device → Create pairing QR code**. Keep the code on screen.
3. On iPhone, open **Hopper → Devices → Scan pairing QR code**, allow the camera, and scan the desktop. You can also paste the invitation in Devices.
4. In the iPhone **Library**, open the archive, then its plotline or season. **Play folder** starts that branch. Tapping an episode starts there and continues within the branch. On desktop, open **Folders** for the same hierarchy.
5. Repeat the QR scan for each additional desktop. All paired libraries appear together. Use **Keep on this device** on an episode for offline listening.

Original-owning desktops must stay online for streaming. Downloaded copies work offline and are
also available from that paired device while it is serving. Closing a desktop app stops its node.
Pairing grants that device library access; unpairing cannot delete copies already downloaded.

## Formats and next steps

Import recognizes MP3, M4A/M4B, AAC, FLAC, Ogg/Opus, WAV, AIFF, MP4/M4V, MOV, WebM and MKV.
Actual playback depends on the native player/system webview's codec support. Broad format
conversion, embedded artwork, resumable downloads and automatic folder replication are follow-ups.

Remaining product work includes publishing private cleaned RSS feeds, full Podcasting 2.0 support,
broader embedded-art support, semantic transcript search, and evaluating real-world ad-detection accuracy.
See [the roadmap](docs/roadmap.md) and [agent limitations](docs/agents.md).

```sh
cargo test -p hopper-core
cargo check -p hopper-desktop
```

See [architecture](docs/architecture.md) and [local API](docs/api.md). Each source module has a
companion Markdown contract. Beads (`bd ready --no-daemon`) tracks ongoing implementation.
