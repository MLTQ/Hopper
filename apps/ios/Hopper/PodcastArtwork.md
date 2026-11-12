# PodcastArtwork.swift

Show, episode, and player artwork share coalesced bounded requests through Hopper’s local node.
ImageIO samples the original into a thumbnail. Decoded images and failed requests have bounded
memory caches. The app keeps a bounded private disk cache for previously displayed covers.
