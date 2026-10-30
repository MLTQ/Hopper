# lib.rs

## Purpose
Shared process-independent node lifecycle and merged library. Tauri and Swift embed this core.

## Components
- Config selects a persistent data directory; local_only disables public discovery for tests.
- Node owns Iroh, SQLite, runtime authorization, and cached availability.
- Hello is a local-shell secret and must never be advertised to peers.
- library merges local and explicitly paired catalogues, attaching known progress.

## Contracts
An OS lock prevents simultaneous instances mutating one library. Identity corruption fails rather
than silently generating a new device. Media originals are not modified. Per-peer refresh runs
every 15 seconds; cached catalogues remain visible offline. Queues are local, progress replicated.
Closing a node cancels the HTTP listener and closes Iroh, releasing server-owned node references.
Node also owns podcast command/download locks and the durable desktop worker. Library/snapshots
include show-owner catalogues and processor capability; remote shows remain authoritative on their host.

Desktop FFmpeg tools resolve through media_tools, including standard macOS Homebrew locations.

Public library/snapshot catalogues strip full editing documents to keep long podcast transcripts
from filling peer frames. Original and cleaned metadata remain in the owner’s durable records.
