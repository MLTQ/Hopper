# SubscribePodcastView.swift

Native Apple/RSS link and OPML file intake. Files use security-scoped access and a bounded read;
the core validates XML, deduplicates feed links and queues subscriptions on a selected desktop.
Offline hosts remain selectable and the preferred Home is preselected. Partial OPML warnings
remain visible after accepted entries enqueue. No history, private subscription entitlement,
or app-owned downloads are inferred from the export.
