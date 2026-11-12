# PodcastSettingsView.swift

Normal episode-order/unsubscribe settings and separate Cut ads preferences. Defaults preserve
banter/promos and uncertain content; fades do not overlap retained audio. Explicit backlog count
is bounded; automatic processing applies only to future releases. Missing agent configuration is
visible and queues durable waiting jobs instead of pretending to analyze audio. Original downloads
remain available after unsubscribe. Processing commands target the show's owning desktop.

A sheet opened from an episode sends cut_episode with per-job settings. The show-level form starts with one unplayed episode.
