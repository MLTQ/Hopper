import AVFoundation
import Combine
import MediaPlayer
import UIKit
import SwiftUI

@MainActor
final class HopperPlayer: ObservableObject {
    let engine = AVQueuePlayer()
    @Published var queue: [String] = []
    @Published var currentId: String?
    @Published var position = 0.0
    @Published var duration = 0.0
    @Published var playing = false
    @Published var loading = false
    @Published var rate: Float = 1 { didSet { if playing { engine.rate = rate }; nowPlaying() } }
    private let client: HopperClient
    private let journal: PlaybackJournal
    private var observation: NSKeyValueObservation?
    private var statusObservation: NSKeyValueObservation?
    private var itemStatus: NSKeyValueObservation?
    private var periodic: Any?
    private var observers: [NSObjectProtocol] = []
    private var mediaIds: [ObjectIdentifier: String] = [:]
    private var generation = UUID()
    private var lastCheckpoint = Date.distantPast
    private var resumeAfterInterruption = false
    private var queueWrites: Task<Void, Never>?
    private var restored = false
    private var filling = false
    private var needsFill = false
    private var queueRevision = UUID()

    var current: MediaItem? { client.library.items.first { $0.id == currentId } }
    var queuedItems: [MediaItem] { queue.compactMap { id in client.library.items.first { $0.id == id } } }

    init(client: HopperClient) {
        self.client = client; journal = PlaybackJournal(client: client)
        engine.automaticallyWaitsToMinimizeStalling = true
        observation = engine.observe(\.currentItem, options: [.new]) { [weak self] _, _ in Task { @MainActor in self?.currentChanged() } }
        statusObservation = engine.observe(\.timeControlStatus, options: [.new]) { [weak self] player, _ in
            Task { @MainActor in self?.playing = player.timeControlStatus == .playing; self?.loading = player.timeControlStatus == .waitingToPlayAtSpecifiedRate; self?.nowPlaying() }
        }
        periodic = engine.addPeriodicTimeObserver(forInterval: CMTime(seconds: 1, preferredTimescale: 600), queue: .main) { [weak self] time in
            Task { @MainActor in
                guard let self, self.engine.currentItem != nil else { return }
                self.position = time.seconds.isFinite ? time.seconds : 0
                let d = self.engine.currentItem?.duration.seconds ?? 0
                self.duration = d.isFinite ? d : self.current?.duration ?? 0
                if self.playing && Date().timeIntervalSince(self.lastCheckpoint) > 10 { self.checkpoint() }
                self.nowPlaying()
            }
        }
        observers.append(NotificationCenter.default.addObserver(forName: .AVPlayerItemDidPlayToEndTime, object: nil, queue: .main) { [weak self] note in
            guard let item = note.object as? AVPlayerItem else { return }
            Task { @MainActor in
                guard let self, let id = self.mediaIds[ObjectIdentifier(item)] else { return }
                let d = item.duration.seconds
                self.journal.record(ProgressBody(itemId: id, position: d.isFinite ? d : 0, duration: d.isFinite ? d : nil, completed: true))
                await self.fillQueue()
            }
        })
        observers.append(NotificationCenter.default.addObserver(forName: AVAudioSession.interruptionNotification, object: nil, queue: .main) { [weak self] note in
            let type = note.userInfo?[AVAudioSessionInterruptionTypeKey] as? UInt
            let options = note.userInfo?[AVAudioSessionInterruptionOptionKey] as? UInt ?? 0
            Task { @MainActor in
                guard let self else { return }
                if type == AVAudioSession.InterruptionType.began.rawValue { self.resumeAfterInterruption = self.playing; self.checkpoint(); self.engine.pause() }
                else if self.resumeAfterInterruption && AVAudioSession.InterruptionOptions(rawValue: options).contains(.shouldResume) { self.resumeAfterInterruption = false; self.resume() }
            }
        })
        observers.append(NotificationCenter.default.addObserver(forName: AVAudioSession.routeChangeNotification, object: nil, queue: .main) { [weak self] note in
            if note.userInfo?[AVAudioSessionRouteChangeReasonKey] as? UInt == AVAudioSession.RouteChangeReason.oldDeviceUnavailable.rawValue { Task { @MainActor in self?.pause() } }
        })
        configureCommands()
    }

