# main.rs

## Purpose
Native desktop window around an embedded Hopper node and local media interface.

## Contracts
The same Tauri source targets macOS/Linux/Windows. No remote page receives Tauri commands.
Navigation stays on this node's loopback origin. HTML file input/drop imports managed copies.
HOPPER_DATA_DIR optionally selects an isolated library for development. Closing the app stops
hosting; use hopper-node on an always-on machine for continuous availability.

The shell supplies a native asynchronous folder chooser to the embedded node. A cancellation returns an empty selection.
