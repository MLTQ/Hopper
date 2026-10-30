# analyzer.rs

Uniform source-bound analysis interface for built-in audio inference, Hermes CLI, custom argv,
and an MCP/file inbox. Writes a private request manifest for each job, reuses only valid cached
analysis, validates every provider's result, and bounds execution to four hours. Cancellation
drops HTTP work and kills a child process. Inbox mode waits for an atomic analysis.json or error.json.
Only trusted local configuration can launch commands; no shell evaluation. Hermes receives a
single `chat --quiet -q` request and must have its own actual audio-understanding tools configured.
No undocumented Dot/Muse command is assumed: those clients can use the shared MCP inbox if supported.
