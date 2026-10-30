import Foundation

@MainActor
final class PlaybackJournal {
    private struct Entry: Codable { let sequence: UUID; let body: ProgressBody }
    private var pending: [String: Entry] = [:]
    private var task: Task<Void, Never>?
    private let file: URL?
    private let client: HopperClient

    init(client: HopperClient) {
        self.client = client
        file = try? FileManager.default.url(for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true).appendingPathComponent("playback-checkpoints.json")
        if let file, let data = try? Data(contentsOf: file), let saved = try? JSONDecoder().decode([String: Entry].self, from: data) { pending = saved }
    }
    func record(_ body: ProgressBody) {
        pending[body.itemId] = Entry(sequence: UUID(), body: body)
        persist(); flush()
    }
    func flushAndWait() async { flush(); await task?.value }
    func pendingProgress(for id: String) -> ProgressBody? { pending[id]?.body }
    func flush() {
        guard task == nil, client.ready else { return }
        task = Task { [weak self] in
            guard let self else { return }
            defer { self.task = nil }
            while let (id, entry) = self.pending.first {
                do {
                    let _: MediaProgress = try await self.client.post("progress", entry.body)
                    if self.pending[id]?.sequence == entry.sequence { self.pending.removeValue(forKey: id); self.persist() }
                } catch { return } // Retain the durable checkpoint for the next attempt.
            }
        }
    }
    private func persist() {
        guard let file else { return }
        do { try JSONEncoder().encode(pending).write(to: file, options: [.atomic, .completeFileProtectionUntilFirstUserAuthentication]) }
        catch { client.error = "Could not save your listening position: \(error.localizedDescription)" }
    }
}
