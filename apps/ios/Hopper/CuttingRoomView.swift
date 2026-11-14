import SwiftUI

struct CuttingRoomView: View {
    @EnvironmentObject var client: HopperClient
    private var jobs: [PodcastJob] {
        client.podcasts.jobs.sorted {
            if $0.active != $1.active { return $0.active }
            return $0.createdAt > $1.createdAt
        }
    }
    var body: some View {
        List {
            if jobs.isEmpty {
                Text("Choose an episode and tap Cut ads to send it here.")
                    .foregroundColor(HopperTheme.muted)
            }
            ForEach(jobs) { job in
                if let show = client.podcasts.shows.first(where: { $0.id == job.showId }) {
                    let episode = client.podcasts.episodes.first { $0.id == job.episodeId }
                    VStack(alignment: .leading, spacing: 9) {
                        NavigationLink { PodcastShowView(showId: show.id) } label: {
                            VStack(alignment: .leading, spacing: 4) {
                                Text(episode?.title ?? show.title).font(.headline)
                                Text("\(show.title) · \(show.owner.name)").font(.caption).foregroundColor(HopperTheme.muted)
                            }
                        }
                        Text(job.message).font(.subheadline)
                            .foregroundColor(job.state == "failed" ? .orange : HopperTheme.muted)
                        if job.removedSeconds > 0 {
                            Text("\(mediaTime(job.removedSeconds)) removed").font(.caption)
                        }
                        if job.active {
                            Button("Cancel") { command(show, job, action: "cancel") }.buttonStyle(.borderless)
                        } else if job.state == "failed" || job.state == "cancelled" {
                            Button("Retry") { command(show, job, action: "retry") }.buttonStyle(.borderless)
                        }
                    }.padding(.vertical, 6).listRowBackground(HopperTheme.background)
                }
            }
        }.listStyle(.plain).scrollContentBackground(.hidden).hopperScene()
            .navigationTitle("Cutting room")
            .refreshable { await client.refresh(sync: true) }
    }
    private func command(_ show: PodcastShow, _ job: PodcastJob, action: String) {
        Task {
            do { let _ = try await client.podcast(PodcastCommand(ownerId: show.owner.id, action: action, jobId: job.id)) }
            catch { client.error = error.localizedDescription }
        }
    }
}
