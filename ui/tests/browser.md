# browser.mjs

## Purpose
Run the real desktop UI in isolated headless Chromium with a synthetic media/podcast catalogue.
Verify asynchronous material installation, accessibility and existing layout/interaction contracts.

## Contracts
Run `node ui/tests/browser.mjs`. Node 22+ and Chrome required. Set CHROME_BIN on Linux/Windows;
HOPPER_SCREENSHOTS optionally redirects PNG evidence (default artifacts/glass, ignored by Git).
Uses an ephemeral local HTTP server/profile; no personal browser state or library is read.
Fixture IDs deliberately vary their leading bytes so every artwork palette is represented.

## Checks
- Real CSP, decoded per-card rim lenses and shared podcast ink.
- Light/dark geometry parity, preference persistence and independent tint values.
- Dialog lens exclusion, solid reduced-transparency/increased-contrast/forced-color surfaces.
- Import dialog, error color, compact layout and filter cleanup across rerenders.
- Non-Chromium engine guard retains CSS glass without creating SVG filters (UA emulation;
  this verifies the guard, not actual WebKit rasterization).
- Complete initial material even when background-window animation frames are suspended.
- Browser exceptions and console errors fail the run. Screenshots require visual review separately.

An additional 4,800-item fixture drills into a plotline and season and verifies bounded episode rendering, starting at the selected episode, the saved queue, and advancing within the season.
