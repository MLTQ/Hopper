import { browsePodcasts } from './directory.js';
import { importPodcasts } from './home.js';
import { agentSettings } from './agents.js';
import { api, escape as esc, time, duration } from './api.js';

// Only publishers' HTTP images are eligible for display. Feed text stays escaped.
function cover(address) {
  if (!address || !/^https?:\/\//i.test(address)) return '';
  return `<img src="${esc(address)}" alt="" loading="lazy" referrerpolicy="no-referrer">`;
}
export class Podcasts {
  constructor({ dialog, toast, load, player }) { Object.assign(this, { dialog, toast, load, player }); this.selected = null; this.library = null; }
  get catalogue() { return this.library?.podcasts || { shows: [], episodes: [], jobs: [], hosts: [] }; }
  show(id) { return this.catalogue.shows.find(s => s.id === id); }
  episodes(show) { return this.catalogue.episodes.filter(e => e.show_id === show.id).sort((a,b) => show.newest_first ? b.published-a.published : a.published-b.published); }
  latestJob(id) { return this.catalogue.jobs.filter(j => j.episode_id === id).sort((a,b) => b.created_at-a.created_at)[0]; }
  async command(owner, action) { const result = await api('podcasts/command', { owner_id: owner, ...action }); await this.load(); return result; }
  render(library, query = '') {
    this.library = library;
    const show = this.show(this.selected);
    document.getElementById('section-label').textContent = show ? show.title : 'Your podcasts';
    document.getElementById('play-all').hidden = true;
    document.getElementById('folder-list').hidden = true;
    const target = document.getElementById('collection');
    if (!show) {
      target.innerHTML = `<div class="podcast-toolbar"><button class="secondary" data-agent-settings>Audio agent</button><button class="secondary" data-podcast-subscribe>Import</button><button class="primary" data-podcast-browse>Browse podcasts</button></div>` + (this.catalogue.shows.length ? `<div class="podcast-grid">${this.catalogue.shows.filter(s=>s.title.toLowerCase().includes(query)).map(s=>`<article class="podcast-card"><div class="podcast-cover">◉${cover(s.artwork_url)}</div><button class="podcast-title" data-podcast-show="${s.id}"><strong>${esc(s.title)}</strong><small>${esc(s.author || s.owner.name)}</small></button><p>${this.episodes(s).length} episodes · ${esc(s.owner.name)}</p><button class="icon-button" data-podcast-settings="${s.id}" aria-label="Settings for ${esc(s.title)}">⚙</button></article>`).join('')}</div>` : `<div class="empty-filter"><span>◉</span><h2>Good company, on repeat.</h2><p>Import Apple Podcasts links or a Castamatic subscription list.<br>Its episodes will appear across your paired devices.</p></div>`);
      for (const image of target.querySelectorAll('.podcast-cover img')) image.onerror = () => image.remove();
      return;
    }
    const episodes = this.episodes(show).filter(e=>e.title.toLowerCase().includes(query));
    target.innerHTML = `<div class="podcast-toolbar"><button class="text-button" data-podcast-back>‹ All podcasts</button><button class="secondary" data-podcast-settings="${show.id}">⚙ Show settings</button></div><div class="podcast-description"><strong>${esc(show.author)}</strong><p>${esc(show.description)}</p><small>Library on ${esc(show.owner.name)}${show.error ? ` · ${esc(show.error)}` : ''}</small></div><div class="episode-list">${episodes.map(e=>{
      const job = this.latestJob(e.id), id = e.cleaned_id || e.original_id;
      const status = job?.active ? job.message : job?.message || (e.cleaned_id ? 'Cleaned version ready' : e.original_id ? 'Original ready' : 'Not downloaded');
      const active = job && ['queued','downloading','analysing','rendering','waiting_for_agent'].includes(job.state);
      return `<article class="episode-row"><div><h3>${esc(e.title)}</h3><small>${e.published > 0 ? new Date(e.published*1000).toLocaleDateString()+' · ' : ''}${duration(e.duration)}</small><p class="job-status ${job?.state === 'failed' ? 'job-error' : ''}">${esc(status)}${job?.removed_seconds > 0 ? ` · ${time(job.removed_seconds)} removed` : ''}</p>${e.metadata_warnings?.length ? `<small>${esc(e.metadata_warnings.join(' · '))}</small>` : ''}</div><div class="episode-actions">${active ? '' : `<button class="text-button" data-cut-episode="${e.id}">Cut ads</button>`}<button class="primary" data-episode-play="${e.id}">${id ? '▶ Play' : '↓ Get & play'}</button><button class="secondary" data-episode-queue="${e.id}" aria-label="Queue ${esc(e.title)}">＋</button>${e.cleaned_id && e.original_id !== e.cleaned_id ? `<button class="text-button" data-episode-original="${e.id}">Original</button>` : ''}${active ? `<button class="text-button" data-job-cancel="${job.id}" data-owner="${show.owner.id}">Cancel</button>` : job?.state === 'failed' || job?.state === 'cancelled' ? `<button class="text-button" data-job-retry="${job.id}" data-owner="${show.owner.id}">Retry</button>` : ''}</div></article>`;
    }).join('')}</div>`;
  }
  subscribe() { importPodcasts({ library: this.library, dialog: this.dialog, toast: this.toast, load: this.load }); }
  settings(id) {
    const show=this.show(id); if (!show) return;
    this.dialog(show.title, `<div class="podcast-form"><label>Episode order<select id="podcast-order"><option value="oldest" ${!show.newest_first?'selected':''}>Oldest first — listen in sequence</option><option value="newest" ${show.newest_first?'selected':''}>Newest first</option></select></label><button class="import-choice" data-podcast-cut="${id}"><span>✂</span><div><strong>Cut ads</strong><small>Keep the banter. Make the edits yours.</small></div>↗</button><button class="secondary" data-podcast-refresh="${id}">↻ Check for new episodes</button><button class="text-button" data-podcast-unsubscribe="${id}">Unsubscribe · keep downloaded files</button></div>`);
    document.getElementById('podcast-order').onchange = async event => {try {await this.command(show.owner.id,{action:'settings',show_id:id,newest_first:event.target.value==='newest',cut_options:show.cut_options});this.toast('Episode order saved');}catch(e){this.toast(e.message,true)}};
  }
  cut(id, episodeId) {
    const show=this.show(id); if (!show)return;
    const host=this.catalogue.hosts.find(h=>h.device.id===show.owner.id), o=show.cut_options;
    const episode = this.catalogue.episodes.find(e => e.show_id === id && e.id === episodeId);
    const available=episode ? [episode] : this.episodes(show).filter(e=>{const item=this.library.items.find(i=>i.id===(e.cleaned_id||e.original_id));return !item?.progress?.completed;});
    this.dialog('Cut ads', `<p class="dialog-lead">${esc(episode?.title || show.title)}</p><form id="cut-form" class="podcast-form"><label class="setting-toggle"><span><strong>Keep banter</strong><small>Leave the hosts’ conversations and tangents in.</small></span><input type="checkbox" name="banter" ${o.keep_banter?'checked':''}></label><label class="setting-toggle"><span><strong>Keep show promos</strong><small>Retain trailers and announcements for other shows.</small></span><input type="checkbox" name="promos" ${o.keep_promos?'checked':''}></label><label>Fade at cuts<select name="fade"><option value="0" ${o.fade_ms===0?'selected':''}>Off — direct stitch</option><option value="20" ${o.fade_ms===20?'selected':''}>Short · 20 ms</option><option value="100" ${o.fade_ms===100?'selected':''}>Soft · 100 ms</option><option value="250" ${o.fade_ms===250?'selected':''}>Long · 250 ms</option></select></label><label>When the agent is uncertain<select name="confidence"><option value="0.9" ${o.minimum_confidence===0.9?'selected':''}>Keep it · conservative</option><option value="0.98" ${o.minimum_confidence===0.98?'selected':''}>Keep more · very conservative</option><option value="0.75" ${o.minimum_confidence===0.75?'selected':''}>Allow more cuts</option></select></label>${episode ? '' : `<label>Episodes to prepare<select name="count"><option value="1" selected>Next unplayed episode</option><option value="5">Next 5 unplayed episodes</option><option value="10">Next 10 unplayed episodes</option><option value="500">All unplayed · up to 500</option></select></label><label class="setting-toggle"><span><strong>Also clean future episodes</strong><small>Prepare new releases with these settings.</small></span><input type="checkbox" name="auto" ${o.auto_process?'checked':''}></label>` }<p class="hint">Runs on ${esc(show.owner.name)}. Original files are kept. Chapters, supported transcripts, and listening position follow the edited timeline.</p>${!host?.agent_available?`<p class="job-notice">Set up the audio agent on ${esc(show.owner.name)}. Jobs will wait until it is available.</p>${show.owner.id === this.library.device.id ? '<button type="button" class="secondary" data-agent-settings>Set up audio agent</button>' : ''}`:''}<button class="primary" ${!available.length?'disabled':''}>${available.length?'Start cutting ads':'No unplayed episodes'}</button></form>`);
    document.getElementById('cut-form').onsubmit=async event=>{event.preventDefault();const form=event.target,data=new FormData(form),button=form.querySelector('button.primary');button.disabled=true;try {const ids=available.slice(0,episode ? 1 : Number(data.get('count'))).map(e=>e.id);const options={keep_banter:data.has('banter'),keep_promos:data.has('promos'),fade_ms:Number(data.get('fade')),minimum_confidence:Number(data.get('confidence')),auto_process:data.has('auto')};await this.command(show.owner.id,episode ? {action:'cut_episode',episode_id:episode.id,options} : {action:'cut',show_id:id,episode_ids:ids,options});document.getElementById('dialog').close();this.selected=id;await this.load();this.toast(`${ids.length} episode${ids.length===1?'':'s'} queued for your agent`);}catch(e){this.toast(e.message,true)}finally{button.disabled=false}};
  }
  async play(id, enqueue=false, original=false) {
    const episode=this.catalogue.episodes.find(e=>e.id===id),show=this.show(episode?.show_id);if(!episode||!show)return;
    let media=original ? episode.original_id : episode.cleaned_id || episode.original_id;
    if(!media){this.toast('Downloading the episode to its desktop…');const r=await this.command(show.owner.id,{action:'download',episode_id:id});media=r.media_id;}
    await this.load(); if(enqueue){this.player.enqueue(media);this.toast('Added to your queue');}else await this.player.play(media);
  }
  async click(target) {
    const d=target.dataset;
    if('agentSettings' in d)await agentSettings({dialog:this.dialog,toast:this.toast,load:this.load});
    if('podcastBrowse' in d)browsePodcasts(this);
    if('podcastSubscribe' in d)this.subscribe();
    if(d.podcastShow){this.selected=d.podcastShow;await this.load();}
    if('podcastBack' in d){this.selected=null;await this.load();}
    if(d.podcastSettings)this.settings(d.podcastSettings);
    if(d.podcastCut)this.cut(d.podcastCut);
    if(d.cutEpisode)this.cut(this.selected,d.cutEpisode);
    if(d.podcastRefresh){await this.command(this.show(d.podcastRefresh).owner.id,{action:'refresh',show_id:d.podcastRefresh});this.toast('Episodes refreshed');}
    if(d.podcastUnsubscribe){await this.command(this.show(d.podcastUnsubscribe).owner.id,{action:'unsubscribe',show_id:d.podcastUnsubscribe});this.selected=null;document.getElementById('dialog').close();await this.load();}
    if(d.episodePlay||d.episodeQueue||d.episodeOriginal){target.disabled=true;try{await this.play(d.episodePlay||d.episodeQueue||d.episodeOriginal,Boolean(d.episodeQueue),Boolean(d.episodeOriginal));}finally{target.disabled=false}}
    if(d.jobCancel||d.jobRetry)await this.command(d.owner,{action:d.jobCancel?'cancel':'retry',job_id:d.jobCancel||d.jobRetry});
  }
}
