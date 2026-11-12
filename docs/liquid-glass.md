# Liquid glass appearance

The existing Hopper layout, type, spacing, icons, radii and media behavior are preserved. Glass
is a shared surface treatment over a quiet, locally drawn forest/amber/teal scene. No remote
image, font, video readback or personal artwork is needed. Both interfaces follow system light
and dark appearance; desktop also allows an explicit appearance choice.

## Surface inventory

| Surface | Location | Material / lens |
|---------|----------|-----------------|
| Device/navigation sidebar | Desktop `.sidebar` | CSS glass; no SVG on fixed chrome |
| Up next panel | Desktop `.queue-panel` | CSS glass; no SVG on large scrolling chrome |
| Persistent player | Desktop `.player` | CSS glass; no SVG on fixed playback chrome |
| Resume card | Desktop `.continue-card` | Glass + rim lens and subtle color split on Chromium |
| Podcast show cards | Desktop `.podcast-card` | Same lens; neighbouring cards share ink |
| Import/pair/device/detail/settings dialog | Desktop `dialog` | Dense glass; no SVG on top-layer content |
| Status toast / drop target | Desktop `#toast`, `#drop-overlay` | CSS glass; errors retain red; no animated/composited lens |
| Video close button | Desktop `#close-video` | Dense dark overlay; no video sampling/lens |
| Mini-player / speed control | iOS `MiniPlayer`, `PlayerView` | Native material, tint and rim; original geometry |
| Native navigation, menus, sheets and search | SwiftUI/system | OS-owned material and accessibility behavior |
| Library/episode reading rows | Both interfaces | Solid or unfiltered; long text stays quiet |
| Search/form inputs and import choices | Desktop/native | Solid, clear boundaries for data entry |
| Empty state, artwork and video | Both interfaces | Solid original content; no glass over media itself |

## Material and preferences

Desktop `material.css` owns registered numeric/color tokens, directional rim highlights, edge
shadows, ink and full light/dark palettes. `styles.css` retains all component geometry. `glass.js`
samples the known local backdrop on a 5 × 3 grid, trims the two darkest/brightest points for ink
selection, uses hysteresis, and raises density when secondary contrast needs it. It still checks
the extreme samples when calculating contrast. Unknown/layered content uses a dense fallback.
Tint values below the default deliberately reduce opacity and can reduce contrast.

Appearance (the existing top-right H button) stores separate light/dark tint scales, 0–150%,
and a system/light/dark choice in local storage. iOS Devices → Appearance exposes the two scales
in AppStorage and follows the device appearance. No presentation preference enters Rust or sync.

Chromium cards get a decoded SVG displacement filter with a neutral center and inward rim bend.
The rim is min(44px, short side / 3.5), with a 4px minimum and 2.6 falloff; red/blue split ±15%.
Cached corner/rim tiles are moved with nine-slice composition on resize. Every surface has a
unique filter; removed surfaces release it. WebKit (including macOS Tauri) and Firefox retain
CSS glass. Fixed, top-layer, transformed, small and will-change surfaces do not get the SVG lens.

Native iOS uses `HopperTheme` and `hopperGlass` with the OS ultra-thin material; it does not emulate
SVG refraction or inspect playback pixels. Reduced transparency, increased contrast and forced
colors use solid surfaces. Existing focus, disabled and semantic error states are retained.

## Changed files

- Desktop shell: `ui/index.html`, `ui/index.md`, `ui/app.js`, `ui/app.md`, `ui/styles.md`.
- Desktop material: `ui/material.css`, `ui/material.md`, `ui/glass.js`, `ui/glass.md`,
  `ui/glass-lens.js`, `ui/glass-lens.md`.
- Local asset routes: `crates/hopper-core/src/http.rs`, `crates/hopper-core/src/http.md`.
- Native helpers: `apps/ios/Hopper/HopperTheme.swift`, `HopperTheme.md`,
  `AppearanceSection.swift`, `AppearanceSection.md`.
- Native integration and companion docs: `HopperApp`, `LibraryView`, `PlayerView`, `DevicesView`,
  `PodcastsView` in `apps/ios/Hopper/` (each `.swift` and `.md`).
- Verification: `ui/tests/glass.test.mjs`, `ui/tests/glass.test.md`, `ui/tests/browser.mjs`,
  `ui/tests/browser.md`, `docs/validation.md`, this document and `.beads/issues.jsonl`.

See [validation](validation.md) for observed checks and device/platform limitations.
