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
Generic update holds the connection mutex across read/validation/write so a processing-state
transition cannot resurrect a job cancelled concurrently.

Regression tests borrow single progress records directly; no extra clones are needed for merging.
