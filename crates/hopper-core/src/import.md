# import.rs

## Purpose
Reference local media without modifying it, hash identity, and extract optional metadata/chapters.

## Contracts
- Unsupported extensions are skipped. A container extension does not guarantee client codec support.
- Recursive imports retain logical folder names and do not traverse symlinks.
- Size and nanosecond modification time detect changed references before serving.
- ffprobe is optional on desktops and never spawned on iOS. No media URL is accepted here.
- Adding identical bytes replaces the local reference with the most recently imported location.
- Raw AAC uses audio/aac; MP4 audio containers use audio/mp4.

Desktop FFmpeg tools resolve through media_tools, including standard macOS Homebrew locations.

Import reports include successful item IDs so local API callers can queue verified Home deliveries.

Recursive path imports record the selected root separately from each item’s logical folder. Publishing verified managed bytes reuses an identical existing blob so other source records retain their modification timestamp.

Native folder copies preserve relative directory names and publish content-hashed files from temporary storage. Managed copies and referenced folder traversals share file filtering.
