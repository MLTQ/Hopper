let token = location.hash.slice(1) || sessionStorage.getItem('hopper-token');
if (token) sessionStorage.setItem('hopper-token', token);
history.replaceState(null, '', location.pathname);

export async function api(path, body, method) {
  const multipart = body instanceof FormData;
  const response = await fetch(`api/${path}`, {
    method: method || (body === undefined ? 'GET' : 'POST'),
    headers: { Authorization: `Bearer ${token || ''}`, ...(!multipart && body !== undefined ? { 'Content-Type': 'application/json' } : {}) },
    body: body === undefined ? undefined : multipart ? body : JSON.stringify(body),
  });
  const result = await response.json().catch(() => ({}));
  if (!response.ok) throw new Error(result.error || `Request failed (${response.status})`);
  return result;
}

export const escape = value => String(value ?? '').replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]);
export function time(seconds) {
  if (!Number.isFinite(seconds)) return '–:––';
  seconds = Math.max(0, Math.floor(seconds));
  const hours = Math.floor(seconds / 3600), minutes = Math.floor(seconds / 60) % 60;
  return hours ? `${hours}:${String(minutes).padStart(2, '0')}:${String(seconds % 60).padStart(2, '0')}` : `${minutes}:${String(seconds % 60).padStart(2, '0')}`;
}
export function duration(seconds) {
  if (!seconds) return 'Duration unknown';
  return seconds >= 3600 ? `${Math.floor(seconds / 3600)} hr ${Math.round(seconds % 3600 / 60)} min` : `${Math.max(1, Math.round(seconds / 60))} min`;
}
export const tone = id => `tone-${parseInt(id.slice(0, 2), 16) % 6}`;
export const initials = item => item.title.split(/\s+/).slice(0, 2).map(s => s[0]).join('').toUpperCase();
