# Podcast Index discovery

The desktop signs requests to the fixed HTTPS API with its key, secret and current Unix time. Redirects are disabled. Responses have a size cap and timeout. The saved key pair lives in a private local file, with optional PODCAST_INDEX_API_KEY and PODCAST_INDEX_API_SECRET environment fallbacks. The settings endpoint accepts replacements and never returns either value.

The desktop caches successful category lists in memory. Trending queries send the category ID to the provider. Text queries filter the bounded by-term response by category ID. Results preserve upstream order and prefer the feed artwork.
