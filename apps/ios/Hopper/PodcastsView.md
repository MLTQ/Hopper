# PodcastsView.swift

Native podcast destination, RSS subscription and show/episode browsing. Select a desktop
as the show owner, so subscriptions and processing survive phone suspension. Episodes distinguish
original/downloaded/cleaned states and show real durable-job progress. Play/queue prepares missing
original media on the owner then uses the existing native player and Iroh streaming path.
Show menus open normal settings and a dedicated Cut ads sheet. No model inference runs in a view.

Subscription intake is split into SubscribePodcastView. DeliverySection displays pending and completed additions.
