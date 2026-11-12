# HopperTheme.swift

## Purpose
One native palette, local scene and reusable glass modifier for both system appearances.
Keep presentation separate from the shared Rust core and native player.

## Components
- `HopperTheme`: dynamic background, forest, accent, muted, surface and ink colors. `scene` draws
  quiet forest/amber/teal gradients locally without external assets or animation.
  The separate accent keeps white action labels legible while remaining visible on dark scenes.
- `hopperGlass(radius:)`: native ultra-thin material with a dense tint, inner highlight and rim.
  Reads separate light/dark AppStorage tint scales; bounds them to 0–1.5.
- `hopperScene()`: use the shared scene behind existing containers without changing geometry.

## Contracts
| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| Native views | Stable color names and View modifiers | Names and radius semantics |
| AppearanceSection | hopper.glass.light/dark keys, default 1 | Preference names/ranges |
| HopperApp | System appearance and contrast-safe accent | Appearance ownership |

## Notes
Reduce Transparency and increased contrast replace the material with an opaque background.
Library/episode/form rows stay solid. MediaArt has fixed pale fills and retains fixed dark ink.
No native SVG displacement, screenshot sampling or video readback is attempted; the OS material
handles live backdrop composition. The iOS 16 minimum is preserved.
