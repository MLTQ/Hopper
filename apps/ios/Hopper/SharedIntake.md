# SharedIntake.swift

Foreground bridge from App Group share manifests to the Rust durable outbox. Requires a selected
Home, uses each manifest UUID for idempotent intake, and deletes only after successful enqueue.
A reentrancy guard serializes activation/startup and setting-Home triggers. Errors retain the
manifest and display once to avoid repeated alerts. Failed remote deliveries remain in Devices.

Each manifest failure is isolated so later valid shares can proceed; Devices provides explicit
retry/discard for retained invalid inputs.
