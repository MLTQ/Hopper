import SwiftUI

private struct BrowseRequest: Encodable, Hashable {
    let provider: String
    let term: String
    let category: String
    let ownerId: String?
}
private struct DirectoryCategory: Identifiable, Decodable {
    let id: String
    let name: String
}
private struct DirectoryShow: Decodable, Identifiable {
    let title: String
    let author: String
    let url: String
    let artwork: String?
    var id: String { url }
}
private struct DirectoryPage: Decodable {
    let shows: [DirectoryShow]
    let categories: [DirectoryCategory]
}

struct PodcastDirectoryView: View {
    @EnvironmentObject var client: HopperClient
    @State private var provider = "apple"
    @State private var term = ""
    @State private var submitted = ""
    @State private var category = ""
    @State private var owner = ""
    @State private var categories: [DirectoryCategory] = []
    @State private var shows: [DirectoryShow] = []
    @State private var pending: Set<[String]> = []
    @State private var loading = true
    @State private var failure: String?
    @State private var retry = 0

    private var hosts: [PodcastHost] { client.podcasts.hosts.filter { $0.desktop } }
    private var destination: String? { owner.isEmpty ? client.library.home?.id ?? hosts.first?.id : owner }
    private var request: BrowseRequest {
        BrowseRequest(provider: provider, term: submitted, category: category, ownerId: destination)
    }

    var body: some View {
        List {
            Section {
                Picker("Directory", selection: Binding(get: { provider }, set: { value in
                    category = ""; categories = []; provider = value
                })) {
                    Text("Apple Podcasts · US").tag("apple")
                    Text("Podcast Index").tag("podcast_index")
                }
                Picker("Category", selection: $category) {
                    Text("All categories").tag("")
                    ForEach(categories) { Text($0.name).tag($0.id) }
                }
                if hosts.isEmpty { Text("Pair a desktop in Devices to subscribe.").font(.footnote) }
                else {
                    Picker("Subscribe on", selection: Binding(get: { destination ?? "" }, set: { owner = $0 })) {
                        ForEach(hosts) { host in Text(host.device.name + (host.online ? "" : " · offline")).tag(host.id) }
                    }
                }
            }
            Section(submitted.isEmpty ? (provider == "apple" ? "Top podcasts" : "Trending podcasts") : "Search results") {
                if loading { ProgressView("Looking up podcasts…") }
                else if let failure {
                    Text(failure)
                    Button("Try again") { retry += 1 }
                } else if shows.isEmpty { Text("No shows found. Try another category or search.") }
                ForEach(shows) { show in
                    HStack(spacing: 12) {
                        PodcastCover(urls: [show.artwork].compactMap { $0 }, size: 48) {
                            HopperTheme.surface.overlay(Image(systemName: "mic.fill"))
                        }
                        VStack(alignment: .leading, spacing: 3) {
                            Text(show.title).font(.headline)
                            if !show.author.isEmpty { Text(show.author).font(.caption).foregroundStyle(.secondary) }
                        }
                        Spacer()
                        let followed = client.podcasts.shows.contains { $0.owner.id == destination && $0.feedUrl == show.url }
                        let queued = pending.contains([destination ?? "", show.url]) || client.library.deliveries?.contains {
                            $0.owner.id == destination && ["waiting","sending"].contains($0.state) && $0.payload?.url == show.url
                        } == true
                        Button { subscribe(show) } label: {
                            Image(systemName: followed ? "checkmark" : queued ? "clock" : "plus.circle")
                        }.buttonStyle(.borderless).accessibilityLabel(followed ? "Subscribed" : queued ? "Queued" : "Subscribe to \(show.title)")
                            .disabled(destination == nil || followed || queued)
                    }
                }
            }
        }.navigationTitle("Browse podcasts")
            .searchable(text: $term, placement: .navigationBarDrawer(displayMode: .always), prompt: "Show or publisher")
            .onSubmit(of: .search) { submitted = term.trimmingCharacters(in: .whitespacesAndNewlines); retry += 1 }
            .onChange(of: term) { text in if text.isEmpty { submitted = "" } }
            .task(id: SearchID(request: request, retry: retry)) {
                loading = true; failure = nil; shows = []
                do {
                    let page: DirectoryPage = try await client.post("podcasts/directory", request)
                    try Task.checkCancellation()
                    categories = page.categories; shows = page.shows; loading = false
                } catch {
                    if !Task.isCancelled { failure = error.localizedDescription; loading = false }
                }
            }
    }

    private struct SearchID: Hashable { let request: BrowseRequest; let retry: Int }
    private func subscribe(_ show: DirectoryShow) {
        guard let destination else { return }
        let key = [destination, show.url]
        guard pending.insert(key).inserted else { return }
        Task {
            do {
                let _: AddedPodcasts = try await client.post("add-podcasts", AddPodcastsBody(ownerId: destination, url: show.url))
                await client.refresh()
            } catch { pending.remove(key); client.error = error.localizedDescription }
        }
    }
}
