import { pairDevices } from './pairing.js';
import { addedFolders, sharedFolders } from './folders.js';
import { homePicker, deliveryList } from './home.js';
import { api, escape as esc, time, duration, tone, initials } from './api.js';
import { Player } from './player.js';
import { Podcasts } from './podcasts.js';
import { Glass } from './glass.js';

const $ = id => document.getElementById(id);
let library = { items: [], peers: [], queue: { items: [] } }, view = 'all', folder = '', query = '', toastTimer;
const player = new Player(renderQueue, message => toast(message, true));
const podcasts = new Podcasts({ dialog, toast, load, player });
const refreshAddedFolders = addedFolders($('added-folder-rows'), dialog, toast, load);
const glass = new Glass();
$('appearance').onclick = () => glass.settings(dialog);
const names = { all: ['In the hopper', 'A home for everything you want to hear. And see.', 'All media'], continue: ['Pick up the thread', 'Right where you left off. Wherever you are.', 'Continue listening'], audiobook: ['Another chapter', 'Long stories. A little time for yourself.', 'Audiobooks'], audio: ['Good company', 'The voices, sounds, and songs you keep coming back to.', 'Audio'], video: ['Worth a watch', 'A little something for the bigger picture.', 'Video'], folders: ['Just as you left it', 'Your folders, across the devices you call home.', 'Folders'] };

