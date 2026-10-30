import SwiftUI

struct DevicesView: View {
    @EnvironmentObject var client: HopperClient
    @State private var code = ""
    @State private var invitation: String?
    @State private var creating = false
    @State private var removing: Peer?
    var body: some View {
        NavigationStack {
            List {
                Section {
                    Label { VStack(alignment: .leading) { Text(client.library.device.name); Text("This device").font(.caption).foregroundColor(HopperTheme.muted) } } icon: { Image(systemName: "iphone").foregroundColor(HopperTheme.forest) }
                    ForEach(client.library.peers) { peer in
                        HStack {
                            Image(systemName: "desktopcomputer").foregroundColor(HopperTheme.muted)
                            VStack(alignment: .leading, spacing: 5) { Text(peer.name); Text(peer.online ? "Connected" : "Offline").font(.caption).foregroundColor(HopperTheme.muted) }
                            Spacer(); Circle().fill(peer.online ? HopperTheme.forest : .gray.opacity(0.4)).frame(width: 7, height: 7)
                        }.swipeActions { Button("Unpair", role: .destructive) { removing = peer } }
                    }
                } header: { Text("Your devices") } footer: { Text("Pair directly with each desktop you want to access. Keep it running to stream, or download a copy to listen offline.") }
                Section {
                    TextEditor(text: $code).frame(minHeight: 90).font(.system(.caption, design: .monospaced)).autocorrectionDisabled().textInputAutocapitalization(.never)
                    HStack {
                        PasteButton(payloadType: String.self) { values in if let value = values.first { code = value } }
                        Spacer()
                        Button { Task { await client.pair(code); if client.error == nil { code = "" } } } label: { if client.working { ProgressView() } else { Text("Pair device") } }.buttonStyle(.borderedProminent).disabled(code.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || client.working || !client.ready)
                    }
                } header: { Text("Paste an invitation") } footer: { Text("On your desktop, choose Pair a device → Create invitation. Pairing shares Hopper media and listening progress between these devices.") }
                Section {
                    Button { Task { creating = true; defer { creating = false }; do { let result: Invite = try await client.post("invite", EmptyBody()); invitation = result.code } catch { client.error = error.localizedDescription } } } label: { if creating { ProgressView() } else { Label("Create an invitation", systemImage: "link") } }.disabled(!client.ready || creating)
                    if let invitation {
                        Text(invitation).font(.system(size: 9, design: .monospaced)).textSelection(.enabled).lineLimit(6)
                        ShareLink(item: invitation) { Label("Share invitation", systemImage: "square.and.arrow.up") }
                        Text("Works once. Expires in ten minutes.").font(.caption).foregroundColor(HopperTheme.muted)
                    }
                }
            }.navigationTitle("Your devices").scrollContentBackground(.hidden).background(HopperTheme.background)
                .refreshable { await client.refresh(sync: true) }
                .confirmationDialog("Unpair this device?", isPresented: Binding(get: { removing != nil }, set: { if !$0 { removing = nil } })) {
                    Button("Unpair", role: .destructive) { if let peer = removing { Task { await client.unpair(peer.id) } }; removing = nil }
                } message: { Text("Downloaded copies remain available. This device will stop accepting requests from that peer.") }
        }
    }
}
