import { createServer } from 'node:http';
import { readFile, writeFile, mkdtemp, mkdir, rm } from 'node:fs/promises';
import { spawn } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';

// Isolated synthetic catalogue. No user library, credentials or external service is involved.
const ui = dirname(dirname(fileURLToPath(import.meta.url)));
const device = { id: 'preview', name: 'Studio Mac', desktop: true };
const items = ['The Art of Noticing', 'A Walk in the Woods', 'Sunday, Slowly', 'Ways of Seeing'].map((title, i) => ({
  id: (i + 1).toString(16).padStart(2, '0').repeat(32), title, creator: ['Rob Walker', 'Bill Bryson', 'Field Notes', 'John Berger'][i],
  album: '', folder: 'On the bookshelf', filename: `${title}.m4b`, kind: i === 2 ? 'audio' : 'audiobook',
  duration: 7200 + i * 900, sources: [{ ...device, device_id: device.id, local: true, online: true }], chapters: [],
  progress: i === 0 ? { position: 2100, completed: false, updated_at: 1 } : null,
}));
const library = { device, peers: [], items, queue: { items: items.slice(0, 3).map(i => i.id), current: items[0].id }, podcasts: {
  hosts: [{ device, desktop: true, online: true }], jobs: [], episodes: [],
  shows: ['The Quiet Hours', 'Good Things Take Time', 'Outside, Together'].map((title, i) => ({ id: `show-${i}`, title, author: 'Hopper studio', owner: device })),
} };
let failRefresh = false, queued, directorySubscription;
const server = createServer(async (request, response) => {
  const path = new URL(request.url, 'http://localhost').pathname;
  if (path === '/favicon.ico') { response.writeHead(204); response.end(); return; }
  if (path.startsWith('/api/')) {
    response.setHeader('Content-Type', 'application/json');
    if (path === '/api/podcasts/directory') {
      let body = ''; for await (const chunk of request) body += chunk;
      const {term, category, provider} = JSON.parse(body);
      response.end(JSON.stringify({categories:[{id:'1',name:'Fiction'}], shows:[{
        title:[provider,category,term].filter(Boolean).join(' · '), author:'Adventure crew', url:'https://example.org/feed', artwork:null,
      }]})); return;
    }
    if (path === '/api/add-podcasts') {
      let body=''; for await (const chunk of request) body+=chunk;
      directorySubscription=JSON.parse(body); response.end(JSON.stringify({delivery_ids:['test'],warnings:[]})); return;
    }
    if (path === '/api/queue' && request.method === 'POST') {
      let body = ''; for await (const chunk of request) body += chunk; queued = JSON.parse(body);
    }
    if (path.startsWith('/api/play/')) { response.end(JSON.stringify({path: '/test-audio.wav'})); return; }
    if (path === '/api/refresh' && failRefresh) { response.writeHead(400); response.end(JSON.stringify({ error: 'Preview connection unavailable' })); return; }
    response.end(JSON.stringify(path === '/api/library' || path === '/api/refresh' ? library : path === '/api/folders' ? {folders: [], imports: [], can_choose: true} : {})); return;
  }
  if (path === '/test-audio.wav') {
    const wav = Buffer.alloc(44 + 16000);
    wav.write('RIFF'); wav.writeUInt32LE(wav.length - 8, 4); wav.write('WAVEfmt ', 8);
    wav.writeUInt32LE(16, 16); wav.writeUInt16LE(1, 20); wav.writeUInt16LE(1, 22);
    wav.writeUInt32LE(8000, 24); wav.writeUInt32LE(16000, 28); wav.writeUInt16LE(2, 32);
    wav.writeUInt16LE(16, 34); wav.write('data', 36); wav.writeUInt32LE(16000, 40);
    response.writeHead(200, {'Content-Type': 'audio/wav'}); response.end(wav); return;
  }
  const file = path === '/' ? 'index.html' : path.slice(1);
  if (!/^[a-z-]+\.(html|js|css)$/.test(file)) { response.writeHead(404); response.end(); return; }
  try {
    response.setHeader('Content-Type', file.endsWith('.js') ? 'text/javascript' : file.endsWith('.css') ? 'text/css' : 'text/html');
    response.setHeader('Content-Security-Policy', "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data: https: http:; media-src 'self' blob:; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'");
    response.end(await readFile(join(ui, file)));
  } catch { response.writeHead(404); response.end(); }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const origin = `http://127.0.0.1:${server.address().port}`;
const profile = await mkdtemp(join(tmpdir(), 'hopper-glass-chrome-'));
const chrome = spawn(process.env.CHROME_BIN || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', [
  '--headless=new', '--no-first-run', '--no-default-browser-check', '--disable-gpu', '--remote-debugging-port=0',
  `--user-data-dir=${profile}`, 'about:blank',
], { stdio: 'ignore' });
let socket;
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
const waitFor = async (check, description) => {
  for (let attempt = 0; attempt < 120; attempt++) { if (await check()) return; await delay(100); }
  throw new Error(`Timed out: ${description}`);
};
try {
  let port;
  await waitFor(async () => { try { port = (await readFile(join(profile, 'DevToolsActivePort'), 'utf8')).split('\n')[0]; return true; } catch { return false; } }, 'Chrome startup');
  const target = await (await fetch(`http://127.0.0.1:${port}/json/new?${origin}`, { method: 'PUT' })).json();
  socket = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => { socket.onopen = resolve; socket.onerror = reject; });
  let nextId = 0; const pending = new Map(), errors = [];
  socket.onmessage = event => {
    const message = JSON.parse(event.data);
    if (message.method === 'Runtime.exceptionThrown') errors.push(message.params.exceptionDetails.text);
    if (message.method === 'Log.entryAdded' && message.params.entry.level === 'error'
      && !(failRefresh && message.params.entry.url?.endsWith('/api/refresh'))) errors.push(message.params.entry.text);
    if (message.id) { const callback = pending.get(message.id); pending.delete(message.id); message.error ? callback.reject(message.error) : callback.resolve(message.result); }
  };
  const call = (method, params = {}) => new Promise((resolve, reject) => { const id = ++nextId; pending.set(id, { resolve, reject }); socket.send(JSON.stringify({ id, method, params })); });
  const evaluate = async expression => {
    const result = await call('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
    if (result.exceptionDetails) throw new Error(JSON.stringify(result.exceptionDetails));
    return result.result.value;
  };
  await call('Runtime.enable'); await call('Log.enable'); await call('Page.enable');
  await call('Emulation.setDeviceMetricsOverride', { width: 1180, height: 820, deviceScaleFactor: 1, mobile: false });
  await call('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-color-scheme', value: 'light' }] });
  await call('Page.navigate', { url: origin });
  await waitFor(() => evaluate("document.querySelectorAll('.media-row').length === 4 && !!document.querySelector('.continue-card')?.style.getPropertyValue('--glass-lens')"), 'catalogue and decoded lens');
  const output = process.env.HOPPER_SCREENSHOTS || join(ui, '..', 'artifacts', 'glass');
  await mkdir(output, { recursive: true });
  const screenshot = async name => { await delay(350); const { data } = await call('Page.captureScreenshot', { format: 'png' }); await writeFile(join(output, `${name}.png`), Buffer.from(data, 'base64')); };
  await screenshot('library-light');
  const geometry = await evaluate("JSON.stringify([...document.querySelectorAll('.sidebar,.player,.continue-card,.media-row')].map(e => {const r=e.getBoundingClientRect();return [r.x,r.y,r.width,r.height]}))");
  await evaluate("document.getElementById('appearance').click(); document.getElementById('appearance-mode').value='dark'; document.getElementById('appearance-mode').dispatchEvent(new Event('change'))");
  await waitFor(() => evaluate("document.documentElement.dataset.theme === 'dark'"), 'dark appearance');
  await screenshot('appearance-dark');
  await evaluate("document.getElementById('dark-tint').value='1.35'; document.getElementById('dark-tint').dispatchEvent(new Event('input')); document.getElementById('dialog').close()");
  await screenshot('library-dark');
  assert.equal(await evaluate("JSON.stringify([...document.querySelectorAll('.sidebar,.player,.continue-card,.media-row')].map(e => {const r=e.getBoundingClientRect();return [r.x,r.y,r.width,r.height]}))"), geometry, 'appearance must preserve geometry');
  await call('Page.reload');
  await waitFor(() => evaluate("document.querySelectorAll('.media-row').length === 4 && document.documentElement.dataset.theme === 'dark'"), 'saved appearance after reload');
  assert.equal(await evaluate("JSON.parse(localStorage.getItem('hopper-appearance')).dark"), 1.35);
  assert.equal(await evaluate("JSON.parse(localStorage.getItem('hopper-appearance')).light"), 1);
  await evaluate("document.querySelector('[data-view=podcasts]').click()");
  await waitFor(() => evaluate("[...document.querySelectorAll('.podcast-card')].every(e => e.style.getPropertyValue('--glass-lens'))"), 'podcast lenses');
  await screenshot('podcasts-dark');
  await evaluate("document.querySelector('[data-podcast-browse]').click()");
  await waitFor(() => evaluate("document.querySelector('.directory-row strong')?.textContent === 'apple'"), 'default Apple chart');
  await evaluate("{const f=document.getElementById('directory-search'); f.elements.provider.value='podcast_index'; f.elements.provider.dispatchEvent(new Event('change'))}");
  await waitFor(() => evaluate("document.querySelector('.directory-row strong')?.textContent === 'podcast_index'"), 'Podcast Index chart');
  await evaluate("{const f=document.getElementById('directory-search'); f.elements.term.value='dragons'; f.elements.category.value='1'; f.requestSubmit()}");
  await waitFor(() => evaluate("document.querySelector('.directory-row strong')?.textContent === 'podcast_index · 1 · dragons'"), 'filtered directory search');
  await evaluate("document.querySelector('[data-listing]').click()");
  await waitFor(() => directorySubscription?.url === 'https://example.org/feed', 'directory subscription');
  assert.equal(directorySubscription.owner_id, device.id);
  await evaluate("document.getElementById('dialog').close()");
  assert.equal(await evaluate("new Set([...document.querySelectorAll('.podcast-card')].map(e=>e.dataset.glassInk)).size"), 1);
  // Dialogs are deliberately excluded from SVG refraction.
  await evaluate("document.getElementById('appearance').click()");
  assert.equal(await evaluate("document.querySelector('dialog').style.getPropertyValue('--glass-lens')"), '');
  await evaluate("document.getElementById('appearance-mode').value='light'; document.getElementById('appearance-mode').dispatchEvent(new Event('change')); document.getElementById('dialog').close()");
  await screenshot('podcasts-light');
  for (const features of [
    [{ name: 'prefers-reduced-transparency', value: 'reduce' }],
    [{ name: 'prefers-contrast', value: 'more' }],
    [{ name: 'forced-colors', value: 'active' }],
  ]) {
    await call('Emulation.setEmulatedMedia', { features });
    await waitFor(() => evaluate("document.querySelectorAll('.glass-filter').length === 0"), features[0].name);
    assert.equal(await evaluate("getComputedStyle(document.querySelector('.player')).backdropFilter"), 'none');
    if (features[0].name === 'forced-colors') assert.equal(await evaluate("getComputedStyle(document.querySelector('.player')).backgroundColor"), await evaluate("getComputedStyle(document.body).backgroundColor"));
  }
  await call('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-color-scheme', value: 'light' }] });
  await evaluate("document.getElementById('add').click()"); await screenshot('import-light');
  await evaluate("document.getElementById('dialog').close()");
  failRefresh = true; await evaluate("document.getElementById('refresh').click()");
  await waitFor(() => evaluate("document.getElementById('toast').classList.contains('error')"), 'error toast');
  await delay(400);
  assert.equal(await evaluate("getComputedStyle(document.getElementById('toast')).backgroundColor"), 'rgb(147, 92, 68)');
  // Responsive layout and lens cleanup after rerender/resize.
  failRefresh = false;
  await call('Emulation.setDeviceMetricsOverride', { width: 800, height: 700, deviceScaleFactor: 1, mobile: false });
  await delay(400); assert.equal(await evaluate("getComputedStyle(document.querySelector('.queue-panel')).display"), 'none');
  await screenshot('compact-light');
  await evaluate("document.querySelector('[data-view=all]').click()"); await delay(400);
  assert.equal(await evaluate("document.querySelectorAll('.glass-filter').length"), 1);
  await call('Network.setUserAgentOverride', { userAgent: 'Mozilla/5.0 AppleWebKit/605.1.15 Version/17.0 Safari/605.1.15' });
  await call('Page.reload');
  await waitFor(() => evaluate("document.querySelectorAll('.media-row').length === 4 && document.querySelector('.player').classList.contains('glass-surface')"), 'CSS fallback');
  await delay(300);
  assert.equal(await evaluate("document.querySelectorAll('.glass-filter').length"), 0);
  assert.ok((await evaluate("getComputedStyle(document.querySelector('.player')).backdropFilter")).includes('blur(2px)'));
  await screenshot('library-css-fallback');
  // A background native webview may defer rAF indefinitely. Its first palette must still be complete.
  await call('Page.addScriptToEvaluateOnNewDocument', { source: 'window.requestAnimationFrame = () => 0;' });
  await call('Page.reload');
  await waitFor(() => evaluate("document.querySelectorAll('.media-row').length === 4"), 'background-window startup');
  assert.equal(await evaluate("document.querySelectorAll('.sidebar.glass-surface,.queue-panel.glass-surface,.player.glass-surface').length"), 3);
  assert.equal(await evaluate("document.documentElement.dataset.theme"), 'light');
  library.items = Array.from({ length: 4800 }, (_, index) => ({
    ...items[0], id: index.toString(16).padStart(64, '0'), title: `Episode ${index % 120 + 1}`,
    folder: `Archive/Plot ${Math.floor(index / 240) + 1}/Season ${Math.floor(index / 120) % 2 + 1}`,
    progress: null
  }));
  await call('Page.reload');
  await waitFor(() => evaluate("document.getElementById('library-count').textContent === '4800'"), 'large library');
  await evaluate("document.querySelector('[data-view=folders]').click()");
  await waitFor(() => evaluate("!!document.querySelector('[data-folder=Archive]')"), 'large folder library');
  assert.equal(await evaluate("document.querySelectorAll('.media-row').length"), 0);
  await evaluate("document.querySelector('[data-folder=Archive]').click()");
  await waitFor(() => evaluate("document.querySelectorAll('.folder-children button').length === 20"), 'plotlines');
  await evaluate("document.querySelector('[data-folder=\"Archive/Plot 2\"]').click()");
  await waitFor(() => evaluate("document.querySelectorAll('.folder-children button').length === 2"), 'seasons');
  await evaluate("document.querySelector('[data-folder=\"Archive/Plot 2/Season 1\"]').click()");
  await waitFor(() => evaluate("document.querySelectorAll('.media-row').length === 100"), 'bounded episode list');
  await evaluate("document.querySelector('[data-more]').click()");
  await waitFor(() => evaluate("document.querySelectorAll('.media-row').length === 120"), 'remaining episodes');
  assert.equal(await evaluate("document.querySelector('.media-row strong').textContent"), 'Episode 1');
  await screenshot('nested-archive');
  const season = library.items.filter(item => item.folder === 'Archive/Plot 2/Season 1');
  await evaluate("document.querySelectorAll('.media-row .media-main')[5].click()");
  await waitFor(() => queued?.current === season[5].id, 'starting at the selected episode');
  assert.deepEqual(queued.items, season.map(item => item.id), 'folder playback must exclude sibling plotlines and seasons');
  await evaluate("document.getElementById('next').click()");
  await waitFor(() => queued?.current === season[6].id, 'next episode in the same season');
  assert.deepEqual(errors, [], `Browser errors: ${errors.join(', ')}`);
  console.log(`Browser checks passed. Screenshots: ${output}`);
} finally {
  socket?.close(); chrome.kill(); server.close();
  await delay(300); await rm(profile, { recursive: true, force: true }).catch(() => {});
}
