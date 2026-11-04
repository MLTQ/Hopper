# DevicesView.swift

## Purpose
Explicit device pairing, invitation sharing, connection status and local trust revocation.

## Contracts
Invitations are one-use secrets. ShareLink only shares after the user's explicit action. Names
are labels; authenticated keys determine identity. Device removal does not delete cached media.

HomeSection chooses a paired desktop, including offline hosts; DeliverySection shows receipts and retries.

AppearanceSection provides separate device-local light/dark glass tint sliders. The list uses
the shared scene behind solid system rows; pairing and Home ownership are unaffected.
