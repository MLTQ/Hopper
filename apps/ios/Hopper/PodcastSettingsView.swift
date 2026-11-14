import SwiftUI

struct PodcastSettingsView: View {
    let show: PodcastShow
    @EnvironmentObject var client: HopperClient
    @Environment(\.dismiss) var dismiss
    @State private var newestFirst = false
    var body: some View {
        NavigationStack {
            Form {
                Section("Listening") { Picker("Episode order", selection: $newestFirst) { Text("Oldest first").tag(false); Text("Newest first").tag(true) } }
                Section { Button("Save settings") {
                    Task { do { let _ = try await client.podcast(PodcastCommand(ownerId: show.owner.id, action: "settings", showId: show.id, newestFirst: newestFirst, cutOptions: show.cutOptions)); dismiss() } catch { client.error = error.localizedDescription } }
                }.disabled(client.working) }
                Section { Button("Unsubscribe", role: .destructive) {
                    Task { do { let _ = try await client.podcast(PodcastCommand(ownerId: show.owner.id, action: "unsubscribe", showId: show.id)); dismiss() } catch { client.error = error.localizedDescription } }
                } } footer: { Text("Downloaded originals and cleaned episodes stay in your library.") }
            }.navigationTitle("Show settings").toolbar { Button("Done") { dismiss() } }.onAppear { newestFirst = show.newestFirst }
        }
    }
}

struct CutAdsView: View {
    let show: PodcastShow
    var episode: PodcastEpisode? = nil
    @EnvironmentObject var client: HopperClient
    @Environment(\.dismiss) var dismiss
    @State private var options = CutOptions()
    @State private var count = 1
    private var episodes: [PodcastEpisode] {
        if let episode { return [episode] }
        return client.podcasts.episodes(for: show).filter { episode in
            !client.library.items.contains { $0.id == episode.playableId && $0.progress?.completed == true }
        }
    }
    private var connected: Bool { client.podcasts.hosts.first { $0.id == show.owner.id }?.agentAvailable == true }
    var body: some View {
        NavigationStack {
            Form {
                Section {
                    Toggle("Keep show promos", isOn: $options.keepPromos)
                } footer: { Text("Show promos include trailers and announcements for other shows.") }
                Section("Edit style") {
                    Picker("Fade at cuts", selection: $options.fadeMs) { Text("Off — direct stitch").tag(0); Text("Short · 20 ms").tag(20); Text("Soft · 100 ms").tag(100); Text("Long · 250 ms").tag(250) }
                    Picker("When uncertain", selection: $options.minimumConfidence) { Text("Keep it").tag(0.9); Text("Keep more").tag(0.98); Text("Allow more cuts").tag(0.75) }
                }
                if let episode { Section("Episode to prepare") { Text(episode.title) } }
                else { Section("Episodes to prepare") {
                    Picker("Unplayed episodes", selection: $count) { Text("Next episode").tag(1); Text("Next 5").tag(5); Text("Next 10").tag(10); Text("All · up to 500").tag(500) }
                    Toggle("Also clean future episodes", isOn: $options.autoProcess)
                } }
                Section {
                    Text("Runs on \(show.owner.name). Original files are kept. Chapters, supported transcripts and listening position follow the edited timeline.").font(.footnote).foregroundColor(HopperTheme.muted)
                    if !connected { Text("The ad-cutting agent is not connected on this desktop yet. Jobs will wait until it is available.").font(.footnote).foregroundColor(.orange) }
                    Button {
                        Task {
                            do {
                                let command: PodcastCommand
                                if let episode { command = PodcastCommand(ownerId: show.owner.id, action: "cut_episode", episodeId: episode.id, options: options) }
                                else { command = PodcastCommand(ownerId: show.owner.id, action: "cut", showId: show.id, episodeIds: Array(episodes.prefix(count)).map(\.id), options: options) }
                                let _ = try await client.podcast(command); dismiss()
                            }
                            catch { client.error = error.localizedDescription }
                        }
                    } label: { if client.working { ProgressView() } else { Text(episode != nil ? "Send to cutting room" : "Send \(min(count, episodes.count)) to cutting room") } }.disabled(client.working || episodes.isEmpty)
                }
            }.navigationTitle("Cut ads").toolbar { Button("Cancel") { dismiss() } }.onAppear { options = show.cutOptions }
        }
    }
}
