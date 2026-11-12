# glass.test.mjs

## Purpose
Exercise the material's observable legibility and adaptation contracts with synthetic extremes.

## Contracts
Run `node --test ui/tests/glass.test.mjs`. Covers ink hysteresis, dark appearance, outlier trimming,
texture/chroma response, 4.5:1 secondary contrast including highlights, independent tint scaling
and dense fallback. Does not claim browser filter rasterization or native visual verification.
