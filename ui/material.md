# material.css

## Purpose
Shared liquid-glass tokens and complete light/dark palettes, layered after styles.css. Preserve
all existing component layout, typography, radii and responsive breakpoints.

## Components
- Registered `--glass-*` properties animate tint, ink, brightness and saturation.
- `.glass-surface` supplies tint, a shallow blur, directional rim light and edge shadows.
- Theme tokens cover solid reading surfaces, controls, dividers, selected/hover rows and focus ink.
- Accessibility queries use opaque surfaces and disable refraction/blending.

## Contracts
| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| index.html | Stylesheet loads after styles.css | Cascade order |
| glass.js | Theme and material custom properties/classes | Token names |
| styles.css | Geometry stays in the original stylesheet | Layout overrides |

## Notes
Artwork stays opaque and keeps its own palette. Error toasts retain their original red.
Podcast settings glyphs over pale placeholder artwork retain fixed dark ink in both appearances.
Selected light rows use white layers, never gray washes. The scene contains no downloaded assets.
Navigation glyphs use opaque ink with whole-icon opacity so overlapping strokes stay even.
Secondary ink is #505b53 on light glass and #bac4bc on dark glass, matching the contrast model.
A legacy rgba fallback and prefixed backdrop-filter preserve material on older WebKit versions.
