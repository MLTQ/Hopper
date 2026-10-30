# store.rs

## Purpose
Transactional SQLite persistence behind a mutex; the same schema runs on desktop and iOS.

## Components
- Typed records persist local media, trusted peers, cached snapshots, and device queue.
- progress increments a durable Lamport clock in the same transaction as a local update.
- merge_progress atomically advances the clock and applies deterministic newer records.

## Contracts
Only validated finite nonnegative progress enters storage. Rewinds are intentional updates.
Tests cover convergence and replay of stale records. Private file paths remain in local records.
