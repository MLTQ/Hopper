# DevicesView.swift

## Purpose
Explicit device pairing, invitation sharing, connection status and local trust revocation.

## Contracts
Invitations are one-use secrets. ShareLink only shares after the user's explicit action. Names
are labels; authenticated keys determine identity. Device removal does not delete cached media.
