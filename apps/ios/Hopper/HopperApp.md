# HopperApp.swift

Podcasts is a fourth native tab with the same persistent player and client environment.

The cream/forest theme follows the system appearance using the dynamic HopperTheme palette.
Primary actions use a contrast-safe accent; native materials respect Reduce Transparency.
The mini-player is inset inside each tab's content, keeping system tab navigation reachable.

## Purpose
Native iOS composition root: one client and player survive tab/sheet changes.

## Contracts
Node startup precedes queue restoration. Foreground refreshes peer catalogues, backgrounding
checkpoints playback without stopping the player. UI surfaces actual errors. No webview hosts
the player, and no app lifecycle event synthesizes playback without user intent.

Returning to the foreground resumes App Group share intake after refreshing the library.
