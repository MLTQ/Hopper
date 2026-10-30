# podcast/render.rs

Desktop FFprobe/FFmpeg execution. Inspect actual audio duration; trim and concatenate retained
spans into AAC/M4A. Fades apply only at cut edges, never overlap, and clamp to half a segment.
The duration must match the edit timeline within encoder tolerance before the worker publishes it.
Original source files are never modified. Strip source chapters from the rendered container to
avoid exposing old timestamps; the core serves remapped chapters and retains the JSON document.
Commands are argv-based with private generated files, bounded runtime, no stdin and local-only
demuxer protocols. Requires FFmpeg/FFprobe on the processing desktop PATH. No iOS subprocesses.

Filter graphs use a file to avoid command-line length limits. Capability detection selects the legacy
filter_complex_script option on older FFmpeg or file-valued `-/filter_complex` on current versions.

Desktop FFmpeg tools resolve through media_tools, including standard macOS Homebrew locations.
