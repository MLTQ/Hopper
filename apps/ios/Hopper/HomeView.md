# HomeView.swift

Typed intake/delivery contracts and native Home picker/status sections. All known paired desktop
hosts stay selectable offline. Home is a per-device default for future additions, not an owner
migration. Retry explicitly reschedules failed/waiting work. No phone agent or background delivery
is implied; the source app must run long enough to transfer pending additions.

The Home setter decodes the returned Library directly. Devices links to a separate Import activity list with failed counts and retry actions.
