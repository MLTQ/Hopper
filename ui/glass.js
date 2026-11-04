import { GlassLens } from './glass-lens.js';

const surfaces = '.sidebar,.queue-panel,.player,.continue-card,.podcast-card,dialog,#toast:not(.error),#drop-overlay,#close-video';
const preferenceKey = 'hopper-appearance';
const clamp = (value, min = 0, max = 1) => Math.min(max, Math.max(min, value));
const luminance = rgb => (.299 * rgb[0] + .587 * rgb[1] + .114 * rgb[2]) / 255;
const relativeLuminance = rgb => rgb.map(c => c / 255).map(c => c <= .04045 ? c / 12.92 : ((c + .055) / 1.055) ** 2.4).reduce((sum, c, i) => sum + c * [.2126, .7152, .0722][i], 0);
const contrast = (a, b) => (Math.max(relativeLuminance(a), relativeLuminance(b)) + .05) / (Math.min(relativeLuminance(a), relativeLuminance(b)) + .05);

export function materialFor(samples, dark, previousInk, tint = 1, dense = false) {
  const sorted = [...samples].sort((a, b) => luminance(a) - luminance(b));
  const trimmed = sorted.length > 4 ? sorted.slice(2, -2) : sorted;
  const average = [0, 1, 2].map(i => trimmed.reduce((sum, rgb) => sum + rgb[i], 0) / trimmed.length);
  const light = luminance(average), texture = luminance(sorted[sorted.length - 1]) - luminance(sorted[0]);
  const chroma = (Math.max(...average) - Math.min(...average)) / 255;
  const threshold = previousInk === 'light' ? .55 : previousInk === 'dark' ? .45 : .5;
  const ink = dark || light < threshold ? 'light' : 'dark';
  const density = dense ? 1 : clamp(.55 * (ink === 'light' ? light : 1 - light) + .7 * texture + .3 * chroma);
  const min = dark ? .16 : ink === 'light' ? .1 : .06, max = dark ? .38 : ink === 'light' ? .3 : .22;
  const color = ink === 'light' ? [20, 20, 24] : [255, 255, 255];
  const secondary = ink === 'light' ? [186, 196, 188] : [80, 91, 83];
  const brightness = ink === 'light' ? dark ? .8 : .85 : 1 + .06 * Math.min(tint, 1);
  const saturation = (dark ? 1.6 : ink === 'light' ? 1.7 : 1.8) * Math.max(.45, 1 - .75 * chroma);
  let alpha = (dense ? .92 : min + (max - min) * density) * tint;
  // Raise density before sacrificing ink. Check every sample, including the trimmed extremes.
  const passes = value => samples.every(rgb => {
    const gray = rgb.reduce((sum, c, i) => sum + c * [.213, .715, .072][i], 0);
    const filtered = rgb.map(c => clamp((gray + saturation * (c - gray)) * brightness, 0, 255));
    const background = filtered.map((c, i) => c * (1 - value) + color[i] * value);
    // The top-left highlight is the brightest part of dark glass.
    const highlighted = background.map(c => c * (ink === 'light' ? .9 : .7) + 255 * (ink === 'light' ? .1 : .3));
    return Math.min(contrast(secondary, background), contrast(secondary, highlighted)) >= 4.5;
  });
  const ceiling = Math.min(.96, .96 * tint);
  while (alpha < ceiling && !passes(alpha)) alpha = Math.min(ceiling, alpha + .02);
  return { ink, density, alpha: Math.min(.96, alpha), brightness, saturation };
}