    func restore() {
        journal.flush()
        guard !restored, client.ready else { return }; restored = true
        let saved = UserDefaults.standard.data(forKey: "hopper-queue").flatMap { try? JSONDecoder().decode(QueueState.self, from: $0) } ?? client.library.queue
        queue = saved.items; currentId = saved.current
        position = current?.progress?.position ?? 0; duration = current?.duration ?? 0
    }
    func play(_ item: MediaItem) async {
        checkpoint(); engine.pause()
        generation = UUID(); let token = generation
        engine.removeAllItems(); mediaIds.removeAll(); currentId = item.id; loading = true
        if !queue.contains(item.id) { queue.append(item.id) }
        persistQueue()
        do {
            await journal.flushAndWait()
            await client.refresh()
            guard generation == token else { return }
            try activateAudio()
            let url = try await client.mediaURL(item.id)
            guard generation == token else { return }
            let entry = AVPlayerItem(url: url); mediaIds[ObjectIdentifier(entry)] = item.id
            engine.insert(entry, after: nil)
            let progress = client.library.items.first { $0.id == item.id }?.progress
            let pending = journal.pendingProgress(for: item.id)
            position = pending.map { $0.completed ? 0 : $0.position } ?? (progress?.completed == false ? progress?.position ?? 0 : 0)
            if position > 0 { await engine.seek(to: CMTime(seconds: position, preferredTimescale: 600)) }
            guard generation == token else { return }
            engine.playImmediately(atRate: rate)
            await fillQueue()
        } catch { loading = false; client.error = error.localizedDescription }
    }
    func toggle() {
        if playing { pause() }
        else if engine.currentItem == nil { if let item = current ?? queuedItems.first { Task { await play(item) } } }
        else { resume() }
    }
    func pause() { checkpoint(); engine.pause(); playing = false; try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation); nowPlaying() }
    func resume() { do { try activateAudio(); engine.playImmediately(atRate: rate) } catch { client.error = error.localizedDescription } }
    func seek(_ seconds: Double) {
        let target = max(0, min(seconds, duration > 0 ? duration : seconds))
        engine.seek(to: CMTime(seconds: target, preferredTimescale: 600)) { [weak self] finished in
            Task { @MainActor in if finished { self?.position = target; self?.checkpoint(); self?.nowPlaying() } }
        }
    }
    func skip(_ by: Double) { seek(position + by) }
    func next(_ offset: Int = 1) {
        guard let currentId, let i = queue.firstIndex(of: currentId), queue.indices.contains(i + offset), let item = client.library.items.first(where: { $0.id == queue[i + offset] }) else { return }
        Task { await play(item) }
    }
    func enqueue(_ item: MediaItem) { if !queue.contains(item.id) { queue.append(item.id); persistQueue(); Task { await fillQueue() } } }
    func playAll(_ items: [MediaItem]) { queue = items.map(\.id); if let first = items.first { Task { await play(first) } } else { persistQueue() } }
    func remove(at offsets: IndexSet) {
        let visible = queuedItems
        let removed = Set(offsets.compactMap { visible.indices.contains($0) ? visible[$0].id : nil })
        if let id = currentId, removed.contains(id) { pause(); generation = UUID(); engine.removeAllItems(); currentId = nil }
        queue.removeAll { removed.contains($0) }; trimUpcoming(); persistQueue(); Task { await fillQueue() }
    }
    func move(from: IndexSet, to: Int) {
        var visible = queuedItems.map(\.id); let visibleIds = Set(visible)
        visible.move(fromOffsets: from, toOffset: to)
        queue = visible + queue.filter { !visibleIds.contains($0) }
        trimUpcoming(); persistQueue(); Task { await fillQueue() }
    }
    private func trimUpcoming() { queueRevision = UUID(); for item in engine.items() where item !== engine.currentItem { engine.remove(item) } }
    func checkpoint() {
        guard let item = engine.currentItem, let id = mediaIds[ObjectIdentifier(item)] else { return }
        let pos = item.currentTime().seconds, d = item.duration.seconds
        guard pos.isFinite && pos >= 0 else { return }
        lastCheckpoint = Date()
        journal.record(ProgressBody(itemId: id, position: pos, duration: d.isFinite ? d : nil, completed: false))
    }
    private func currentChanged() {
        guard let item = engine.currentItem, let id = mediaIds[ObjectIdentifier(item)] else { playing = false; nowPlaying(); return }
        currentId = id; position = 0; duration = current?.duration ?? 0; persistQueue(); nowPlaying()
        Task { await fillQueue() }
        itemStatus = item.observe(\.status, options: [.new]) { [weak self] entry, _ in
            if entry.status == .failed { let message = entry.error?.localizedDescription ?? "This media could not play."; Task { @MainActor in self?.client.error = message; self?.loading = false } }
        }
    }
    private func fillQueue() async {
        needsFill = true
        guard !filling else { return }; filling = true
        defer { filling = false }
        while needsFill {
            needsFill = false
            guard let currentId, let index = queue.firstIndex(of: currentId), engine.currentItem != nil else { return }
            let token = generation, revision = queueRevision
            for id in queue.dropFirst(index + 1).prefix(3) {
                if engine.items().contains(where: { mediaIds[ObjectIdentifier($0)] == id }) { continue }
                do {
                    let url = try await client.mediaURL(id)
                    guard generation == token, queueRevision == revision else { needsFill = true; break }
                    if engine.items().contains(where: { mediaIds[ObjectIdentifier($0)] == id }) { continue }
                    let item = AVPlayerItem(url: url); mediaIds[ObjectIdentifier(item)] = id
                    engine.insert(item, after: engine.items().last)
                } catch { client.error = error.localizedDescription; return }
            }
        }
    }
    private func activateAudio() throws {
        let session = AVAudioSession.sharedInstance()
        try session.setCategory(.playback, mode: current?.kind == "audiobook" ? .spokenAudio : .default)
        try session.setActive(true)
    }
    private func persistQueue() {
        let state = QueueState(items: queue, current: currentId)
        if let data = try? JSONEncoder().encode(state) { UserDefaults.standard.set(data, forKey: "hopper-queue") }
        let previous = queueWrites
        queueWrites = Task { [weak self] in
            await previous?.value
            guard let self else { return }
            do { let _: Ack = try await self.client.post("queue", state) } catch { self.client.error = error.localizedDescription }
        }
    }
    private func nowPlaying() {
        guard let item = current else { MPNowPlayingInfoCenter.default().nowPlayingInfo = nil; return }
        var info: [String: Any] = [MPMediaItemPropertyTitle: item.title, MPMediaItemPropertyArtist: item.creator,
            MPMediaItemPropertyAlbumTitle: item.album, MPNowPlayingInfoPropertyElapsedPlaybackTime: position,
            MPNowPlayingInfoPropertyPlaybackRate: playing ? rate : 0, MPNowPlayingInfoPropertyDefaultPlaybackRate: rate,
            MPNowPlayingInfoPropertyMediaType: item.kind == "video" ? MPNowPlayingInfoMediaType.video.rawValue : MPNowPlayingInfoMediaType.audio.rawValue]
        if duration > 0 { info[MPMediaItemPropertyPlaybackDuration] = duration }
        MPNowPlayingInfoCenter.default().nowPlayingInfo = info
    }
    private func configureCommands() {
        let commands = MPRemoteCommandCenter.shared()
        commands.playCommand.addTarget { [weak self] _ in Task { @MainActor in self?.resume() }; return .success }
        commands.pauseCommand.addTarget { [weak self] _ in Task { @MainActor in self?.pause() }; return .success }
        commands.togglePlayPauseCommand.addTarget { [weak self] _ in Task { @MainActor in self?.toggle() }; return .success }
        commands.nextTrackCommand.addTarget { [weak self] _ in Task { @MainActor in self?.next() }; return .success }
        commands.previousTrackCommand.addTarget { [weak self] _ in Task { @MainActor in self?.next(-1) }; return .success }
        commands.skipBackwardCommand.preferredIntervals = [15]
        commands.skipBackwardCommand.addTarget { [weak self] _ in Task { @MainActor in self?.skip(-15) }; return .success }
        commands.skipForwardCommand.preferredIntervals = [30]
        commands.skipForwardCommand.addTarget { [weak self] _ in Task { @MainActor in self?.skip(30) }; return .success }
        commands.changePlaybackPositionCommand.addTarget { [weak self] event in
            guard let event = event as? MPChangePlaybackPositionCommandEvent else { return .commandFailed }
            let position = event.positionTime; Task { @MainActor in self?.seek(position) }; return .success
        }
    }
}
