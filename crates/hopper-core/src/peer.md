# peer.rs

Explicit refresh waits behind an ongoing refresh, ensuring an owner command's newly created
media is present when the client next asks to play it.

Authenticated podcast commands are dispatched to the receiving owner and return typed results.
Outgoing connections still serve reverse requests while lengthy downloads/processing are running.

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

Revocation closes the cached connection after removing trust; guard checks are Clippy-clean.

Paired Deliver requests dispatch to desktop Home intake. Only IDs published in the sender catalogue can trigger verified copies; local executable/credential configuration remains inaccessible.

Peer reads resolve an advertised ID to an available local import or registered shared-folder file. Only logical folder names appear in snapshots.

Paired snapshot exchanges carry optional current address hints. Updates must match the authenticated peer and cannot recreate a removed pairing.
