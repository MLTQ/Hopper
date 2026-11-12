# agent_modes.rs

Real local worker/HTTP/FFmpeg integration with a deterministic fake audio LLM. Verifies actual WAV
input, explicit model/key configuration, redacted status, key retention only for the same endpoint,
waiting-job wakeup, invalid timestamps preserving the previous variant, and cancellation. Exercises
the real Python MCP bridge over stdio, rejecting traversal and mismatched hashes before accepting
an empty analysis. Requires FFmpeg and Python 3. These are contract tests, not ad-detection evals.

The bridge test requests a clip past the final boundary, decodes its WAV and checks its exact length. It also rejects an oversized request.
