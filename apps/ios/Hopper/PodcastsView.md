# PodcastsView.swift

Native podcast destination, RSS subscription and show/episode browsing. Select a desktop
as the show owner, so subscriptions and processing survive phone suspension. Episodes distinguish
original/downloaded/cleaned states and show real durable-job progress. Play/queue prepares missing
original media on the owner then uses the existing native player and Iroh streaming path.
Show menus open normal settings and a dedicated Cut ads sheet. No model inference runs in a view.

Subscription intake is split into SubscribePodcastView.

Show/library backgrounds use hopperScene while episode rows remain solid for legibility.
Semantic warnings and processing errors retain their original colors.

Podcasts opens on subscriptions. Import activity lives in Devices. Show and episode covers use publisher artwork when available.

The magnifying glass opens directory browsing. An episode’s Cut ads action opens its own settings sheet.

Per-episode Download queues an owner transfer without starting playback or requiring the phone to stay on the screen. Ready status names the owner. The Cutting room link opens the merged processing queue. Download and cut actions occupy their own row for narrow phone screens.
