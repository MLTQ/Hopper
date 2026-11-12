# wire.rs

## Purpose
Versioned length-prefixed JSON request/response protocol over encrypted Iroh streams.

## Contracts
ALPN is hopper/library/1. JSON frames are bounded to 16 MiB. A Media response is followed by
exactly `length` raw bytes. Requests never contain filesystem paths. Pair is the sole request
allowed before the connection identity has been authorized. Callers bound handshake timeouts.
Podcast commands operate on the authenticated owner's subscriptions/jobs, never local executable
configuration or filesystem paths. Responses return show/media/job IDs; snapshots carry status.

Deliver/Delivered carry typed podcast links or published media IDs and idempotent receipts, never arbitrary paths. A paired sender may add its own Hopper media to a receiving desktop library.

Snapshot requests and responses may include the sender’s current endpoint address. Older peers omit or ignore the optional field.

Directory is a paired, read-only request to run a Podcast Index query on a desktop. The response contains categories and show listings. Credentials never enter the wire protocol.
