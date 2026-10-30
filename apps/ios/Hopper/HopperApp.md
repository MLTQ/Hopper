# HopperApp.swift

Podcasts is a fourth native tab with the same persistent player and client environment.

The current cream/forest theme explicitly uses light appearance so system text and list surfaces
retain readable contrast when the device itself uses dark mode. A complete dark palette is future work.
The mini-player is inset inside each tab's content, keeping system tab navigation reachable.

## Purpose
Native iOS composition root: one client and player survive tab/sheet changes.

## Contracts
Node startup precedes queue restoration. Foreground refreshes peer catalogues, backgrounding
checkpoints playback without stopping the player. UI surfaces actual errors. No webview hosts
the player, and no app lifecycle event synthesizes playback without user intent.
