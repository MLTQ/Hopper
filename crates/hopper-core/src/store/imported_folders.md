# Imported desktop folders

One-time folder imports retain a local source per (folder ID, media ID). A path import uses the selected directory's canonical path. A folder uploaded by a browser uses the first component of its relative filename, so uploads with the same top-level name share a group. Listing counts distinct media within each group. Individual files, explicit offline copies, shared-folder scans and peer catalogues have independent lifetimes.

On first open, a SQLite transaction groups older desktop file records with saved folder metadata. For referenced files it reconstructs the root when the stored path matches the logical folder. Older managed files carry no import history. Files identified by saved podcast episodes, incoming Home receipts or cached peer catalogues retain their independent local records. Empty one-time imports have no items to list.

Removal deletes a folder's index entries. Original files and managed blobs stay on disk. A managed source needed for an unfinished Home transfer stays available under an unlisted entry. A database trigger releases that entry when the final delivery of the item succeeds, including after restart. Folder identifiers and paths stay within the local control API.
