// Progressive enhancement only: WebKit and Firefox retain the CSS material.
const tiles = new Map();
let nextId = 0;
const svgNS = 'http://www.w3.org/2000/svg';
const node = (name, attributes) => {
  const element = document.createElementNS(svgNS, name);
  for (const [key, value] of Object.entries(attributes)) element.setAttribute(key, value);
  return element;
};

function rimTile(radius, rim) {
  const key = `${radius}:${rim}`;
  if (tiles.has(key)) return tiles.get(key);
  const edge = Math.ceil(Math.max(radius, rim)) + 2, size = edge * 2 + 1;
  const canvas = document.createElement('canvas'); canvas.width = canvas.height = size;
  const context = canvas.getContext('2d'), pixels = context.createImageData(size, size);
  const maxOffset = 1.1 * rim;
  for (let y = 0; y < size; y++) for (let x = 0; x < size; x++) {
    const px = x + .5 - size / 2, py = y + .5 - size / 2;
    const qx = Math.abs(px) - (size / 2 - radius), qy = Math.abs(py) - (size / 2 - radius);
    const ox = Math.max(qx, 0), oy = Math.max(qy, 0), length = Math.hypot(ox, oy);
    const distance = -(length + Math.min(Math.max(qx, qy), 0) - radius);
    let nx = 0, ny = 0;
    if (length > 0) { nx = -Math.sign(px) * ox / length; ny = -Math.sign(py) * oy / length; }
    else if (qx > qy) nx = -Math.sign(px); else ny = -Math.sign(py);
    const bend = distance >= 0 && distance < rim ? Math.pow(1 - distance / rim, 2.6) : 0;
    const index = (y * size + x) * 4;
    pixels.data[index] = Math.round(128 + 127 * nx * bend);
    pixels.data[index + 1] = Math.round(128 + 127 * ny * bend);
    pixels.data[index + 2] = 128; pixels.data[index + 3] = 255;
  }
  context.putImageData(pixels, 0, 0);
  const tile = { canvas, edge, maxOffset };
  if (tiles.size >= 24) tiles.delete(tiles.keys().next().value);
  tiles.set(key, tile); return tile;
}

// Nine-slice cached corner/rim geometry. Resize moves the edges; it never recomputes the field.
function mapImage(width, height, radius, rim) {
  const tile = rimTile(radius, rim), canvas = document.createElement('canvas');
  canvas.width = width; canvas.height = height;
  const context = canvas.getContext('2d'), edge = Math.min(tile.edge, Math.floor(Math.min(width, height) / 2));
  const source = [0, tile.edge, tile.edge + 1], sourceSize = [tile.edge, 1, tile.edge];
  const xs = [0, edge, width - edge], ys = [0, edge, height - edge];
  const ws = [edge, width - edge * 2, edge], hs = [edge, height - edge * 2, edge];
  for (let y = 0; y < 3; y++) for (let x = 0; x < 3; x++) {
    if (ws[x] && hs[y]) context.drawImage(tile.canvas, source[x], source[y], sourceSize[x], sourceSize[y], xs[x], ys[y], ws[x], hs[y]);
  }
  return { url: canvas.toDataURL(), scale: tile.maxOffset * 2 };
}

export class GlassLens {
  constructor() {
    this.entries = new Map();
    this.supported = /Chrome|Chromium|Edg\//.test(navigator.userAgent)
      && !/Firefox/.test(navigator.userAgent) && CSS.supports('backdrop-filter', 'url(#hopper-lens)');
  }
  async update(element, disabled = false) {
    const bounds = element.getBoundingClientRect(), style = getComputedStyle(element);
    const eligible = this.supported && !disabled && element.matches('.continue-card,.podcast-card')
      && Math.min(bounds.width, bounds.height) >= 32 && style.willChange === 'auto'
      && style.transform === 'none' && style.filter === 'none';
    if (!eligible) { this.remove(element); return; }
    const width = Math.round(bounds.width), height = Math.round(bounds.height);
    const radius = Math.min(parseFloat(style.borderTopLeftRadius) || 0, width / 2, height / 2);
    const rim = Math.min(44, Math.max(4, Math.min(width, height) / 3.5));
    const key = `${width}:${height}:${radius}:${rim}`;
    if (this.entries.get(element)?.key === key) return;
    this.remove(element);
    const entry = { key, svg: null }; this.entries.set(element, entry);
    const map = mapImage(width, height, radius, rim), decoded = new Image(); decoded.src = map.url;
    try { await decoded.decode(); } catch { this.remove(element); return; }
    if (this.entries.get(element) !== entry || !element.isConnected) return;
    const id = `hopper-rim-${++nextId}`, svg = node('svg', { class: 'glass-filter', 'aria-hidden': 'true', width: 0, height: 0 });
    const filter = node('filter', { id, filterUnits: 'userSpaceOnUse', x: 0, y: 0, width, height, 'color-interpolation-filters': 'sRGB' });
    filter.append(node('feImage', { href: map.url, x: 0, y: 0, width, height, result: 'rim', preserveAspectRatio: 'none' }));
    for (const [channel, factor, values] of [
      ['red', 1.15, '1 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 1 0'],
      ['green', 1, '0 0 0 0 0  0 1 0 0 0  0 0 0 0 0  0 0 0 1 0'],
      ['blue', .85, '0 0 0 0 0  0 0 0 0 0  0 0 1 0 0  0 0 0 1 0'],
    ]) {
      filter.append(node('feDisplacementMap', { in: 'SourceGraphic', in2: 'rim', scale: map.scale * factor, xChannelSelector: 'R', yChannelSelector: 'G', result: `${channel}-bend` }));
      filter.append(node('feColorMatrix', { in: `${channel}-bend`, type: 'matrix', values, result: channel }));
    }
    filter.append(node('feBlend', { in: 'red', in2: 'green', mode: 'screen', result: 'rg' }));
    filter.append(node('feBlend', { in: 'rg', in2: 'blue', mode: 'screen' }));
    svg.append(filter); document.body.append(svg); entry.svg = svg;
    element.style.setProperty('--glass-lens', `url(#${id})`);
  }
  remove(element) {
    this.entries.get(element)?.svg?.remove(); this.entries.delete(element);
    element.style.removeProperty('--glass-lens');
  }
  prune() { for (const element of this.entries.keys()) if (!element.isConnected) this.remove(element); }
}
