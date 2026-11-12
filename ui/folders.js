import { api, escape as esc } from './api.js';

export async function sharedFolders(dialog, toast, load) {
  const { folders, can_choose } = await api('folders');
  dialog('Shared folders', `<p>Choose a folder on this desktop to stream its media to your paired devices. Hopper checks for changes while it is open.</p>
    ${can_choose ? '<button class="primary" id="pick-share">Choose folder…</button>' : ''}
    <details ${can_choose ? '' : 'open'}><summary>Enter a folder path</summary><form id="share-path"><input name="path" required placeholder="Full folder path" aria-label="Full folder path"><button class="secondary">Share folder</button></form></details>
    <div aria-live="polite" id="folder-status"></div>
    ${folders.map(f => `<section class="pair-section"><strong>${esc(f.path)}</strong><p>${f.files} media item${f.files === 1 ? '' : 's'}${f.warning ? ' · ' + esc(f.warning) : ''}</p><button class="secondary" data-scan-root="${f.id}">Rescan</button> <button class="text-button" data-stop-root="${f.id}">Stop sharing</button></section>`).join('')}`);
  const form = document.getElementById('share-path');
  const panel = document.getElementById('dialog-body');
  const status = document.getElementById('folder-status');
  const run = async (message, action) => {
    const controls = [...panel.querySelectorAll('button,input')];
    controls.forEach(el => el.disabled = true);
    status.textContent = message;
    try {
      await action();
      await load();
      if (status.isConnected && document.getElementById('dialog').open) await sharedFolders(dialog, toast, load);
    } catch (error) { toast(error.message, true); }
    finally { controls.forEach(el => el.disabled = false); status.textContent = ''; }
  };
  if (can_choose) document.getElementById('pick-share').onclick = () =>
    run('Choose a folder to index its media…', () => api('folders/choose', {}));
  form.onsubmit = event => {
    event.preventDefault();
    const path = new FormData(form).get('path');
    run('Reading media files…', () => api('folders', { path }));
  };
  panel.querySelectorAll('[data-scan-root]').forEach(button => {
    button.onclick = () => run('Checking folder…', () => api(`folders/${button.dataset.scanRoot}/rescan`, {}));
  });
  panel.querySelectorAll('[data-stop-root]').forEach(button => {
    button.onclick = () => run('Removing shared folder…', () => api(`folders/${button.dataset.stopRoot}`, undefined, 'DELETE'));
  });
}
