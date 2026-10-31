import SwiftUI
import UniformTypeIdentifiers

struct SubscribePodcastView: View {
    @EnvironmentObject var client: HopperClient
    @Environment(\.dismiss) var dismiss
    @State private var url = ""
    @State private var owner = ""
    @State private var importing = false
    @State private var sending = false
    @State private var failure: String?
    private var hosts: [PodcastHost] { client.podcasts.hosts.filter { $0.desktop } }
    var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextField("https://podcasts.apple.com/…", text: $url).keyboardType(.URL).textInputAutocapitalization(.never).autocorrectionDisabled()
                    Button("Put in the hopper") { send(AddPodcastsBody(ownerId: owner, url: url)) }.disabled(url.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || owner.isEmpty || sending)
                } header: { Text("Apple Podcasts or RSS link") } footer: { Text("Share a show or episode link from Apple Podcasts, or paste the publisher’s RSS feed.") }
                Section {
                    Button("Import subscription list") { importing = true }.disabled(owner.isEmpty || sending)
                } header: { Text("Castamatic") } footer: { Text("Export your subscriptions as OPML from Castamatic, save the file, then choose it here. Subscriptions transfer; listening history and downloads stay in your old app.") }
                Section {
                    if hosts.isEmpty { Text("Pair a desktop in Devices first.") }
                    else { Picker("Send to", selection: $owner) { ForEach(hosts) { Text($0.device.name + ($0.online ? "" : " · offline")).tag($0.id) } } }
                } footer: { Text("Additions wait on this iPhone until the desktop receives them. Open Hopper to resume pending transfers.") }
                if sending { ProgressView("Adding to your hopper…") }
                if let failure { Text(failure).foregroundColor(.orange) }
            }.navigationTitle("Bring your podcasts").navigationBarTitleDisplayMode(.inline)
                .toolbar { Button("Done") { dismiss() } }
                .onAppear { owner = client.library.home?.id ?? hosts.first?.id ?? "" }
                .fileImporter(isPresented: $importing, allowedContentTypes: [.data, .xml], allowsMultipleSelection: false) { result in
                    do { if let file = try result.get().first { let xml = try ShareInbox.readOPML(file); send(AddPodcastsBody(ownerId: owner, opml: xml)) } }
                    catch { failure = error.localizedDescription }
                }
        }
    }
    private func send(_ body: AddPodcastsBody) {
        sending = true; failure = nil
        Task {
            defer { sending = false }
            do {
                let result: AddedPodcasts = try await client.post("add-podcasts", body)
                await client.refresh()
                if result.warnings.isEmpty { dismiss() } else { failure = "Added \(result.deliveryIds.count) subscriptions. " + result.warnings.joined(separator: " ") }
            } catch { failure = error.localizedDescription }
        }
    }
}
