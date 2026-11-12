import AVFoundation
import Foundation

extension HopperPlayer {
    /// Start one full transfer per playing item. AVQueuePlayer's look-ahead only prepares streaming URLs.
    func playbackDownload(_ id: String) -> Task<URL, Error> {
        if let existing = downloadTasks[id] { return existing }
        downloadStatus = "Downloading…"
        let task = Task { try await client.downloadForPlayback(id) }
        downloadTasks[id] = task
        Task {
            do {
                _ = try await task.value
                if currentId == id { downloadStatus = nil }
                await client.refresh()
            } catch {
                if currentId == id { downloadStatus = "Download unavailable" }
            }
            downloadTasks.removeValue(forKey: id)
        }
        return task
    }

    func handleFailure(_ entry: AVPlayerItem) {
        guard engine.currentItem === entry, let id = currentId, recovering !== entry else { return }
        guard let url = (entry.asset as? AVURLAsset)?.url, !url.isFileURL else {
            loading = false
            client.error = entry.error?.localizedDescription ?? "This file could not play."
            return
        }
        recovering = entry
        loading = true
        let task = playbackDownload(id)
        Task {
            do {
                let local = try await task.value
                await replaceFailedEntry(entry, with: local, id: id)
            } catch {
                guard engine.currentItem === entry, recovering === entry else { return }
                loading = false
                let hosts = client.mediaById[id]?.sources.filter { !$0.local }.map(\.name).joined(separator: ", ") ?? ""
                let source = hosts.isEmpty ? "your desktop" : hosts
                client.error = "Couldn’t play this file from \(source). \(error.localizedDescription)"
            }
        }
    }
}
