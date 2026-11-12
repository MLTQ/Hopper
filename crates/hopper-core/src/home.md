# home.rs

Preferred Home desktop, durable additions and authenticated receipt handling. Home selection is
local to each installation; only a paired desktop or this desktop is valid. iOS has no default Home.
Desktop defaults to itself. Changing Home affects new additions, never silently redirects old work.

OPML/Apple/RSS additions and imported media create persistent outbox records. Optional batch IDs
make share-extension retries idempotent. Reference imports stage a verified managed copy before
delivery; source files are never moved/deleted. Receiver accepts only published IDs from the paired
sender's catalogue, downloads with hash verification, and acknowledges once durable. Receipts are
idempotent per sender/job ID. No filesystem paths cross the mesh. Incoming items never forward again.

Episode links follow the show and save the matched episode; an unmatched episode is explicitly
reported. All cut/agent policy stays with the show owner. Playback queues remain device-local.

Receiver completion additionally stages any pre-existing referenced local replica, ensuring the
Home receipt denotes a managed copy. Boxed receipt futures break the bidirectional transport
future-type cycle while retaining Send for per-stream tasks.

Staging a file from a one-time folder import preserves its membership. Removing the folder during delivery keeps the staged bytes available for Home until outstanding receipts arrive.
