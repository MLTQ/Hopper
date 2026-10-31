# home_delivery.rs

Three independent local Iroh nodes exercise laptop → Home → player. OPML nested duplicate feeds
produce one durable delivery; the receiver owns the subscription and repeated delivery returns
its receipt. Authenticated file import copies before sending, leaves originals unchanged, survives
source moves and serves another player after the laptop shuts down. A second test persists Home
selection and undelivered work across sender restart while Home is offline. No live Apple service,
iOS lifecycle, WAN discovery or actual Castamatic export UI is inferred from these fixtures.
