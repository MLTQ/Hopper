# glass-lens.js

## Purpose
Optional Chromium rim refraction for static resume and podcast cards. CSS glass remains the
complete baseline in WebKit/Tauri macOS, Firefox and accessibility modes.

## Components
- `rimTile`: cache rounded-rectangle displacement fields by radius/rim width (bounded to 24).
  The center is neutral, red/green encode inward normals, and bend falls off with exponent 2.6.
- `mapImage`: nine-slice cached corner/edge geometry to a surface's actual size. Resizing moves
  the existing rim geometry instead of recalculating the displacement field.
- `GlassLens.update/remove/prune`: decode the map before installing one uniquely named SVG filter
  per surface. Split red/blue by ±15% and recombine the three isolated channels with screen.

## Contracts
| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| glass.js | update(element, disabled), remove(element), prune() | Lifecycle signatures |
| material.css | `--glass-lens` is a trailing backdrop-filter URL | Filter ordering |

## Notes
Feature detection includes the engine: Firefox can parse the syntax without rendering it.
Skip surfaces under 32px, transforms, will-change layers, menus, dialogs, fixed chrome and video.
Maps are same-origin data images permitted by the existing CSP. Pending decode is generation
guarded; detached surfaces cannot install a stale filter. Geometry assumes existing circular radii;
Hopper does not use corner-shape superellipses.
