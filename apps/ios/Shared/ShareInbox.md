# ShareInbox.swift

Shared app/extension contract in group.app.hopper.player. Atomic UUID JSON files hold one URL
or OPML document. The extension never starts the Rust node, network or audio agent. The main app
removes a manifest only after its durable core outbox acknowledges it, using UUID as batch ID
to make replay safe. Bounded reads cap incoming files at 2 MB; the core remains XML authority.
App Group entitlement/provisioning must be present on both targets. Missing configuration has
an explicit manual paste/file-import fallback. The inbox is not a background execution promise.
