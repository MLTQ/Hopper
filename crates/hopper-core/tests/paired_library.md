# paired_library.rs

Regression coverage explicitly checks reverse catalogue and media requests over the connection
opened by the invitation recipient, rather than relying on progress pushed by the forward refresh.

## Purpose
Exercise real independent Iroh nodes, authenticated HTTP, seek/range behavior, caching and sync.

## Contracts
Uses local-only endpoint presets and isolated temporary libraries. Tests one-use invitations,
unauthorized origins, remote bytes, HEAD/416, intentional rewind, offline copies after revocation,
deduplication and unchanged source bytes. This does not prove mobile background behavior or WAN
connectivity; those require real devices/networks.
