import { escape as esc, time } from './api.js';

export function cuttingRoom(catalogue) {
  const jobs = [...catalogue.jobs].sort((a,b) => Number(active(b)) - Number(active(a)) || b.created_at - a.created_at);
  const rows = jobs.map(job => {
    const show = catalogue.shows.find(s => s.id === job.show_id);
    if (!show) return '';
    const episode = catalogue.episodes.find(e => e.id === job.episode_id);
    return `<article class="episode-row"><div><button class="text-button" data-podcast-show="${esc(show.id)}"><strong>${esc(episode?.title || show.title)}</strong></button><small>${esc(show.title)} · ${esc(show.owner.name)}</small><p class="job-status ${job.state === 'failed' ? 'job-error' : ''}">${esc(job.message)}${job.removed_seconds > 0 ? ` · ${time(job.removed_seconds)} removed` : ''}</p></div><div class="episode-actions">${active(job) ? `<button class="secondary" data-job-cancel="${esc(job.id)}" data-owner="${esc(show.owner.id)}">Cancel</button>` : ['failed','cancelled'].includes(job.state) ? `<button class="secondary" data-job-retry="${esc(job.id)}" data-owner="${esc(show.owner.id)}">Retry</button>` : ''}</div></article>`;
  }).join('');
  return '<div class="podcast-toolbar"><button class="text-button" data-podcast-back>‹ All podcasts</button></div>' +
    (rows ? `<div class="episode-list">${rows}</div>` : '<p class="empty-filter">Choose an episode and select Cut ads to send it here.</p>');
}
function active(job) {
  return ['queued','downloading','analysing','rendering','waiting_for_agent'].includes(job.state);
}