export class Glass {
  constructor() {
    this.preferences = { mode: 'system', light: 1, dark: 1 };
    this.rememberedInk = {};
    try {
      const saved = JSON.parse(localStorage.getItem(preferenceKey) || '{}');
      if (['system', 'light', 'dark'].includes(saved.mode)) this.preferences.mode = saved.mode;
      for (const theme of ['light', 'dark']) if (Number.isFinite(saved[theme])) this.preferences[theme] = clamp(saved[theme], 0, 1.5);
      if (saved.ink && typeof saved.ink === 'object' && !Array.isArray(saved.ink)) {
        this.rememberedInk = Object.fromEntries(Object.entries(saved.ink).filter(([, value]) => value === 'light' || value === 'dark'));
      }
    } catch { /* Storage can be disabled without preventing playback. */ }
    this.scheme = matchMedia('(prefers-color-scheme: dark)');
    this.reduced = matchMedia('(prefers-reduced-transparency: reduce)');
    this.contrast = matchMedia('(prefers-contrast: more)');
    this.forced = matchMedia('(forced-colors: active)');
    this.lens = new GlassLens();
    this.canvas = document.createElement('canvas');
    this.resizeObserver = new ResizeObserver(() => this.schedule());
    this.observed = new Set();
    this.mutationObserver = new MutationObserver(() => this.schedule());
    this.mutationObserver.observe(document.body, { childList: true, subtree: true, attributes: true, attributeFilter: ['hidden', 'open', 'class'] });
    this.scheme.addEventListener('change', () => this.applyTheme());
    for (const media of [this.reduced, this.contrast, this.forced]) media.addEventListener('change', () => this.schedule());
    window.addEventListener('resize', () => { clearTimeout(this.resizeTimer); this.resizeTimer = setTimeout(() => { this.drawScene(); this.schedule(); }, 140); });
    window.addEventListener('scroll', () => this.schedule(), { passive: true, capture: true });
    this.applyTheme();
  }
  save() {
    try { localStorage.setItem(preferenceKey, JSON.stringify({ ...this.preferences, ink: this.rememberedInk })); } catch { /* Optional local preference. */ }
  }
  applyTheme() {
    this.dark = this.preferences.mode === 'dark' || this.preferences.mode === 'system' && this.scheme.matches;
    document.documentElement.dataset.theme = this.dark ? 'dark' : 'light';
    // Background native webviews can suspend animation frames before their first display.
    this.drawScene(); this.refresh();
  }
  drawScene() {
    const width = Math.max(1, Math.ceil(innerWidth / 2)), height = Math.max(1, Math.ceil(innerHeight / 2));
    this.canvas.width = width; this.canvas.height = height;
    const context = this.canvas.getContext('2d', { willReadFrequently: true });
    context.fillStyle = this.dark ? '#101c18' : '#f7f8f2'; context.fillRect(0, 0, width, height);
    const pools = this.dark ? [
      [.01, .17, .68, '#45654f', '#45654f00'], [1, .32, .65, '#58634b', '#58634b00'],
      [.48, 1.12, .7, '#344f48', '#344f4800'], [.75, -.12, .58, '#544937', '#54493700'],
    ] : [
      [-.03, .12, .75, '#b4cdb7', '#b4cdb700'], [1.04, .3, .67, '#ccd9bc', '#ccd9bc00'],
      [.54, 1.12, .65, '#bfd3cc', '#bfd3cc00'], [.72, -.12, .54, '#ecd3b7', '#ecd3b700'],
    ];
    for (const [x, y, radius, start, end] of pools) {
      const gradient = context.createRadialGradient(x * width, y * height, 0, x * width, y * height, radius * Math.max(width, height));
      gradient.addColorStop(0, start); gradient.addColorStop(1, end);
      context.fillStyle = gradient; context.fillRect(0, 0, width, height);
    }
    document.body.style.setProperty('--scene-image', `url(${this.canvas.toDataURL()})`);
  }
  sample(bounds) {
    const context = this.canvas.getContext('2d'), samples = [];
    for (let y = 0; y < 3; y++) for (let x = 0; x < 5; x++) {
      const px = clamp(Math.floor((bounds.left + bounds.width * (x + .5) / 5) / innerWidth * this.canvas.width), 0, this.canvas.width - 1);
      const py = clamp(Math.floor((bounds.top + bounds.height * (y + .5) / 3) / innerHeight * this.canvas.height), 0, this.canvas.height - 1);
      samples.push([...context.getImageData(px, py, 1, 1).data].slice(0, 3));
    }
    return samples;
  }
  schedule() {
    if (this.frame) return;
    this.frame = requestAnimationFrame(() => { this.frame = 0; this.refresh(); });
  }
  refresh() {
    this.lens.prune();
    for (const element of this.observed) if (!element.isConnected) { this.resizeObserver.unobserve(element); this.observed.delete(element); }
    // Errors keep their established red surface rather than inheriting glass ink.
    document.querySelectorAll('#toast.error.glass-surface').forEach(element => element.classList.remove('glass-surface'));
    const disabled = this.reduced.matches || this.contrast.matches || this.forced.matches;
    for (const element of document.querySelectorAll(surfaces)) {
      if (!element.classList.contains('glass-surface')) element.classList.add('glass-surface');
      if (!this.observed.has(element)) { this.observed.add(element); this.resizeObserver.observe(element); }
      const bounds = element.getBoundingClientRect();
      if (!bounds.width || !bounds.height) { this.lens.remove(element); continue; }
      const group = element.closest('.podcast-grid') || element;
      const key = element.id || element.classList[0];
      const material = materialFor(this.sample(group.getBoundingClientRect()), this.dark, this.rememberedInk[key], this.preferences[this.dark ? 'dark' : 'light'], element.matches('dialog,#close-video'));
      element.dataset.glassInk = material.ink;
      this.rememberedInk[key] = material.ink;
      for (const token of ['alpha', 'brightness', 'saturation']) element.style.setProperty(`--glass-${token}`, material[token]);
      this.lens.update(element, disabled);
    }
    clearTimeout(this.saveTimer); this.saveTimer = setTimeout(() => this.save(), 300);
  }
  settings(dialog) {
    dialog('Appearance', `<form class="appearance-form"><label>Appearance<select id="appearance-mode"><option value="system">Follow system</option><option value="light">Light</option><option value="dark">Dark</option></select></label><label><span>Light glass tint <output id="light-tint-value"></output></span><input id="light-tint" type="range" min="0" max="1.5" step="0.05"></label><label><span>Dark glass tint <output id="dark-tint-value"></output></span><input id="dark-tint" type="range" min="0" max="1.5" step="0.05"></label><p class="hint">A little clearer, or a little softer. Each appearance remembers its own tint. Your device’s Reduce Transparency setting uses solid surfaces.</p><button class="secondary" type="button" id="appearance-reset">Reset appearance</button></form>`);
    const mode = document.getElementById('appearance-mode'); mode.value = this.preferences.mode;
    mode.onchange = () => { this.preferences.mode = mode.value; this.save(); this.applyTheme(); };
    for (const theme of ['light', 'dark']) {
      const input = document.getElementById(`${theme}-tint`), output = document.getElementById(`${theme}-tint-value`);
      input.value = this.preferences[theme]; output.value = `${Math.round(input.value * 100)}%`;
      input.oninput = () => { this.preferences[theme] = Number(input.value); output.value = `${Math.round(input.value * 100)}%`; this.save(); this.schedule(); };
    }
    document.getElementById('appearance-reset').onclick = () => { this.preferences = { mode: 'system', light: 1, dark: 1 }; this.save(); this.applyTheme(); this.settings(dialog); };
  }
}
