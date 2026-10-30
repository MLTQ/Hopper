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
