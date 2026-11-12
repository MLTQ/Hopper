import SwiftUI
import UniformTypeIdentifiers

struct MediaArt: View {
    let item: MediaItem
    var size: CGFloat = 52
    var body: some View {
        let colors: [Color] = [.init(red: 0.77, green: 0.83, blue: 0.71), .init(red: 0.70, green: 0.80, blue: 0.78), .init(red: 0.84, green: 0.72, blue: 0.59), .init(red: 0.77, green: 0.76, blue: 0.85)]
        let index = Int(item.id.prefix(2), radix: 16) ?? 0
        ZStack {
            RoundedRectangle(cornerRadius: size / 10).fill(LinearGradient(colors: [colors[index % colors.count], colors[index % colors.count].opacity(0.6)], startPoint: .topLeading, endPoint: .bottomTrailing))
            Circle().stroke(.white.opacity(0.25), lineWidth: 1).frame(width: size * 0.9).offset(x: size * 0.25, y: -size * 0.2).clipped()
            Text(item.initials).font(.system(size: size * 0.36, weight: .regular, design: .serif)).foregroundColor(Color(red: 0.19, green: 0.34, blue: 0.26))
        }.frame(width: size, height: size).clipShape(RoundedRectangle(cornerRadius: size / 10)).accessibilityHidden(true)
    }
}

struct MediaRow: View {
    let item: MediaItem
    var body: some View {
        HStack(spacing: 13) {
            MediaArt(item: item)
            VStack(alignment: .leading, spacing: 5) {
                Text(item.title).font(.subheadline.weight(.semibold)).lineLimit(2)
                Text(item.subtitle).font(.caption).foregroundColor(HopperTheme.muted).lineLimit(1)
                if let progress = item.progress, progress.position > 0, let duration = item.duration, duration > 0 {
                    ProgressView(value: min(progress.position / duration, 1)).tint(HopperTheme.forest).frame(maxWidth: 130)
                }
            }
            Spacer(minLength: 0)
            if item.isLocal { Image(systemName: "arrow.down.circle.fill").foregroundColor(HopperTheme.muted).font(.caption) }
        }.padding(.vertical, 6)
    }
}

struct LibraryView: View {
    @EnvironmentObject var client: HopperClient
    @EnvironmentObject var player: HopperPlayer
    @State private var search = ""
    @State private var filter = "all"
    @State private var folder = ""
    @State private var importing = false
    @State private var detail: MediaItem?
    private var folders: [String] {
        var paths = Set<String>()
        for item in client.library.items {
            var path = ""
            for part in item.folder.split(separator: "/") {
                path = path.isEmpty ? String(part) : path + "/" + part
                paths.insert(path)
            }
        }
        return paths.sorted { $0.localizedStandardCompare($1) == .orderedAscending }
    }

