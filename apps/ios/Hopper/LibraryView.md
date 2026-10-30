# LibraryView.swift

## Purpose
Native library browsing, folder/type/search filters, file imports, chapters and offline-copy actions.

## Contracts
UI reflects real catalogue/availability. MediaArt is a local placeholder derived from ID/title.
Playback and downloads delegate to native owners; views never drive queue advancement. System
file importer uses explicit user-selected URLs. Details show all advertised sources.
