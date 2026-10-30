# PlaybackJournal.swift

Explicit playback can await delivery and inspect a still-pending checkpoint before restoring position.

## Purpose
Durable native checkpoint outbox independent of screen lifetime and network completion.

## Contracts
Persist locally before sending to the core. Serialize writes, retain failures, and acknowledge
only the exact entry sent so an in-flight response cannot erase a later rewind. Data protection
allows locked-screen access after the first unlock. Latest pending checkpoint per item is retained.