    private var items: [MediaItem] {
        client.library.items.filter { item in
            (filter == "all" || item.kind == filter || filter == "continue" && (item.progress?.position ?? 0) > 0 && item.progress?.completed == false)
            && (folder.isEmpty || item.folder == folder || item.folder.hasPrefix(folder + "/"))
            && (search.isEmpty || "\(item.title) \(item.creator) \(item.folder)".localizedCaseInsensitiveContains(search))
        }.sorted { $0.title.localizedStandardCompare($1.title) == .orderedAscending }
    }
    var body: some View {
        NavigationStack {
            List {
                Section {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("GOOD THINGS, LINED UP.").font(.system(size: 9, weight: .semibold)).tracking(2).foregroundColor(HopperTheme.muted)
                        Text("In the hopper.").font(.system(size: 39, weight: .regular, design: .serif)).foregroundColor(HopperTheme.forest)
                        Text("Your collection. Everywhere you are.").font(.subheadline).foregroundColor(HopperTheme.muted)
                    }.padding(.vertical, 18).listRowBackground(HopperTheme.background).listRowSeparator(.hidden)
                }
                if !client.ready {
                    HStack { ProgressView(); Text("Opening your hopper…").font(.subheadline) }.listRowBackground(HopperTheme.background)
                } else if client.library.items.isEmpty {
                    VStack(spacing: 18) {
                        Image(systemName: "tray.and.arrow.down").font(.system(size: 47, weight: .ultraLight)).foregroundColor(HopperTheme.muted)
                        Text("Good things go in here.").font(.system(size: 23, design: .serif))
                        Text("Scan the invitation on your desktop to play its shared media.").font(.subheadline).foregroundColor(HopperTheme.muted).multilineTextAlignment(.center)
                        ScanPairButton().buttonStyle(.borderedProminent)
                        Button("Add a file") { importing = true }
                    }.frame(maxWidth: .infinity).padding(.vertical, 40).listRowBackground(HopperTheme.background).listRowSeparator(.hidden)
                } else {
                    Section {
                        ForEach(items) { item in
                            Button { Task { await player.play(item) } } label: { MediaRow(item: item) }.buttonStyle(.plain)
                                .listRowBackground(HopperTheme.background)
                                .swipeActions(edge: .trailing, allowsFullSwipe: false) {
                                    Button { player.enqueue(item) } label: { Label("Queue", systemImage: "text.badge.plus") }.tint(HopperTheme.forest)
                                    if !item.isLocal { Button { Task { await client.keep(item) } } label: { Label("Keep", systemImage: "arrow.down") }.tint(.orange) }
                                }
                                .contextMenu {
                                    Button("Add to queue", systemImage: "text.badge.plus") { player.enqueue(item) }
                                    Button("Details & chapters", systemImage: "list.bullet") { detail = item }
                                    if !item.isLocal { Button("Keep on this device", systemImage: "arrow.down.circle") { Task { await client.keep(item) } } }
                                }
                        }
                    } header: {
                        HStack {
                            Menu {
                                Button("All folders") { folder = "" }
                                ForEach(folders, id: \.self) { path in
                                    Button(path) { folder = path }
                                }
                            } label: { Label(folder.isEmpty ? "All folders" : folder, systemImage: "folder") }
                            Spacer()
                            Button("Play all") { player.playAll(items) }.disabled(items.isEmpty)
                        }
                    }
                }
            }
            .listStyle(.plain).scrollContentBackground(.hidden).hopperScene()
            .navigationTitle("hopper.").navigationBarTitleDisplayMode(.inline)
            .searchable(text: $search, prompt: "Find something good")
            .refreshable { await client.refresh(sync: true) }
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Menu {
                        Picker("Collection", selection: $filter) { Text("Everything").tag("all"); Text("Continue listening").tag("continue"); Text("Audiobooks").tag("audiobook"); Text("Audio").tag("audio"); Text("Video").tag("video") }
                    } label: { Image(systemName: "line.3.horizontal.decrease") }.accessibilityLabel("Filter library")
                }
                ToolbarItem(placement: .navigationBarTrailing) { Button { importing = true } label: { Image(systemName: "plus") }.disabled(!client.ready).accessibilityLabel("Add media") }
            }
            .fileImporter(isPresented: $importing, allowedContentTypes: [.audio, .movie, UTType(filenameExtension: "m4b") ?? .audio], allowsMultipleSelection: true) { result in
                switch result { case .success(let urls): Task { await client.importFiles(urls) }; case .failure(let error): client.error = error.localizedDescription }
            }
            .sheet(item: $detail) { item in MediaDetailView(item: item) }
        }
    }
}

struct MediaDetailView: View {
    let item: MediaItem
    @EnvironmentObject var client: HopperClient
    @EnvironmentObject var player: HopperPlayer
    @Environment(\.dismiss) var dismiss
    var body: some View {
        NavigationStack {
            List {
                HStack(spacing: 18) { MediaArt(item: item, size: 90); VStack(alignment: .leading, spacing: 8) { Text(item.title).font(.headline); Text(item.subtitle).font(.caption).foregroundColor(HopperTheme.muted) } }
                HStack { Button("Play") { Task { await player.play(item) } }.buttonStyle(.borderedProminent); Button("Queue") { player.enqueue(item) }.buttonStyle(.bordered) }
                Section("Available from") { ForEach(item.sources, id: \.deviceId) { source in Label(source.name + (source.online ? "" : " · offline"), systemImage: source.local ? "iphone" : "desktopcomputer") } }
                if !item.isLocal { Button("Keep on this device") { Task { await client.keep(item) } }.disabled(client.working) }
                if !item.chapters.isEmpty {
                    Section("Chapters") { ForEach(item.chapters) { chapter in Button { Task { if player.currentId != item.id { await player.play(item) }; player.seek(chapter.start) } } label: { HStack { Text(chapter.title); Spacer(); Text(mediaTime(chapter.start)).foregroundColor(HopperTheme.muted) } } } }
                }
            }.navigationTitle("In your collection").navigationBarTitleDisplayMode(.inline).toolbar { Button("Done") { dismiss() } }
        }
    }
}
