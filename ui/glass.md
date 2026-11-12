# glass.js

## Purpose
Own device-local appearance, the readable local scene and adaptive glass material. Never read
personal artwork, capture video, call the API or touch playback state.

## Components
- `materialFor`: trim a 5 × 3 sample grid, choose ink with ±0.05 hysteresis, calculate texture,
  chroma, density and saturation. Raise tint density until secondary ink reaches 4.5:1 against
  every sampled extreme and the rim highlight at default tint. User-reduced tint can lower contrast.
- `Glass`: persists system/light/dark mode and separate 0–1.5 tint scales; draws a viewport-fixed
  forest/amber scene in a local canvas; resamples visible surfaces after content/theme/position changes.
- `settings`: populate the existing dialog with labeled native controls and a reset action.

## Contracts
| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| app.js | `new Glass()` and `settings(dialog)` | Constructor or dialog signature |
| material.css | `.glass-surface`, `data-glass-ink`, numeric custom properties | Token/class names |
| glass-lens.js | Optional enhancement after CSS material is usable | Lens lifecycle |

## Notes
Podcast cards in a grid share one ink decision. Dialogs and video overlay controls use dense
theme-based material because their actual backdrop can contain content not present in the canvas.
Sampling is limited to Hopper's own drawn backdrop. No cross-origin image or live-video readback.
Rim filters are reclaimed when surfaces disappear. Resize/scroll work is coalesced. Storage failure
must not prevent the app from opening. Reduced transparency, increased contrast and forced colors
disable the lens; CSS supplies solid surfaces. Text colors stay opaque; only decorative icons fade.
Saved ink is validated before use. Sample sorting avoids newer Array APIs for older system webviews.
Initial/theme material setup is synchronous because background native windows can suspend animation
frames. Subsequent geometry/content changes are coalesced into animation frames.
