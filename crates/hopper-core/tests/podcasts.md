# podcasts.rs

Full local HTTP RSS → paired subscribe → download → external-agent protocol → real FFmpeg render
→ remapped metadata/progress → remote playable variant test. Generates a 12-second WAV, serves RSS,
chapters, VTT and soundbites, and uses a clearly synthetic fixture agent with known classifications.
This verifies integration/rendering, not real-world model ad-recognition accuracy. FFmpeg is required;
CI without it explicitly skips rendering coverage. Preserve byte-exact originals and duplicate GUIDs.

The fixture checks episode and show covers, relative chapter artwork and hidden markers after ad removal.

The paired one-episode cut fixture also checks that the owner keeps its saved show preferences. The same paired nodes verify that a Podcast Index request surfaces the owner’s configuration error before any provider call.
