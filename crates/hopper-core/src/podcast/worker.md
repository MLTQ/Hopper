# podcast/worker.rs

Single durable desktop worker. Persist downloading/analysing/rendering/ready/error phases; interrupted
jobs requeue and reuse valid source-bound analysis. No configured agent means waiting_for_agent,
never simulated success. Automatic feed refresh runs every 30 minutes while the node is alive.

Local agent configuration selects built-in audio inference, Hermes, custom argv or an MCP inbox.
The analyzer module owns provider execution and validation. No configured model means waiting,
never simulated success. iOS only requests processing on an online desktop.

Cancellation stops a running agent process and prevents a render from publishing. Rendering is
bounded separately. Commit variants only after verification, preserve originals, remap saved
progress, and store original decisions plus source-to-output spans and edited metadata. A failed
job never replaces an episode's existing playable variant. Process logs remain local/private.

Failure state updates also check cancellation atomically. Ready edit manifests reflect final status.

Guard flow is checked by Clippy; desktop-only analysis dependencies remain gated off on iOS.
