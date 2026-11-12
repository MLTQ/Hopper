# AppearanceSection.swift

## Purpose
Expose two local glass tint preferences in the existing Devices list.

## Components
- `AppearanceSection`: labeled percentage sliders from 0 to 1.5, plus reset to 1.

## Contracts
| Dependent | Expects | Breaking changes |
|-----------|---------|------------------|
| DevicesView | A self-contained List section | Placement/type |
| HopperTheme | hopper.glass.light and hopper.glass.dark AppStorage values | Keys/defaults/range |

The system selects light/dark appearance. These preferences never sync to peers, modify media,
configure agents, or change Home. Accessibility settings can override transparency.
