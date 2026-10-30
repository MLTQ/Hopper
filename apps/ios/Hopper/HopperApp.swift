import SwiftUI

@main
struct HopperApp: App {
    @StateObject private var client: HopperClient
    @StateObject private var player: HopperPlayer
    @Environment(\.scenePhase) private var phase
    @State private var showPlayer = false

    init() {
        let client = HopperClient()
        _client = StateObject(wrappedValue: client)
        _player = StateObject(wrappedValue: HopperPlayer(client: client))
    }
    var body: some Scene {
        WindowGroup {
            TabView {
                LibraryView().safeAreaInset(edge: .bottom, spacing: 0) { MiniPlayer { showPlayer = true } }
                    .tabItem { Label("Library", systemImage: "square.stack") }
                QueueView().safeAreaInset(edge: .bottom, spacing: 0) { MiniPlayer { showPlayer = true } }
                    .tabItem { Label("Up next", systemImage: "text.line.first.and.arrowtriangle.forward") }
                DevicesView().safeAreaInset(edge: .bottom, spacing: 0) { MiniPlayer { showPlayer = true } }
                    .tabItem { Label("Devices", systemImage: "desktopcomputer") }
            }
            .tint(HopperTheme.forest)
            .preferredColorScheme(.light)
            .environmentObject(client).environmentObject(player)
            .sheet(isPresented: $showPlayer) { PlayerView().environmentObject(player).environmentObject(client) }
            .alert("Hopper", isPresented: Binding(get: { client.error != nil }, set: { if !$0 { client.error = nil } })) { Button("OK") { client.error = nil } } message: { Text(client.error ?? "") }
            .task { await client.start(); player.restore() }
            .onChange(of: phase) { new in
                if new == .active { Task { await client.refresh(sync: true); player.restore() } }
                else { player.checkpoint() }
            }
        }
    }
}
