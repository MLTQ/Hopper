# Apple Podcasts discovery

Blank queries use the US top podcasts chart, optionally scoped to an Apple Podcasts genre. Search uses the US iTunes Search API with the podcast entity. A selected genre filters the returned search results by genre ID. Bounded documents use the existing feed HTTP client.

Search listings require a public RSS URL. Charts supply an Apple show ID and cover thumbnails. Hopper constructs the Apple show link and the owner resolves it during subscription intake. If Apple provides no public feed, the delivery explains the failure.
