import SwiftUI

/// All paths are library paths published by the desktop.
struct FolderContents: View {
    let path: String
    let searching: Bool
    let descendants: [MediaItem]
    let direct: [MediaItem]
    let children: [String]
    @EnvironmentObject var player: HopperPlayer
    @EnvironmentObject var client: HopperClient
    @State private var detail: MediaItem?

    init(path: String, items: [MediaItem], searching: Bool = false) {
        self.path = path
        self.searching = searching
        let prefix = path.isEmpty ? "" : path + "/"
        descendants = items.filter { path.isEmpty || $0.folder == path || $0.folder.hasPrefix(prefix) }
        direct = searching ? descendants : descendants.filter { $0.folder == path }
        children = Set(descendants.compactMap { item -> String? in
            guard item.folder != path else { return nil }
            return item.folder.dropFirst(prefix.count).split(separator: "/").first.map(String.init)
        }).sorted { $0.localizedStandardCompare($1) == .orderedAscending }
    }

    var body: some View {
        let prefix = path.isEmpty ? "" : path + "/"
        if !searching && !children.isEmpty {
            Section("Folders") {
                ForEach(children, id: \.self) { name in
                    NavigationLink(value: prefix + name) { Label(name, systemImage: "folder") }
                }
            }
        }
        if !descendants.isEmpty {
            Section {
                ForEach(direct) { item in
                    Button { player.playAll(descendants, startingAt: item) } label: { MediaRow(item: item) }
                        .buttonStyle(.plain)
                        .swipeActions(edge: .trailing, allowsFullSwipe: false) {
                            Button { player.enqueue(item) } label: { Label("Queue", systemImage: "text.badge.plus") }
                            if item.canKeep { Button { Task { await client.keep(item) } } label: { Label("Keep", systemImage: "arrow.down") }.tint(.orange) }
                        }
                        .contextMenu {
                            Button("Add to queue", systemImage: "text.badge.plus") { player.enqueue(item) }
                            Button("Details & chapters", systemImage: "list.bullet") { detail = item }
                            if item.canKeep { Button("Keep on this device", systemImage: "arrow.down.circle") { Task { await client.keep(item) } } }
                        }
                }
            } header: {
                HStack {
                    Text(searching ? "Matches" : "Media")
                    Spacer()
                    Button(path.isEmpty || searching ? "Play all" : "Play folder") { player.playAll(descendants) }
                }
            }
            .sheet(item: $detail) { MediaDetailView(item: $0) }
        } else {
            Text(searching ? "No matching media" : "No media in this folder").foregroundStyle(.secondary)
        }
    }
}
