# PodcastModels.swift

Typed owner-directed subscription, episode, host, preference and job contracts. Library podcast
catalogues are optional for backwards decoding. Source episode IDs and playable byte hashes are
separate; only ready jobs associate a cleaned media ID. API errors propagate to native alerts.
The client waits for owner commands then reloads the merged library; background playback is untouched.

The catalogue decodes optional show and episode cover URLs and original chapter document references.
