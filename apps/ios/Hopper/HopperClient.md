# HopperClient.swift

Owner podcast commands may prepare a full episode download, so their timeout matches long media
transfers. PodcastModels extends this client with typed commands and post-command refresh.

## Purpose
Own native node startup, authenticated API calls and the observable merged library.

## Contracts
Rust starts off-main and remains in process. Loopback authentication stays in native memory.
File imports copy into the app sandbox while security-scoped access is active. Background media
does not depend on this client's polling loop. Refresh failures are visible, never fabricated data.

Starts share-inbox draining after core startup; serialized drain state prevents duplicate concurrent intake.

Index media, shows and media-to-episode associations when a catalogue arrives. The player uses these indexes for repeated queue and artwork lookups.
