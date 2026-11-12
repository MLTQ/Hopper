import { api, escape as esc } from './api.js';

export function browsePodcasts({ library, dialog, toast, load }) {
  const hosts = (library.podcasts?.hosts || []).filter(host => host.desktop);
  const chosen = library.home?.id || hosts[0]?.device.id || '';
  const followed = new Set((library.podcasts?.shows || []).map(show => JSON.stringify([show.owner.id, show.feed_url])));
  const pending = new Set((library.deliveries || []).filter(d => ['waiting','sending'].includes(d.state) && d.payload?.kind === 'podcast')
    .map(d => JSON.stringify([d.owner.id,d.payload.url])));
  dialog('Browse podcasts', `<form id="directory-search" class="podcast-form">
    <label>Directory<select name="provider"><option value="apple">Apple Podcasts · US</option><option value="podcast_index">Podcast Index</option></select></label>
    <label>Search podcasts<input name="term" type="search" maxlength="100" placeholder="Show or publisher"></label>
    <label>Category<select name="category"><option value="">All categories</option></select></label>
    <button class="secondary">Search</button>
    <label>Subscribe on<select name="owner">${hosts.map(h => `<option value="${esc(h.device.id)}" ${h.device.id === chosen ? 'selected' : ''}>${esc(h.device.name)}${h.online ? '' : ' · offline'}</option>`).join('') || '<option value="">Pair a desktop in Devices</option>'}</select></label>
  </form>
  <details id="directory-setup" hidden><summary>Podcast Index credentials</summary>
    <form class="podcast-form"><p>Enter the key and secret issued by Podcast Index on this desktop.</p>
    <label>API key<input name="key" type="password" required autocomplete="off"></label>
    <label>API secret<input name="secret" type="password" required autocomplete="off"></label>
    <button class="secondary">Save credentials</button></form></details>
  <p id="directory-status" role="status"></p><div id="directory-results"></div>`);
  const form = document.getElementById('directory-search'), fields = form.elements;
  const result = document.getElementById('directory-results'), status = document.getElementById('directory-status');
  const setup = document.getElementById('directory-setup');
  let serial = 0, rows = [];
  function render() {
    result.innerHTML = rows.map((s, i) => {
      const key = JSON.stringify([fields.owner.value,s.url]);
      const label = followed.has(key) ? 'Subscribed' : pending.has(key) ? 'Queued' : 'Subscribe';
      return `<article class="directory-row"><div class="directory-cover">◉${s.artwork && /^https?:\/\//i.test(s.artwork) ? `<img src="${esc(s.artwork)}" loading="lazy" alt="" referrerpolicy="no-referrer">` : ''}</div><div><strong>${esc(s.title)}</strong><small>${esc(s.author)}</small></div><button class="secondary" data-listing="${i}" ${!fields.owner.value || label !== 'Subscribe' ? 'disabled' : ''}>${label}</button></article>`;
    }).join('');
    for (const image of result.querySelectorAll('img')) image.onerror = () => image.remove();
  }
  async function search() {
    const version = ++serial, category = fields.category.value, term = fields.term.value.trim();
    rows = []; render(); status.textContent = 'Looking up podcasts…';
    setup.hidden = fields.provider.value !== 'podcast_index' || fields.owner.value !== library.device.id;
    try {
      const page = await api('podcasts/directory', { provider: fields.provider.value, term, category, owner_id: fields.owner.value || null });
      if (version !== serial || !form.isConnected) return;
      fields.category.innerHTML = '<option value="">All categories</option>' + page.categories.map(c => `<option value="${esc(c.id)}">${esc(c.name)}</option>`).join('');
      fields.category.value = category;
      rows = page.shows; render();
      status.textContent = rows.length ? (term ? 'Search results' : fields.provider.value === 'apple' ? 'Top podcasts' : 'Trending podcasts') : 'No shows found. Try another category or search.';
    } catch (e) { if (version === serial) status.textContent = e.message; }
  }
  form.onsubmit = event => { event.preventDefault(); search(); };
  fields.category.onchange = search;
  fields.provider.onchange = () => { fields.category.innerHTML = '<option value="">All categories</option>'; setup.open = false; search(); };
  fields.owner.onchange = () => { if (fields.provider.value === 'podcast_index') search(); else render(); };
  setup.querySelector('form').onsubmit = async event => {
    event.preventDefault(); const settings = event.target, button = settings.querySelector('button');
    button.disabled = true;
    try {
      await api('podcasts/directory/keys', {key: settings.elements.key.value.trim(), secret: settings.elements.secret.value.trim()});
      settings.reset(); setup.open = false; await search();
    } catch (error) { toast(error.message, true); } finally { button.disabled = false; }
  };
  result.onclick = async event => {
    const button = event.target.closest('button[data-listing]');
    if (!button || button.disabled) return;
    const show = rows[Number(button.dataset.listing)], owner = fields.owner.value;
    const key = JSON.stringify([owner,show.url]); pending.add(key); render();
    try {
      await api('add-podcasts', {owner_id: owner, url: show.url});
      toast(`${show.title} queued for subscription`); await load();
    } catch (error) { pending.delete(key); toast(error.message, true); }
    render();
  };
  search();
}
