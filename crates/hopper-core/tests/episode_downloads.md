# Episode download integration

A paired listener subscribes to a feed whose older episode appears first. Only the newest episode is queued. A gated HTTP response lets the owner shut down during download. On restart the persisted request completes, repeated subscription reuses the original, and the peer sees owner status. A manual older-episode download completes while the agent waits in an external inbox. Real audio inspection requires FFmpeg.
