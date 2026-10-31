import { api, escape as esc } from './api.js';

export function deliveryList(library) {
  const jobs = library.deliveries || [];
  if (!jobs.length) return '';
  return `<section class="home-deliveries"><h3>Sending to Home</h3>${jobs.map(j => `<div class="device-card"><div><strong>${esc(j.label)} · ${esc(j.owner.name)}</strong><small>${esc(j.message)}</small></div>${['waiting', 'failed'].includes(j.state) ? `<button class="text-button" data-delivery-retry="${j.id}">Retry</button>` : ''}</div>`).join('')}</section>`;
}
export function homePicker(library) {
  const hosts = (library.podcasts?.hosts || []).filter(h => h.desktop);
  return `<div class="podcast-form"><label>Home desktop<select id="home-device">${hosts.map(h => `<option value="${h.device.id}" ${library.home?.id === h.device.id ? 'selected' : ''}>${esc(h.device.name)}${h.online ? '' : ' · offline'}</option>`).join('')}</select></label><p class="hint">New subscriptions and imported files go here. Set the same Home on your laptop and phone. Keep Hopper running on Home to receive additions, prepare audio, and stream.</p><p class="hint">Changing Home affects new additions. Existing subscriptions stay on their current desktop.</p></div>`;
}
export function importPodcasts({ library, dialog, toast, load }) {
  const hosts = (library.podcasts?.hosts || []).filter(h => h.desktop);
  if (!hosts.length) { toast('Pair a desktop in Devices first.', true); return; }
  dialog('Bring your podcasts', `<form id="podcast-import" class="podcast-form"><label>Apple Podcasts or RSS link<input name="url" type="url" placeholder="https://podcasts.apple.com/…"></label><p class="hint">Paste a show or episode link from Apple Podcasts. For your Castamatic subscriptions, export its OPML file and choose it below.</p><label>Or import a subscription list<input id="opml-file" type="file" accept=".opml,.xml,text/x-opml,application/xml,text/xml"></label><label>Send to<select name="owner">${hosts.map(h => `<option value="${h.device.id}" ${library.home?.id === h.device.id ? 'selected' : ''}>${esc(h.device.name)}${h.online ? '' : ' · offline'}</option>`).join('')}</select></label><p class="hint">Hopper keeps pending additions until this desktop receives them. OPML brings subscriptions; listening history and downloads stay in your old app.</p><button class="primary">Put in the hopper</button></form>`);
  document.getElementById('podcast-import').onsubmit = async event => {
    event.preventDefault(); const form = event.target, button = form.querySelector('button'), data = new FormData(form), file = document.getElementById('opml-file').files[0];
    button.disabled = true;
    try {
      if (file && data.get('url').trim()) throw new Error('Choose a link or a subscription file, one at a time.');
      if (file?.size > 2 * 1024 * 1024) throw new Error('Subscription lists must be smaller than 2 MB.');
      const body = { owner_id: data.get('owner'), ...(file ? { opml: await file.text() } : { url: data.get('url') }) };
      const result = await api('add-podcasts', body);
      document.getElementById('dialog').close(); await load();
      toast(`${result.delivery_ids.length} subscription${result.delivery_ids.length === 1 ? '' : 's'} queued for Home${result.warnings.length ? ` · ${result.warnings[0]}` : ''}`, result.warnings.length > 0);
    } catch (e) { toast(e.message, true); } finally { button.disabled = false; }
  };
}
