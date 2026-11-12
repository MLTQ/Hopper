# styles.css

## Purpose
Warm, quiet desktop interface with a fixed device sidebar, collection, queue and persistent player.

## Contracts
No remote fonts/images. Responsive breakpoints hide the side queue on narrow windows; playback
and library remain accessible. Native hidden attributes override component display declarations.
Focus indicators and reduced-motion preferences are respected. Artwork is a deterministic local
placeholder until embedded image extraction is implemented.
Podcast cards, episode rows and labeled cut-settings controls use the same quiet palette. Job
errors/waiting state remain visible alongside an episode's original/playable variants.

Shared glass, adaptive ink and appearance colors live in material.css. This file remains the source
of truth for spacing, radii, typography and responsive geometry.

The pairing code has a white background, square aspect and responsive width so the phone camera can read the complete quiet zone.

Added-folder rows wrap long paths and keep their Remove control visible.
