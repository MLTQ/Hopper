import { api, time, tone, initials } from './api.js';

export class Player {
  constructor(onChange, onError) {
    this.media = document.getElementById('media');
    this.items = new Map(); this.queue = []; this.current = null; this.loading = 0;
    this.onChange = onChange; this.onError = onError; this.saves = Promise.resolve(); this.queueSaves = Promise.resolve(); this.lastSave = 0; this.loadedId = null;
    this.media.addEventListener('timeupdate', () => { this.paint(); if (Date.now() - this.lastSave > 10000) this.save(); });
    this.media.addEventListener('play', () => this.paint());
    this.media.addEventListener('pause', () => { this.save(); this.paint(); });
    this.media.addEventListener('seeked', () => this.save());
    this.media.addEventListener('ended', () => { this.save(true); this.next(); });
    this.media.addEventListener('error', () => { if (this.media.src) this.onError('This file could not play. Check its source device and whether this format is supported by your system.'); this.paint(); });
    this.media.addEventListener('waiting', () => { document.getElementById('playback-status').textContent = 'BUFFERING…'; });
    document.getElementById('toggle-play').onclick = () => this.toggle();
    document.getElementById('previous').onclick = () => this.next(-1);
    document.getElementById('next').onclick = () => this.next();
    document.getElementById('rewind').onclick = () => this.seek(this.media.currentTime - 15);
    document.getElementById('forward').onclick = () => this.seek(this.media.currentTime + 30);
    document.getElementById('seek').onchange = e => this.seek(Number(e.target.value));
    document.getElementById('rate').onchange = e => { this.media.playbackRate = Number(e.target.value); };
    this.media.addEventListener('loadedmetadata', () => this.paint());
    this.media.addEventListener('playing', () => this.paint());
    if ('mediaSession' in navigator) {
      for (const [name, handler] of Object.entries({ play: () => this.toggle(true), pause: () => this.media.pause(),
        previoustrack: () => this.next(-1), nexttrack: () => this.next(), seekbackward: () => this.seek(this.media.currentTime - 15),
        seekforward: () => this.seek(this.media.currentTime + 30), seekto: d => this.seek(d.seekTime) })) {
        try { navigator.mediaSession.setActionHandler(name, handler); } catch {}
      }
    }
  }
  update(library) {
    this.items = new Map(library.items.map(item => [item.id, item]));
    const covers = new Map((library.podcasts?.shows || []).map(show => [show.id, show.artwork_url]));
    this.episodeArt = new Map();
    for (const episode of library.podcasts?.episodes || []) {
      for (const id of [episode.original_id, episode.cleaned_id]) {
        if (id) this.episodeArt.set(id, { cover: episode.artwork_url || covers.get(episode.show_id), episode });
      }
    }
    if (!this.initialized) {
      this.queue = library.queue.items.filter(id => this.items.has(id));
      this.current = library.queue.current && this.items.has(library.queue.current) ? library.queue.current : null;
      this.initialized = true;
    }
    this.paint();
  }
  async persist() {
    const queue = { items: [...this.queue], current: this.current && this.queue.includes(this.current) ? this.current : null };
    try { this.queueSaves = this.queueSaves.catch(() => {}).then(() => api('queue', queue)); await this.queueSaves; }
    catch (e) { this.onError(e.message); }
    this.onChange();
  }
  async play(id) {
    const item = this.items.get(id); if (!item) return;
    const attempt = ++this.loading;
    this.save(); this.media.pause(); this.loadedId = null; this.media.removeAttribute('src'); this.media.load();
    if (!this.queue.includes(id)) this.queue.push(id);
    this.current = id; this.extraChapters = undefined;
    this.onChange(); this.paint();
    document.getElementById('playback-status').textContent = 'CONNECTING…';
    try {
      const result = await api(`play/${id}`, {});
      if (attempt !== this.loading) return;
      const start = item.progress && !item.progress.completed ? item.progress.position : 0;
      this.media.src = new URL(result.path, location.href).href;
      this.extraChapters = undefined;
      const episode = this.episodeArt?.get(id)?.episode;
      if (episode?.original_id === id && episode.chapters_url && !item.chapters.some(c => c.artwork_url)) {
        api(`podcasts/chapters/${episode.id}`).then(chapters => {
          if (attempt === this.loading) { this.extraChapters = chapters; this.paint(); }
        }).catch(() => {});
      }
      this.media.onloadedmetadata = () => { if (attempt !== this.loading) return; this.loadedId = id; this.media.currentTime = Math.min(start, Number.isFinite(this.media.duration) ? Math.max(0, this.media.duration - 1) : start); };
      this.media.playbackRate = Number(document.getElementById('rate').value);
      document.getElementById('show-video').hidden = item.kind !== 'video';
      if (item.kind === 'video') document.getElementById('video-container').hidden = false;
      await this.persist(); if (attempt !== this.loading) return; await this.media.play();
    } catch (e) { this.onError(e.message); this.paint(); }
  }
  async toggle(forcePlay = false) {
    if (!this.media.src) { const id = this.current || this.queue[0]; if (id) await this.play(id); return; }
    if (this.media.paused || forcePlay) { try { await this.media.play(); } catch (e) { this.onError(e.message); } }
    else this.media.pause();
  }
  enqueue(id) { if (!this.queue.includes(id)) this.queue.push(id); this.persist(); }
  async playAll(items, start = items[0]?.id) { this.queue = items.map(item => item.id); if (start) await this.play(start); }
  move(id, by) { const i = this.queue.indexOf(id), j = i + by; if (j < 0 || j >= this.queue.length) return; [this.queue[i], this.queue[j]] = [this.queue[j], this.queue[i]]; this.persist(); }
  stop() { ++this.loading; this.save(); this.media.pause(); this.loadedId = null; this.media.onloadedmetadata = null; this.media.removeAttribute('src'); this.media.load(); this.current = null; }
  remove(id) { if (id === this.current) this.stop(); this.queue = this.queue.filter(x => x !== id); this.persist(); this.paint(); }
  clear() { this.stop(); this.queue = []; this.persist(); this.paint(); }
  next(by = 1) { const i = this.queue.indexOf(this.current); const id = this.queue[i + by]; if (id) this.play(id); else this.paint(); }
  seek(value) { if (!this.media.src) return; this.media.currentTime = Math.max(0, Math.min(value, Number.isFinite(this.media.duration) ? this.media.duration : value)); }
  save(completed = this.media.ended) {
    if (!this.loadedId || !this.media.src || this.media.readyState === 0) return;
    const item_id = this.loadedId, position = this.media.currentTime, duration = Number.isFinite(this.media.duration) ? this.media.duration : null;
    this.lastSave = Date.now();
    const item = this.items.get(item_id); if (item) item.progress = { position, duration, completed };
    this.saves = this.saves.then(() => api('progress', { item_id, position, duration, completed })).catch(e => this.onError(e.message));
  }
  paint() {
    const item = this.items.get(this.current), media = this.media;
    document.getElementById('player-title').textContent = item?.title || 'Your next good listen';
    document.getElementById('player-subtitle').textContent = item ? item.creator || item.folder || 'From your collection' : 'Pick something from your hopper';
    const chapters = this.extraChapters || item?.chapters || [];
    const now = media.currentTime || 0;
    let chapter;
    for (const cue of chapters) { if (cue.start > now) break; if (cue.end == null || now < cue.end) chapter = cue; }
    const cover = this.episodeArt?.get(item?.id)?.cover;
    const urls = [chapter?.artwork_url, cover].filter(u => typeof u === 'string' && /^https?:\/\//i.test(u));
    const art = document.getElementById('player-art');
    const key = JSON.stringify([item?.id, ...urls]);
    if (art.dataset.cover !== key) {
      art.dataset.cover = key; art.className = `art ${item ? tone(item.id) : 'tone-0'}`;
      art.textContent = item ? initials(item) : '♪';
      if (urls.length) {
        const image = new Image(); image.alt = ''; image.referrerPolicy = 'no-referrer';
        let next = 1;
        image.onerror = () => { if (next < urls.length) image.src = urls[next++]; else image.remove(); };
        image.src = urls[0]; art.append(image);
      }
      if (item && 'mediaSession' in navigator && typeof MediaMetadata !== 'undefined') {
        navigator.mediaSession.metadata = new MediaMetadata({ title: item.title, artist: item.creator,
          album: item.album, artwork: urls.map(src => ({ src })) });
      }
    }
    document.getElementById('toggle-play').textContent = media.paused ? '▶' : 'Ⅱ';
    document.getElementById('toggle-play').setAttribute('aria-label', media.paused ? 'Play' : 'Pause');
    const total = Number.isFinite(media.duration) ? media.duration : item?.duration || 0;
    const position = media.src ? media.currentTime : item?.progress?.position || 0;
    document.getElementById('elapsed').textContent = time(position);
    document.getElementById('duration').textContent = time(total);
    const seek = document.getElementById('seek'); seek.max = total || 1; if (document.activeElement !== seek) seek.value = position;
    document.getElementById('playback-status').textContent = item ? media.paused ? 'PAUSED' : 'IN YOUR OWN WORLD' : 'READY WHEN YOU ARE';
    if ('mediaSession' in navigator) { navigator.mediaSession.playbackState = !item ? 'none' : media.paused ? 'paused' : 'playing'; }
  }
}
