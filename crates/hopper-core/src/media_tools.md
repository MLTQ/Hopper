# media_tools.rs

Resolve FFmpeg/FFprobe using the inherited PATH, plus standard Homebrew locations on macOS.
This keeps Finder-launched desktop apps functional without changing global process environment.
Never downloads or installs executables. Missing programs retain the ordinary process error.
Desktop metadata inspection, audio chunking and rendering share this resolver. Not compiled on iOS.
