# podcast/llm.rs

Hopper's built-in audio-analysis harness uses an audio-capable OpenAI-compatible Chat Completions
endpoint. It sends actual mono WAV audio, not a guessed transcript, in two-minute regions with
8 seconds of context on either side. Clip-relative timestamps are validated and assigned only to
that region's ownership interval. Source/model/prompt-specific checkpoints allow restart without
repeating completed inference. Keys never enter logs, manifests or the peer catalogue.

Requires user-configured endpoint/model/credentials. No default paid calls. At most three attempts
for rate limits/server errors; bound HTTP time and response size, disable redirects, reject malformed
or out-of-range results. Confidence is model-estimated, not guaranteed accuracy; real ad-detection
evaluation remains necessary. Invalid responses preserve originals and fail visibly.

Protocol reference: https://developers.openai.com/api/docs/guides/audio-chat-completions

Checkpoint fingerprints include cut preferences; only validated chunk decisions are cached.

Desktop FFmpeg tools resolve through media_tools, including standard macOS Homebrew locations.

The prompt focuses on promotional intent and evidence in ad breaks and sponsor reads across podcast genres. Normal conversation is content.
