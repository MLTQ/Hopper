# agents.js

Local desktop audio-agent setup dialog. Offers built-in audio-input Chat Completions, Hermes CLI,
custom executable, or MCP inbox. Credentials never appear in status responses; blank retains the
existing key only for the same endpoint, explicit removal clears it. Calls the loopback-only agent
API, never an owner-forwarded peer command. Endpoint/model are deliberate choices; no paid model
is preselected. Explains audio transfer before the user starts cuts and external-client requirements.
