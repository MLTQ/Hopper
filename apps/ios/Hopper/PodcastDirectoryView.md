# Podcast discovery

Native directory, destination and category pickers share the Rust discovery API. Submitting the search field searches the directory. An empty term requests the chart. Changing the request cancels the previous view task and stale results are discarded.

Rows use the podcast thumbnail cache and the durable Home import flow. A destination may be offline when subscribing to an Apple listing. Podcast Index discovery requires its configured desktop to be reachable. Credentials are entered on the desktop.
