# ffi.rs

## Purpose
Small stable C boundary for native Swift embedding; one Rust runtime/node per process.

## Contracts
hopper_start takes NUL-terminated directory/name strings and returns owned JSON Hello or error.
Swift must call it off the UI thread and free the result using hopper_free. Repeat starts return
the same running node. No panic may unwind across the C boundary. Files remain in iOS's sandbox.
