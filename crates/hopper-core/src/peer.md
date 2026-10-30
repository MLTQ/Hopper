# peer.rs

Both incoming and outgoing QUIC connections run a request-serving loop. The peer that accepted
an invitation can request catalogues/media back over the same cached connection. The integration
test verifies both directions independently; progress push alone is not evidence of reverse access.

## Purpose
Explicit pair trust, cached catalogue/progress exchange, and streaming over Iroh QUIC.

## Contracts
- Invitations expire in ten minutes, are one-use and become invalid after restart.
- Peer identity is the authenticated connection key. Supplied identity must match it.
- A paired peer can read the advertised library and exchange progress, not import/delete files.
- File paths stay local. Every media request rechecks authorization and validates its range.
- Connections/streams/frames and timeouts are bounded. Snapshot refresh runs concurrently across
  up to eight peers. Offline catalogues survive refresh failures.
- Pair each intended device pair explicitly. Removing a peer cannot erase its downloaded copies.
