# LibraryView.swift

## Purpose
Native library browsing, folder/type/search filters, file imports, chapters and offline-copy actions.

## Contracts
UI reflects real catalogue/availability. MediaArt is a local placeholder derived from ID/title.
Playback and downloads delegate to native owners; views never drive queue advancement. System
file importer uses explicit user-selected URLs. Details show all advertised sources.

HopperTheme is defined in HopperTheme.swift. Library/detail backgrounds use its local scene;
library rows stay solid for reading. Artwork retains dark ink over its fixed pale palette.

The empty library offers a pairing scan. The folder menu beside Play all filters the combined catalogue, including parent folders whose episodes all live in subfolders. Selecting a parent includes its descendants.
