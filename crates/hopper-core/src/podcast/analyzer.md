# analyzer.rs

Uniform source-bound analysis interface for built-in audio inference, Hermes CLI, custom argv,
and an MCP/file inbox. Writes a private request manifest for each job, reuses only valid cached
analysis, validates every provider's result, and bounds execution to four hours. Cancellation
drops HTTP work and kills a child process. Inbox mode waits for an atomic analysis.json or error.json.
Only trusted local configuration can launch commands; no shell evaluation. Hermes receives a
single `chat --quiet -q` request and uses its configured listening or source-timed transcription tools.
No undocumented Dot/Muse command is assumed: those clients can use the shared MCP inbox if supported.

The Hermes launcher includes the repository’s hopper-cut-ads skill in the job prompt, so it uses the current source-time and atomic-output contract without a separate skill installation. Hermes may use local timed transcription generated from the immutable job input for spoken ads. Other modes retain their existing evidence contract.

New manifests request ad, promo, content and uncertain classifications. The renderer also accepts older conversation labels and protects them.
