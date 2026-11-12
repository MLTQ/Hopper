import { api, escape as esc } from './api.js';

export function pairDevices(dialog, toast, load) {
  dialog('Pair a device', `<section class="pair-section"><button id="make-invite" class="primary">Create pairing QR code</button><div id="invitation" aria-live="polite"></div></section>
    <details><summary>Pair with an invitation from another device</summary><form id="pair-form"><textarea name="code" placeholder="Paste hopper1:…" rows="3" required aria-label="Pairing invitation"></textarea><button class="secondary">Pair this device</button></form></details>
    <p class="hint">Paired devices can browse and play this desktop’s Hopper library.</p>`);
  const button = document.getElementById('make-invite');
  const output = document.getElementById('invitation');
  let expires;
  button.onclick = async () => {
    button.disabled = true;
    try {
      const result = await api('invite/qr', {});
      clearTimeout(expires);
      output.innerHTML = `<img class="pair-qr" alt="One-use Hopper pairing QR code"><p>On iPhone, tap <strong>Scan pairing QR code</strong> in Hopper. This invitation expires in 10 minutes.</p><details><summary>Copy or paste invitation</summary><textarea readonly rows="3" aria-label="Your invitation">${esc(result.code)}</textarea><button class="secondary" id="copy-invite">Copy invitation</button></details>`;
      output.querySelector('img').src = result.image;
      document.getElementById('copy-invite').onclick = async () => {
        try { await navigator.clipboard.writeText(result.code); toast('Invitation copied'); }
        catch { output.querySelector('textarea').select(); }
      };
      button.textContent = 'Create another code';
      expires = setTimeout(() => { output.textContent = 'This invitation expired. Create another code to pair.'; }, result.expires_in * 1000);
    } catch (error) { toast(error.message, true); }
    finally { button.disabled = false; }
  };
  document.getElementById('pair-form').onsubmit = async event => {
    event.preventDefault();
    const submit = event.target.querySelector('button');
    submit.disabled = true;
    try {
      await api('pair', { code: new FormData(event.target).get('code') });
      document.getElementById('dialog').close();
      await load();
      toast('Your devices are paired');
    } catch (error) { toast(error.message, true); }
    finally { submit.disabled = false; }
  };
}
