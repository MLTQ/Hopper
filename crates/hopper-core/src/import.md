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
