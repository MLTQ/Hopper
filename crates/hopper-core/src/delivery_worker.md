# delivery_worker.rs

One durable outbox worker per node, independent of ad analysis. Restarts requeue interrupted sends;
receiver receipts prevent duplicate effects. Transport failures retry with bounded backoff while
Hopper runs. Explicit receiver errors remain visible until manual retry. Never mark an item delivered
before Home acknowledges persistence. Shutdown drops pending work, leaving a recoverable record.
iOS execution is subject to app suspension; no promise of transfers while the app is terminated.
