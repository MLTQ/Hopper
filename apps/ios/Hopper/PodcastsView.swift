import SwiftUI

struct PodcastsView: View {
    @EnvironmentObject var client: HopperClient
    @State private var subscribing = false
    var body: some View {
        NavigationStack {
            List {
                if client.podcasts.shows.isEmpty {
                    VStack(spacing: 18) {
                        Image(systemName: "dot.radiowaves.left.and.right").font(.largeTitle)
                        Text("Good company, on repeat.").font(.system(size: 26, design: .serif))
                        Text("Import an Apple Podcasts link or a Castamatic subscription list. Its episodes will appear across your paired devices.").font(.subheadline).multilineTextAlignment(.center)
                        NavigationLink("Browse podcasts") { PodcastDirectoryView() }.buttonStyle(.borderedProminent)
                    }.foregroundColor(HopperTheme.forest).padding(.vertical, 55).frame(maxWidth: .infinity).listRowBackground(HopperTheme.background).listRowSeparator(.hidden)
                }
                ForEach(client.podcasts.shows) { show in
                    NavigationLink { PodcastShowView(showId: show.id) } label: {
                        HStack(spacing: 16) {
                            PodcastCover(urls: [show.artworkUrl].compactMap { $0 }, size: 58) { HopperTheme.surface.overlay(Image(systemName: "mic.fill")) }
                            VStack(alignment: .leading, spacing: 5) {
                                Text(show.title).font(.headline)
                                Text(show.author.isEmpty ? show.owner.name : show.author).font(.caption).foregroundColor(HopperTheme.muted)
                            }
                        }.padding(.vertical, 7)
                    }.listRowBackground(HopperTheme.background)
                }
            }.listStyle(.plain).scrollContentBackground(.hidden).hopperScene()
                .navigationTitle("Podcasts").toolbar {
                    NavigationLink { PodcastDirectoryView() } label: { Image(systemName: "magnifyingglass") }.accessibilityLabel("Browse podcasts")
                    Button { subscribing = true } label: { Image(systemName: "plus") }.accessibilityLabel("Import a podcast link or subscription list")
                }
                .refreshable { await client.refresh(sync: true) }
                .sheet(isPresented: $subscribing) { SubscribePodcastView() }
        }
    }
}

struct PodcastShowView: View {
    let showId: String
    @EnvironmentObject var client: HopperClient
    @EnvironmentObject var player: HopperPlayer
    @Environment(\.dismiss) var dismiss
    @State private var settings = false
    @State private var cutting = false
    @State private var cutEpisode: PodcastEpisode?
    @State private var preparing: String?
    private var show: PodcastShow? { client.podcasts.shows.first { $0.id == showId } }
    var body: some View {
        Group {
            if let show {
                List {
                    Section {
                        HStack(spacing: 16) {
                            PodcastCover(urls: [show.artworkUrl].compactMap { $0 }, size: 90) { HopperTheme.surface.overlay(Image(systemName: "mic.fill")) }
                            Text(show.author).font(.headline)
                        }
                        Text(show.description).font(.subheadline).foregroundColor(HopperTheme.muted).lineLimit(5)
                        Text("Library on \(show.owner.name)").font(.caption).foregroundColor(HopperTheme.muted)
                        if let error = show.error { Text(error).font(.caption).foregroundColor(.orange) }
                    }.listRowBackground(HopperTheme.background)
                    ForEach(client.podcasts.episodes(for: show)) { episode in
                        VStack(alignment: .leading, spacing: 10) {
                            HStack {
                                if episode.artworkUrl != nil { PodcastCover(urls: client.coverURLs(for: episode), size: 46) { HopperTheme.surface } }
                                Text(episode.title).font(.headline)
                            }
                            HStack { if episode.published > 0 { Text(Date(timeIntervalSince1970: Double(episode.published)), style: .date) }; if let duration = episode.duration { Text(mediaTime(duration)) } }.font(.caption).foregroundColor(HopperTheme.muted)
                            if let job = client.podcasts.job(for: episode) {
                                Text(job.message + (job.removedSeconds > 0 ? " · \(mediaTime(job.removedSeconds)) removed" : "")).font(.caption).foregroundColor(job.state == "failed" ? .orange : HopperTheme.forest)
                                if job.active { Button("Cancel processing") { action(PodcastCommand(ownerId: show.owner.id, action: "cancel", jobId: job.id)) }.font(.caption) }
                                else if job.state == "failed" || job.state == "cancelled" { Button("Retry") { action(PodcastCommand(ownerId: show.owner.id, action: "retry", jobId: job.id)) }.font(.caption) }
                            }
                            ForEach(episode.metadataWarnings, id: \.self) { Text($0).font(.caption).foregroundColor(.orange) }
                            HStack {
                                Button(episode.playableId == nil ? "Get & play" : "Play") { play(episode, show: show) }.buttonStyle(.borderedProminent)
                                Button("Queue") { play(episode, show: show, enqueue: true) }.buttonStyle(.bordered)
                                if episode.cleanedId != nil && episode.cleanedId != episode.originalId { Button("Original") { play(episode, show: show, original: true) }.font(.caption) }
                                if client.podcasts.job(for: episode)?.active != true { Button("Cut ads") { cutEpisode = episode }.buttonStyle(.borderless) }
                                if preparing == episode.id { ProgressView() }
                            }.disabled(preparing != nil)
                        }.padding(.vertical, 8).listRowBackground(HopperTheme.background)
                    }
                }.listStyle(.plain).scrollContentBackground(.hidden).hopperScene()
                    .navigationTitle(show.title).navigationBarTitleDisplayMode(.inline)
                    .toolbar {
                        Menu {
                            Button("Cut ads", systemImage: "scissors") { cutting = true }
                            Button("Show settings", systemImage: "gearshape") { settings = true }
                            Button("Check for new episodes", systemImage: "arrow.clockwise") { action(PodcastCommand(ownerId: show.owner.id, action: "refresh", showId: show.id)) }
                        } label: { Image(systemName: "ellipsis.circle") }.accessibilityLabel("Podcast settings")
                    }
                    .sheet(isPresented: $settings) { PodcastSettingsView(show: show) }
                    .sheet(item: $cutEpisode) { episode in CutAdsView(show: show, episode: episode) }
                    .sheet(isPresented: $cutting) { CutAdsView(show: show) }
                    .refreshable { action(PodcastCommand(ownerId: show.owner.id, action: "refresh", showId: show.id)) }
            } else { Text("This show is no longer followed.").foregroundColor(HopperTheme.muted) }
        }
    }
    private func action(_ command: PodcastCommand) {
        Task { do { let _ = try await client.podcast(command) } catch { client.error = error.localizedDescription } }
    }
    private func play(_ episode: PodcastEpisode, show: PodcastShow, enqueue: Bool = false, original: Bool = false) {
        preparing = episode.id
        Task {
            defer { preparing = nil }
            do {
                var id = original ? episode.originalId : episode.playableId
                if id == nil { id = try await client.podcast(PodcastCommand(ownerId: show.owner.id, action: "download", episodeId: episode.id)).mediaId }
                await client.refresh()
                guard let item = client.library.items.first(where: { $0.id == id }) else { throw NSError(domain: "Hopper", code: 1, userInfo: [NSLocalizedDescriptionKey: "The episode is being shared by its desktop. Refresh and try again."]) }
                if enqueue { player.enqueue(item) } else { await player.play(item) }
            } catch { client.error = error.localizedDescription }
        }
    }
}