function toast(message, error = false) { clearTimeout(toastTimer); $('toast').textContent = message; $('toast').classList.toggle('error', error); $('toast').hidden = false; toastTimer = setTimeout(() => { $('toast').hidden = true; }, error ? 10000 : 4500); }
function art(item, extra = '') { return `<div class="art ${tone(item.id)} ${extra}" aria-hidden="true">${esc(initials(item))}<span>${item.kind === 'video' ? '▶' : item.kind === 'audiobook' ? '▥' : '♫'}</span></div>`; }
function locationLabel(item) { return item.sources.some(s => s.local && s.online) ? 'On this device' : item.sources.filter(s => s.online).map(s => s.name).join(', ') || 'Source offline'; }
function filtered() {
  return library.items.filter(item => (view === 'all' || view === 'folders' || view === 'continue' && item.progress?.position > 0 && !item.progress.completed || item.kind === view)
    && (!folder || item.folder === folder) && [item.title, item.creator, item.album, item.folder, item.filename].some(s => s.toLowerCase().includes(query)))
    .sort((a, b) => a.folder.localeCompare(b.folder, undefined, { numeric: true }) || a.title.localeCompare(b.title, undefined, { numeric: true }));
}
async function load(sync = false) {
  try { library = await api(sync ? 'refresh' : 'library', sync ? {} : undefined); player.update(library); render(); }
  catch (e) { toast(e.message, true); }
  if (view === 'folders') await refreshAddedFolders();
}
function render() {
  podcasts.library = library;
  $('device-name').textContent = library.device?.name || 'This device';
  $('library-count').textContent = library.items.length;
  $('device-count').textContent = library.peers.length + 1;
  $('device-summary').innerHTML = library.peers.slice(0, 4).map(p => `<div><i class="${p.online ? 'live-dot' : 'offline-dot'}"></i><span>${esc(p.name)}</span><small>${p.online ? 'Online' : 'Offline'}</small></div>`).join('') || '<p>Your other devices belong here.</p>';
  $('connection-status').textContent = `${library.peers.filter(p => p.online).length + 1} DEVICE${library.peers.filter(p => p.online).length ? 'S' : ''} ONLINE · YOURS ALWAYS`;
  const [title, description, crumb] = view === 'podcasts' ? ['Good company', 'Your shows. Your listening ritual.', 'Podcasts'] : names[view];
  $('view-title').innerHTML = `${esc(title)}<span>.</span>`; $('view-description').textContent = description; $('crumb').textContent = crumb;
  document.querySelectorAll('[data-view]').forEach(el => el.classList.toggle('active', el.dataset.view === view));
  $('added-folders').hidden = view !== 'folders';
  $('play-all').hidden = view === 'podcasts';
  if (view === 'podcasts') { $('continue-card').hidden = true; podcasts.render(library, query); renderQueue(); return; }
  const resume = library.items.filter(i => i.progress?.position > 0 && !i.progress.completed).sort((a, b) => (b.progress.updated_at || 0) - (a.progress.updated_at || 0))[0];
  $('continue-card').hidden = !resume || view !== 'all' || Boolean(query);
  if (resume) $('continue-card').innerHTML = `${art(resume, 'hero-art')}<div class="continue-copy"><div class="eyebrow">RIGHT WHERE YOU LEFT OFF</div><h2>${esc(resume.title)}</h2><p>${esc(resume.creator || resume.folder || 'Your collection')} <span>·</span> ${time(resume.progress.position)} in</p><progress max="${resume.duration || 1}" value="${resume.progress.position}"></progress></div><button class="round-play" data-play="${resume.id}" aria-label="Resume ${esc(resume.title)}">▶</button>`;
  if (folder && !library.items.some(item => item.folder === folder)) folder = '';
  const items = filtered();
  $('section-label').innerHTML = `${folder ? esc(folder) : 'Your collection'} <span>${items.length} item${items.length === 1 ? '' : 's'}</span>`;
  $('play-all').disabled = !items.length;
  $('folder-list').hidden = view !== 'folders';
  if (view === 'folders') $('folder-list').innerHTML = `<button class="folder-chip ${!folder ? 'selected' : ''}" data-folder="">All folders</button>` + [...new Set(library.items.map(i => i.folder).filter(Boolean))].sort().map(f => `<button class="folder-chip ${folder === f ? 'selected' : ''}" data-folder="${esc(f)}">▱ ${esc(f)}</button>`).join('');
  if (!items.length) {
    $('collection').innerHTML = !library.items.length ? `<div class="empty-state"><div class="empty-illustration"><div class="record"></div><div class="book">h.</div><div class="hopper-bin">↓</div></div><h2>Good things go in here.</h2><p>Drop in a few favorite tracks, a long audiobook,<br>or the next episode you can’t wait to hear.</p><button class="primary" data-add>＋ Add your first files</button><small>MP3, M4B, FLAC, WAV, MP4 & more</small><button class="text-button" data-pair>Already have a hopper? Pair a device ↗</button></div>` : '<div class="empty-filter"><span>⌕</span><h2>A little quiet in here.</h2><p>Try a different search or collection.</p></div>';
  } else {
    $('collection').innerHTML = `<div class="list-heading"><span>IN YOUR COLLECTION</span><span>AVAILABLE FROM</span><span>LENGTH</span><span></span></div><div class="media-list">` + items.map(item => `<article class="media-row ${player.current === item.id ? 'current' : ''}"><button class="media-main" data-play="${item.id}" aria-label="Play ${esc(item.title)}">${art(item)}<div class="media-copy"><strong>${esc(item.title)}</strong><small>${esc(item.creator || item.album || item.folder || item.filename)}</small>${item.progress?.position ? `<progress max="${item.duration || item.progress.duration || item.progress.position}" value="${item.progress.position}"></progress>` : ''}</div></button><div class="source-label"><i class="${item.sources.some(s => s.online) ? 'live-dot' : 'offline-dot'}"></i>${esc(locationLabel(item))}</div><span class="length">${item.duration ? time(item.duration) : '—'}</span><div class="row-actions"><button class="icon-button" data-enqueue="${item.id}" title="Add to queue" aria-label="Queue ${esc(item.title)}">＋</button><button class="icon-button" data-detail="${item.id}" title="Details and chapters" aria-label="Details for ${esc(item.title)}">···</button></div></article>`).join('') + '</div>';
  }
  renderQueue();
}
function renderQueue() {
  $('queue-count').textContent = player.queue.length;
  $('clear-queue').disabled = !player.queue.length;
  $('queue').innerHTML = player.queue.map((id, index) => {
    const item = player.items.get(id); if (!item) return '';
    return `<div class="queue-item ${id === player.current ? 'current' : ''}"><div class="queue-index">${id === player.current ? '♫' : String(index + 1).padStart(2, '0')}</div><button data-play="${id}" class="queue-track">${art(item)}<div><strong>${esc(item.title)}</strong><small>${duration(item.duration)}</small></div></button><div class="queue-actions"><button class="icon-button" data-up="${id}" title="Move earlier" aria-label="Move ${esc(item.title)} earlier">↑</button><button class="icon-button" data-remove="${id}" title="Remove" aria-label="Remove ${esc(item.title)}">×</button></div></div>`;
  }).join('') || '<div class="queue-empty"><span>☷</span><h3>A little room for more.</h3><p>Add something with ＋<br>and let the day unfold.</p></div>';
}
function dialog(title, body) { $('dialog-title').textContent = title; $('dialog-body').innerHTML = body; if (!$('dialog').open) $('dialog').showModal(); }
function addDialog() {
  dialog('Put something in the hopper', `<p class="dialog-lead">A file, a folder, a whole afternoon of listening.</p><button data-shared-folders class="import-choice"><span>▱</span><div><strong>Share a desktop folder</strong><small>Stream its media in place from paired devices</small></div>↗</button><button id="choose-files" class="import-choice"><span>♫</span><div><strong>Choose media files</strong><small>Copy audio, audiobooks, or video into Hopper</small></div>↗</button><button id="choose-folder" class="import-choice"><span>▱</span><div><strong>Choose a folder</strong><small>Keep its folder structure in your library</small></div>↗</button><details><summary>Or reference a folder on this desktop</summary><p class="hint">With this desktop as Home, files stay where they are. With a remote Home, Hopper makes a copy to send.</p><form id="import-path"><input name="path" placeholder="Full path to a file or folder" required><button class="primary">Add folder</button></form></details>`);
  $('choose-files').onclick = () => $('file-input').click(); $('choose-folder').onclick = () => $('folder-input').click();
  const podcastButton = document.createElement('button'); podcastButton.className = 'import-choice'; podcastButton.textContent = '◉ Subscribe to a podcast ↗'; podcastButton.onclick = () => { view = 'podcasts'; render(); podcasts.subscribe(); }; $('dialog-body').prepend(podcastButton);
  $('import-path').onsubmit = async e => { e.preventDefault(); const button = e.target.querySelector('button'); button.disabled = true; button.textContent = 'Reading your collection…'; try { const r = await api('import', { paths: [new FormData(e.target).get('path')] }); imported(r); } catch (e) { toast(e.message, true); } finally { button.disabled = false; button.textContent = 'Add folder'; } };
}
function imported(report) { $('dialog').close(); toast(`${report.added} item${report.added === 1 ? '' : 's'} added${report.skipped ? ` · ${report.skipped} other files skipped` : ''}${report.errors.length ? ` · ${report.errors[0]}` : ''}`, report.errors.length > 0); load(); }
async function upload(files) {
  if (!files.length) return;
  const body = new FormData(); for (const file of files) body.append('files', file, file.webkitRelativePath || file.name);
  toast(`Putting ${files.length} file${files.length === 1 ? '' : 's'} in the hopper…`); clearTimeout(toastTimer);
  try { imported(await api('upload', body)); } catch (e) { toast(e.message, true); }
}
function pairDialog() { pairDevices(dialog, toast, load); }
function openSharedFolders() { sharedFolders(dialog, toast, load).catch(error => toast(error.message, true)); }
function devicesDialog() {
  dialog('Your devices', homePicker(library) + deliveryList(library) + `<p class="dialog-lead">One collection, wherever you put it.</p><div class="device-card"><span class="live-dot"></span><div><strong>${esc(library.device?.name)}</strong><small>This device</small></div></div>` + library.peers.map(p => `<div class="device-card"><span class="${p.online ? 'live-dot' : 'offline-dot'}"></span><div><strong>${esc(p.name)}</strong><small>${p.online ? 'Connected' : esc(p.error || 'Not connected yet')}</small></div><button class="text-button" data-unpair="${p.id}">Unpair</button></div>`).join('') + '<button class="primary" data-pair>＋ Pair a device</button><p class="hint">Keep a desktop running to stream from it. Download a copy to listen when it is offline.</p>');
  document.getElementById('home-device').onchange = async event => { try { await api('home', { device_id: event.target.value }); await load(); toast('Home desktop saved'); } catch (e) { toast(e.message, true); } };
}
function detail(id) {
  const item = library.items.find(i => i.id === id); if (!item) return;
  dialog(item.title, `<div class="detail-hero">${art(item)}<div><strong>${esc(item.creator || item.kind)}</strong><p>${esc(item.folder || item.filename)}</p><small>${duration(item.duration)} · ${(item.bytes / 1024 / 1024).toFixed(1)} MB</small></div></div><div class="detail-actions"><button class="primary" data-play="${id}">▶ Play</button><button class="secondary" data-enqueue="${id}">＋ Queue</button>${item.sources.some(s => s.local && s.online) ? '<span class="local-badge">✓ On this device</span>' : `<button class="secondary" data-keep="${id}">↓ Keep on this device</button>`}</div><div class="eyebrow">AVAILABLE FROM</div><p>${item.sources.map(s => `${esc(s.name)}${s.local ? ' (this device)' : ''}${s.online ? '' : ' · offline'}`).join('<br>')}</p>${item.chapters.length ? '<div class="eyebrow">CHAPTERS</div><div class="chapters">' + item.chapters.map(c => `<button data-chapter="${c.start}" data-item="${id}"><span>${esc(c.title)}</span><small>${time(c.start)}</small></button>`).join('') + '</div>' : '<p class="hint">No embedded chapters were found. Desktop chapter extraction uses ffprobe when available.</p>'}`);
}
document.addEventListener('click', async e => {
  const target = e.target.closest('button'); if (!target) return;
  const d = target.dataset;
  try { await podcasts.click(target); } catch (error) { toast(error.message, true); }
  if (d.deliveryRetry) { try { await api(`deliveries/${d.deliveryRetry}/retry`, {}); await load(); toast('Queued to retry'); } catch (e) { toast(e.message, true); } }
  if (d.play) player.play(d.play);
  if (d.enqueue) { player.enqueue(d.enqueue); toast('Added to your queue'); }
  if (d.detail) detail(d.detail);
  if (d.up) player.move(d.up, -1);
  if (d.remove) player.remove(d.remove);
  if (d.view) { view = d.view; folder = ''; render(); if (view === 'folders') await refreshAddedFolders(); }
  if ('folder' in d) { folder = d.folder; render(); }
  if ('add' in d) addDialog();
  if ('pair' in d) pairDialog();
  if ('sharedFolders' in d) openSharedFolders();
  if (d.keep) { target.disabled = true; target.textContent = 'Downloading…'; try { await api(`keep/${d.keep}`, {}); toast('Ready to listen offline'); await load(); detail(d.keep); } catch (e) { toast(e.message, true); target.disabled = false; target.textContent = '↓ Keep on this device'; } }
  if (d.unpair && confirm('Unpair this device? Downloaded copies will stay available.')) { try { await api(`peers/${d.unpair}`, undefined, 'DELETE'); await load(); devicesDialog(); } catch (e) { toast(e.message, true); } }
  if ('chapter' in d) { if (player.current !== d.item || !player.media.src) await player.play(d.item); player.seek(Number(d.chapter)); }
});
$('add').onclick = addDialog; $('pair-side').onclick = pairDialog; $('devices-nav').onclick = devicesDialog;
$('dialog-close').onclick = () => $('dialog').close(); $('dialog').onclick = e => { if (e.target === $('dialog')) $('dialog').close(); };
$('refresh').onclick = async () => { $('refresh').classList.add('spinning'); await load(true); $('refresh').classList.remove('spinning'); };
$('search').oninput = e => { query = e.target.value.toLowerCase(); render(); };
$('play-all').onclick = () => player.playAll(filtered()); $('clear-queue').onclick = () => player.clear();
$('chapters').onclick = () => { if (player.current) detail(player.current); };
$('show-video').onclick = () => { $('video-container').hidden = false; }; $('close-video').onclick = () => { $('video-container').hidden = true; };
for (const id of ['file-input', 'folder-input']) $(id).onchange = e => { upload([...e.target.files]); e.target.value = ''; };
let dragDepth = 0;
window.addEventListener('dragenter', e => { if ([...e.dataTransfer.types].includes('Files')) { e.preventDefault(); dragDepth++; $('drop-overlay').hidden = false; } });
window.addEventListener('dragover', e => e.preventDefault());
window.addEventListener('dragleave', () => { if (--dragDepth <= 0) $('drop-overlay').hidden = true; });
window.addEventListener('drop', e => { e.preventDefault(); dragDepth = 0; $('drop-overlay').hidden = true; upload([...e.dataTransfer.files]); });
document.querySelector('.brand').onclick = e => { e.preventDefault(); view = 'all'; folder = ''; render(); };
await load(); setInterval(() => load(), 15000);
