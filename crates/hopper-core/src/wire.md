# wire.rs

## Purpose
Versioned length-prefixed JSON request/response protocol over encrypted Iroh streams.

## Contracts
ALPN is hopper/library/1. JSON frames are bounded to 16 MiB. A Media response is followed by
exactly `length` raw bytes. Requests never contain filesystem paths. Pair is the sole request
allowed before the connection identity has been authorized. Callers bound handshake timeouts.
