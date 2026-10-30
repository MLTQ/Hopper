# podcast/agent.rs

Local-only processing configuration: built-in audio LLM, generic executable, Hermes CLI or external
MCP inbox. Config/keys never enter peer snapshots. Status returns key presence, never the key.
Save via a private atomic file in the node directory; no shell evaluation. Omitted key preserves
an existing key only at the same endpoint; empty key clears it. HOPPER_LLM_API_KEY is an optional
runtime credential. Remote endpoints require HTTPS; localhost permits local inference servers.
Legacy HOPPER_AD_AGENT/agent-command.json remains supported and visibly overrides UI configuration.
Only a desktop-local authenticated API may configure executable commands or credentials.
