# hopper-agent-mcp.py

Local stdio MCP bridge for external audio agents. Python standard library only. Uses newline JSON-RPC
and the initialize/tools protocol through 2025-11-25 (newer clients must support that negotiated version).
Read-only SQLite access determines whether an inbox job is active; tools expose only those jobs,
never arbitrary caller-supplied paths. Source paths remain local to this desktop and its trusted agent.

List/get jobs, submit validated source-bound analysis, or explicitly report failure. Results are private,
atomic files; the Rust worker revalidates before rendering. Submit is serialized and duplicate/stale
results fail. No inference is simulated and the bridge does not grant an agent audio understanding.
The external client needs actual audio tools and an active turn/watch to poll pending jobs.
No third-party app configuration is changed automatically. See docs/agents.md for setup.

hopper_listen_clip returns native MCP audio content plus its source offset for up to 60 seconds of the active managed episode. FFmpeg decodes a bounded mono PCM excerpt locally. The bridge checks the active job again before returning audio. An audio-capable client on another machine can use an SSH stdio transport and hear clips without a shared filesystem.
