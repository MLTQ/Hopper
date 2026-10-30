# http.rs

## Purpose
Loopback-only shell API and byte-range media gateway used by web/native players.

## Contracts
- /api requires a per-run bearer token and validates Host/Origin. No CORS is enabled.
- /media requires a separate item-scoped grant and supports GET/HEAD and single HTTP ranges.
- Import paths refer to the local desktop. Multipart uploads copy into managed storage.
- UI assets contain no authorization secrets; shells pass the secret via URL fragment.
- APIs return JSON errors; unavailable media returns 503 and invalid ranges return 416.
- Long imports/downloads currently keep the request pending; background job UI is follow-up work.
- The HTTP listener shuts down when its owning node closes.
