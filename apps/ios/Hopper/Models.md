# Models.swift

## Purpose
Typed local API contracts and media formatting. JSON uses snake_case through client strategies.

## Contracts
Media IDs are content hashes, sources indicate replicas, and only local+online implies available
offline. Queue is per-device. ProgressBody is also the durable pending-checkpoint representation.
