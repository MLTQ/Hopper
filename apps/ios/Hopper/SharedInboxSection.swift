import SwiftUI

struct SharedInboxSection: View {
    @EnvironmentObject var client: HopperClient
    @State private var pending: [(URL, SharedPodcast)] = []
    var body: some View {
        Group {
            if !pending.isEmpty {
                Section {
                    ForEach(pending, id: \.1.id) { file, item in
                        VStack(alignment: .leading, spacing: 6) {
                            Text(item.kind == "opml" ? "Shared subscription list" : item.value).font(.caption).lineLimit(2)
                            Button("Discard this shared import", role: .destructive) {
                                do { try FileManager.default.removeItem(at: file); reload() }
                                catch { client.error = error.localizedDescription }
                            }.font(.caption)
                        }
                    }
                    Button("Try shared imports again") { Task { client.shareNoticeShown = false; await client.drainShareInbox(); reload() } }
                } header: { Text("Shared imports waiting") } footer: { Text("Choose Home to finish these imports. If a shared file or link is invalid, you can discard it here.") }
            }
        }.task { reload() }.onChange(of: client.library.home?.id) { _ in reload() }
    }
    private func reload() { pending = (try? ShareInbox.pending()) ?? [] }
}
