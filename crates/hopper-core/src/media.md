# media.rs

## Purpose
Range-aware readers, scoped playback grants, and content-verified offline copies.

## Contracts
- Byte ranges are inclusive externally and offset/length internally; suffix ranges are supported.
- Playback grants authorize one media ID for seven days and never authorize control operations.
- Local copies win; remote sources are tried in turn if missing/unreachable.
- Downloads stream to a private temporary file, verify BLAKE3 and size, then publish atomically.
- Failed downloads leave no catalogue entry. Automatic resumable downloads are future work.
- No method here deletes an original or propagates file deletion across devices.

Desktop playback and peer reads resolve registered folder sources by the same content ID used by managed copies.

A verified offline copy reuses identical managed bytes already kept by a folder import. Publishing the independent copy preserves the other source’s modification stamp.
