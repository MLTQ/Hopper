# intake.rs

Bounded OPML subscriptions (Castamatic and other standard exports), direct RSS URLs, and Apple
Podcasts share-link resolution. Nested OPML folders are traversed; normalized feed URLs deduplicate.
Only HTTP(S) feeds without embedded credentials are accepted. Spotify is explicitly deferred.
Apple numeric show IDs resolve through its public lookup endpoint; no page scraping or guessed feeds.
An episode ID is looked up separately and matched by GUID/enclosure; missing episode mapping is
reported by the delivery receiver, never silently represented as a successful episode import.
No listening history or downloaded-media migration is implied by OPML import.

References: https://castamatic.com/ (OPML support); Apple's archived iTunes Search API Lookup Examples.

Episode resolution queries the parent show with entity=podcastEpisode and a 200-item bound; direct
episode-ID lookups can return zero results. Both collection and track ID must match. Older items
not returned by Apple remain an explicit unmatched-episode warning, never a fuzzy title guess.

Numeric show IDs are found independently of title slugs that happen to begin with “id”.
