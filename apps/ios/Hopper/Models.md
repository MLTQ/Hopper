# Models.swift

Library includes an optional PodcastCatalogue, retaining decode compatibility with older cores.

## Purpose
Typed local API contracts and media formatting. JSON uses snake_case through client strategies.

## Contracts
Media IDs are content hashes, sources indicate replicas, and only local+online implies available
offline. Queue is per-device. ProgressBody is also the durable pending-checkpoint representation.

Library now decodes optional Home and delivery status fields.
