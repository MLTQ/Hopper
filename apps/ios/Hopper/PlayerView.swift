import SwiftUI
import AVKit

struct QueueView: View {
    @EnvironmentObject var player: HopperPlayer
    var body: some View {
        NavigationStack {
            List {
                if player.queuedItems.isEmpty { VStack(spacing: 15) { Image(systemName: "text.line.first.and.arrowtriangle.forward").font(.largeTitle); Text("A little room for more.").font(.system(size: 24, design: .serif)); Text("Add a few things from your library.\nWe’ll keep them moving.").font(.subheadline).multilineTextAlignment(.center) }.foregroundColor(HopperTheme.muted).frame(maxWidth: .infinity).padding(.vertical, 80).listRowBackground(HopperTheme.background).listRowSeparator(.hidden) }
                ForEach(player.queuedItems) { item in Button { Task { await player.play(item) } } label: { HStack { if item.id == player.currentId { Image(systemName: "waveform").foregroundColor(HopperTheme.forest) }; MediaRow(item: item) } }.buttonStyle(.plain).listRowBackground(HopperTheme.background) }
                    .onDelete { player.remove(at: $0) }.onMove { player.move(from: $0, to: $1) }
            }.listStyle(.plain).scrollContentBackground(.hidden).hopperScene().navigationTitle("Up next").toolbar { EditButton() }
        }
    }
}

struct MiniPlayer: View {
    @EnvironmentObject var player: HopperPlayer
    let open: () -> Void
    var body: some View {
        if let item = player.current {
            HStack(spacing: 12) {
                Button(action: open) { HStack(spacing: 12) { MediaArt(item: item, size: 40); VStack(alignment: .leading, spacing: 4) { Text(item.title).font(.subheadline.weight(.semibold)).lineLimit(1); Text(player.loading ? "Connecting…" : item.subtitle).font(.caption).foregroundColor(HopperTheme.muted).lineLimit(1) }; Spacer() } }.buttonStyle(.plain)
                Button { player.toggle() } label: { Image(systemName: player.playing ? "pause.fill" : "play.fill").font(.title3).frame(width: 40, height: 40) }.accessibilityLabel(player.playing ? "Pause" : "Play")
                Button { player.next() } label: { Image(systemName: "forward.end.fill").frame(width: 30, height: 40) }.accessibilityLabel("Next item")
            }.padding(.horizontal, 17).padding(.vertical, 9).hopperGlass()
        }
    }
}

struct PlayerView: View {
    @EnvironmentObject var player: HopperPlayer
    @Environment(\.dismiss) var dismiss
    @State private var scrubbing = false
    @State private var scrubPosition = 0.0
    var body: some View {
        NavigationStack {
            ScrollView {
                if let item = player.current {
                    VStack(spacing: 25) {
                        Text("IN YOUR OWN WORLD").font(.system(size: 10, weight: .semibold)).tracking(2).foregroundColor(HopperTheme.muted).padding(.top, 20)
                        if item.kind == "video" { VideoPlayer(player: player.engine).frame(height: 230) }
                        else { MediaArt(item: item, size: 260).shadow(color: .black.opacity(0.08), radius: 20, y: 10) }
                        VStack(spacing: 8) { Text(item.title).font(.system(size: 27, weight: .regular, design: .serif)).multilineTextAlignment(.center); Text(item.subtitle).font(.subheadline).foregroundColor(HopperTheme.muted).multilineTextAlignment(.center) }
                        VStack(spacing: 8) {
                            Slider(value: Binding(get: { scrubbing ? scrubPosition : min(player.position, max(player.duration, 1)) }, set: { scrubPosition = $0 }), in: 0...max(player.duration, 1)) { editing in scrubbing = editing; if !editing { player.seek(scrubPosition) } }
                            HStack { Text(mediaTime(player.position)); Spacer(); Text(mediaTime(player.duration)) }.font(.caption.monospacedDigit()).foregroundColor(HopperTheme.muted)
                        }
                        HStack(spacing: 30) {
                            Button { player.next(-1) } label: { Image(systemName: "backward.end.fill").font(.title3) }.accessibilityLabel("Previous item")
                            Button { player.skip(-15) } label: { Image(systemName: "gobackward.15").font(.title) }.accessibilityLabel("Back 15 seconds")
                            Button { player.toggle() } label: { Image(systemName: player.playing ? "pause.fill" : "play.fill").font(.title).frame(width: 68, height: 68).foregroundColor(.white).background(HopperTheme.accent).clipShape(Circle()) }.accessibilityLabel(player.playing ? "Pause" : "Play")
                            Button { player.skip(30) } label: { Image(systemName: "goforward.30").font(.title) }.accessibilityLabel("Forward 30 seconds")
                            Button { player.next() } label: { Image(systemName: "forward.end.fill").font(.title3) }.accessibilityLabel("Next item")
                        }
                        HStack {
                            Menu { ForEach([Float(0.75), 1, 1.25, 1.5, 1.75, 2], id: \.self) { speed in Button("\(speed.formatted())×") { player.rate = speed } } } label: { Text("\(player.rate.formatted())×").font(.subheadline.weight(.semibold)).padding(10).hopperGlass(radius: 8) }
                            Spacer(); Text(item.isLocal ? "ON THIS DEVICE" : "STREAMING FROM YOUR DEVICES").font(.system(size: 9)).tracking(1).foregroundColor(HopperTheme.muted)
                        }
                        if !item.chapters.isEmpty {
                            VStack(alignment: .leading, spacing: 14) { Text("CHAPTERS").font(.caption).tracking(2).foregroundColor(HopperTheme.muted); ForEach(item.chapters) { chapter in Button { player.seek(chapter.start) } label: { HStack { Text(chapter.title); Spacer(); Text(mediaTime(chapter.start)).font(.caption) }.padding(.vertical, 9) }; Divider() } }
                        }
                    }.padding(.horizontal, 26).padding(.bottom, 40)
                }
            }.hopperScene().toolbar { ToolbarItem(placement: .navigationBarLeading) { Button { dismiss() } label: { Image(systemName: "chevron.down") }.accessibilityLabel("Close player") }; ToolbarItem(placement: .principal) { Text("hopper.").font(.system(size: 21, weight: .semibold, design: .serif)) } }
        }
    }
}
