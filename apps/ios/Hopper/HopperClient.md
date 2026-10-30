# HopperClient.swift

## Purpose
Own native node startup, authenticated API calls and the observable merged library.

## Contracts
Rust starts off-main and remains in process. Loopback authentication stays in native memory.
File imports copy into the app sandbox while security-scoped access is active. Background media
does not depend on this client's polling loop. Refresh failures are visible, never fabricated data.
