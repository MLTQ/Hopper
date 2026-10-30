import { api, escape as esc } from './api.js';

export async function agentSettings({ dialog, toast, load }) {
  const config = await api('podcasts/agent');
  dialog('Your audio agent', `<p class="dialog-lead">Choose who listens. Hopper handles the edits.</p>
    <form id="agent-form" class="podcast-form">
      <label>Processing on this desktop<select name="mode">
        <option value="builtin">Hopper · connect an audio LLM</option><option value="hermes">Hermes agent</option>
        <option value="command">Custom agent command</option><option value="inbox">External agent · MCP inbox</option>
      </select></label>
      <div id="agent-builtin" class="podcast-form">
        <label>Audio LLM endpoint<input name="endpoint" type="url" value="${esc(config.endpoint)}" placeholder="https://…/v1/chat/completions"></label>
        <label>Audio-capable model<input name="model" value="${esc(config.model)}" placeholder="Model name from your provider"></label>
        <label>API key<input name="key" type="password" autocomplete="new-password" placeholder="${config.key_present ? 'Saved · leave blank to keep' : 'Optional for a local server'}"></label>
        <label class="setting-toggle"><span>Remove saved key</span><input name="clear_key" type="checkbox"></label>
        <p class="hint">Uses audio-input Chat Completions. Starting a cut sends episode audio to this endpoint; provider charges may apply. The key stays on this desktop.</p>
      </div>
      <div id="agent-command" class="podcast-form">
        <label>Executable and arguments · JSON array<input name="command" value="${esc(JSON.stringify(config.command.length ? config.command : ['hermes']))}" spellcheck="false"></label>
        <p class="hint" id="agent-command-hint"></p>
      </div>
      <div id="agent-inbox"><p class="hint">Connect Hopper’s MCP bridge to your agent app and ask it to process pending jobs. The agent must have audio-understanding tools. This mode waits for the agent; it cannot launch Dot or Muse automatically.</p><p class="hint">Job folder on this desktop:<br><code>${esc(config.jobs_directory)}</code></p><p class="hint">Setup instructions: docs/agents.md in the Hopper project.</p></div>
      <p class="hint">FFmpeg and FFprobe must be installed on this desktop. You can start cuts from any paired device.</p>
      <button class="primary" type="submit">Save agent settings</button>
    </form>`);
  const form = document.getElementById('agent-form');
  form.elements.mode.value = config.mode;
  const update = () => {
    const mode = form.elements.mode.value;
    document.getElementById('agent-builtin').hidden = mode !== 'builtin';
    document.getElementById('agent-command').hidden = !['hermes', 'command'].includes(mode);
    document.getElementById('agent-inbox').hidden = mode !== 'inbox';
    form.elements.endpoint.required = form.elements.model.required = mode === 'builtin';
    document.getElementById('agent-command-hint').textContent = mode === 'hermes'
      ? 'Hopper starts Hermes with one chat request. Hermes needs its own working audio tools and LLM credentials. Use an absolute executable path if it is not on Hopper’s PATH.'
      : 'Hopper appends --request and --output paths. Your command must listen to the supplied audio and return timestamped analysis JSON. No shell is used.';
  };
  form.elements.mode.onchange = update; update();
  form.onsubmit = async event => {
    event.preventDefault(); const button = form.querySelector('button[type=submit]'); button.disabled = true;
    try {
      const mode = form.elements.mode.value;
      const command = ['hermes','command'].includes(mode) ? JSON.parse(form.elements.command.value) : [];
      if (!Array.isArray(command) || command.some(arg => typeof arg !== 'string')) throw new Error('Enter an array of quoted executable arguments.');
      const body = { mode, endpoint: form.elements.endpoint.value.trim(), model: form.elements.model.value.trim(), command };
      if (form.elements.clear_key.checked) body.api_key = '';
      else if (form.elements.key.value) body.api_key = form.elements.key.value;
      await api('podcasts/agent', body);
      form.elements.key.value = ''; document.getElementById('dialog').close(); await load(); toast('Agent settings saved. Waiting jobs can now start.');
    } catch (error) { toast(error.message, true); } finally { button.disabled = false; }
  };
}
