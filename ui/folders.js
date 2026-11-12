import { api, escape as esc } from './api.js';

export async function sharedFolders(dialog, toast, load, onShared) {
  const { folders, can_choose } = await api('folders');
  dialog('Shared folders', `<p>Choose a folder on this desktop to stream its media to your paired devices. Hopper checks for changes while it is open.</p>
    ${can_choose ? '<button class="primary" id="pick-share">Choose folder…</button>' : ''}
    <details ${can_choose ? '' : 'open'}><summary>Enter a folder path</summary><form id="share-path"><input name="path" required placeholder="Full folder path" aria-label="Full folder path"><button class="secondary">Share folder</button></form></details>
    <div aria-live="polite" id="folder-status"></div>
    ${folders.map(f => `<section class="pair-section"><strong>${esc(f.path)}</strong><p>${f.files} media item${f.files === 1 ? '' : 's'}${f.warning ? ' · ' + esc(f.warning) : ''}</p><button class="secondary" data-scan-root="${f.id}">Rescan</button> <button class="text-button" data-stop-root="${f.id}">Stop sharing</button></section>`).join('')}`);
  const form = document.getElementById('share-path');
  const panel = document.getElementById('dialog-body');
  const status = document.getElementById('folder-status');
  const run = async (message, action, adding = false) => {
    const controls = [...panel.querySelectorAll('button,input')];
    controls.forEach(el => el.disabled = true);
    status.textContent = message;
    try {
      const result = await action();
      await load();
      if (adding && result?.path && status.isConnected && document.getElementById('dialog').open) {
        onShared(result.path.split(/[\\/]/).filter(Boolean).pop() || '');
        return;
      }
      if (status.isConnected && document.getElementById('dialog').open) await sharedFolders(dialog, toast, load, onShared);
    } catch (error) { toast(error.message, true); }
    finally { controls.forEach(el => el.disabled = false); status.textContent = ''; }
  };
  if (can_choose) document.getElementById('pick-share').onclick = () =>
    run('Choose a folder to index its media…', () => api('folders/choose', {}), true);
  form.onsubmit = event => {
    event.preventDefault();
    const path = new FormData(form).get('path');
    run('Reading media files…', () => api('folders', { path }), true);
  };
  panel.querySelectorAll('[data-scan-root]').forEach(button => {
    button.onclick = () => run('Checking folder…', () => api(`folders/${button.dataset.scanRoot}/rescan`, {}));
  });
  panel.querySelectorAll('[data-stop-root]').forEach(button => {
    button.onclick = () => run('Removing shared folder…', () => api(`folders/${button.dataset.stopRoot}`, undefined, 'DELETE'));
  });
}

export function addedFolders(panel, dialog, toast, load) {
  let revision = 0;
  let rows = [];
  let lastMarkup;

  async function refresh() {
    const request = ++revision;
    if (!panel.hasChildNodes()) panel.textContent = 'Loading folders…';
    try {
      const { folders, imports } = await api('folders');
      if (request !== revision) return;
      rows = [
        ...folders.map(folder => ({ ...folder, label: folder.path, shared: true })),
        ...imports.map(folder => ({ ...folder, label: folder.name, shared: false })),
      ].sort((a, b) => a.label.localeCompare(b.label));
      const markup = rows.length ? '<ul>' + rows.map((row, index) => {
        const count = row.files === 1 ? '1 item' : `${row.files} items`;
        return `<li><div><strong>${esc(row.label)}</strong><small>${row.shared ? 'Shared' : 'Imported'} · ${count}${row.warning ? ' · ' + esc(row.warning) : ''}</small></div>
          <button class="secondary" data-remove-folder="${index}" aria-label="Remove ${row.shared ? 'shared' : 'imported'} folder ${esc(row.label)}">Remove</button></li>`;
      }).join('') + '</ul>' : '<p>No folders added on this desktop.</p>';
      if (markup !== lastMarkup) panel.innerHTML = markup;
      lastMarkup = markup;
    } catch (error) {
      if (request !== revision) return;
      lastMarkup = undefined;
      panel.innerHTML = `<p>${esc(error.message)} <button class="text-button" data-retry-folders>Retry</button></p>`;
    }
  }

  panel.addEventListener('click', event => {
    if (event.target.closest('[data-retry-folders]')) { refresh(); return; }
    const button = event.target.closest('[data-remove-folder]');
    if (!button) return;
    const row = rows[Number(button.dataset.removeFolder)];
    const effect = row.shared ? 'Hopper will stop scanning and sharing this folder.'
      : 'Hopper will remove this folder’s imported items from this desktop’s library.';
    dialog('Remove folder?', `<p class="folder-path"><strong>${esc(row.label)}</strong></p><p>${effect} Your files stay on disk. Items with another source remain available.</p>
      <div class="detail-actions"><button class="secondary" id="cancel-folder">Cancel</button><button class="primary" id="remove-folder">Remove folder</button></div>`);
    const modal = document.getElementById('dialog');
    document.getElementById('cancel-folder').onclick = () => modal.close();
    document.getElementById('cancel-folder').focus();
    const remove = document.getElementById('remove-folder');
    remove.onclick = async () => {
      remove.disabled = true;
      try {
        await api(`${row.shared ? 'folders' : 'imported-folders'}/${encodeURIComponent(row.id)}`, undefined, 'DELETE');
        if (remove.isConnected) modal.close();
        await load();
        toast('Folder removed');
      } catch (error) { toast(error.message, true); }
      finally { remove.disabled = false; }
    };
  });
  return refresh;
}

export function folderLevel(items, path) {
  const prefix = path ? path + '/' : '';
  const children = new Map(), files = [];
  for (const item of items) {
    if (item.folder === path) { files.push(item); continue; }
    if (!item.folder.startsWith(prefix)) continue;
    const name = item.folder.slice(prefix.length).split('/', 1)[0];
    if (name) children.set(name, (children.get(name) || 0) + 1);
  }
  return { files, children: [...children].sort(([a], [b]) => a.localeCompare(b, undefined, { numeric: true }))
    .map(([name, count]) => ({ name, path: prefix + name, count })) };
}

export function folderTrail(path) {
  const parts = path.split('/').filter(Boolean);
  let html = '<button class="text-button" data-folder="">All folders</button>';
  parts.forEach((part, i) => {
    html += ` › <button class="text-button" data-folder="${esc(parts.slice(0, i + 1).join('/'))}">${esc(part)}</button>`;
  });
  return '<nav aria-label="Folder path">' + html + '</nav>';
}
