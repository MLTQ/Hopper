# Episode downloads

The desktop keeps one durable download request per episode. Subscription queues the latest known published episode. An explicit QueueDownload action uses the same path and returns after persistence. Existing synchronous Download callers can still request a playable ID.

A dedicated worker performs downloads independently of the audio agent. Duplicate requests reuse pending work. Downloading already available media uses the existing managed copy. Interrupted operations resume on startup. Failures get up to three attempts and remain visible for manual retry. Both processing and downloads serialize through download_episode, which verifies publication of the original and metadata. Queued work for an unsubscribed show is removed. iOS observes owner status and never runs this worker.
