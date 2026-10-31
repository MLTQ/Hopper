# project.yml

XcodeGen is the source of truth for the native iOS target. The app embeds the Rust XCFramework
and links the Apple frameworks required by Iroh. Target iOS 16, with arm64 simulator and device
builds. Companion Markdown documents are development material and excluded from app resources.
Set a development team locally before physical-device installation; no team is assumed.

HopperShare is embedded with the Shared sources compiled into both targets. Enable App Group
`group.app.hopper.player` for the app and extension when configuring physical-device signing.
The extension stores URL/OPML intake; Rust and playback remain exclusively in the main app.
