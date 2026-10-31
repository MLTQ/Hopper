import SwiftUI

struct HomeDelivery: Decodable, Identifiable {
    let id: String; let owner: Device; let label: String; let state: String; let message: String
}
struct HomeBody: Encodable { let deviceId: String }
struct AddPodcastsBody: Encodable {
    var ownerId: String? = nil; var url: String? = nil; var opml: String? = nil; var batchId: String? = nil
}
struct AddedPodcasts: Decodable { let deliveryIds: [String]; let warnings: [String] }

struct HomeSection: View {
    @EnvironmentObject var client: HopperClient
    var body: some View {
        Section {
            Picker("Home desktop", selection: Binding(get: { client.library.home?.id ?? "" }, set: { id in
                guard !id.isEmpty else { return }
                Task {
                    do { let _: Ack = try await client.post("home", HomeBody(deviceId: id)); await client.refresh(); await client.drainShareInbox() }
                    catch { client.error = error.localizedDescription }
                }
            })) {
                Text("Choose a desktop").tag("")
                ForEach(client.podcasts.hosts.filter { $0.desktop }) { host in
                    Text(host.device.name + (host.online ? "" : " · offline")).tag(host.id)
                }
            }
        } header: { Text("Home") } footer: {
            Text("New subscriptions and files go to this desktop. Keep Hopper running there to receive additions and prepare audio. Existing shows stay on their current desktop.")
        }
    }
}
struct DeliverySection: View {
    @EnvironmentObject var client: HopperClient
    var body: some View {
        if let jobs = client.library.deliveries, !jobs.isEmpty {
            Section("Sending to Home") {
                ForEach(jobs) { job in
                    VStack(alignment: .leading, spacing: 5) {
                        Text(job.label + " · " + job.owner.name).font(.subheadline)
                        Text(job.message).font(.caption).foregroundColor(job.state == "failed" ? .orange : .secondary)
                        if job.state == "waiting" || job.state == "failed" {
                            Button("Retry now") { Task {
                                do { let _: Ack = try await client.post("deliveries/\(job.id)/retry", EmptyBody()); await client.refresh() }
                                catch { client.error = error.localizedDescription }
                            } }.font(.caption)
                        }
                    }.padding(.vertical, 4)
                }
            }
        }
    }
}
