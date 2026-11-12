# model.rs

## Purpose
Serializable contracts shared by the local API, Iroh protocol, desktop and iOS clients.

## Contracts
- Media IDs are lowercase BLAKE3 hex; files with identical bytes share an ID.
- LocalMedia is private persistence data; only Media crosses the network.
- Progress orders by (Lamport revision, device ID), never furthest playback position.
- Timestamps, positions, and chapter boundaries use seconds. Queue is local to a device.
- Source availability is advisory; streaming still handles errors and tries other replicas.
- Snapshots default missing podcast catalogues for compatibility. Libraries merge explicitly paired
  show owners and advertise host capabilities; episode IDs differ from immutable media variant hashes.

Local Library adds home and deliveries; ImportReport adds item_ids. These do not change peer snapshot serialization.

Chapter artwork URLs and hidden cues are optional on the wire so old persisted media and peers still decode.
