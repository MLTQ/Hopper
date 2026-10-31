import Foundation

extension HopperClient {
    func drainShareInbox() async {
        guard ready, !drainingShares else { return }
        drainingShares = true; defer { drainingShares = false }
        do {
            let pending = try ShareInbox.pending()
            guard !pending.isEmpty else { return }
            guard library.home != nil else {
                if !shareNoticeShown { error = "Choose your Home desktop in Devices to finish importing your shared podcasts."; shareNoticeShown = true }
                return
            }
            var failed: String?
            for (file, item) in pending {
                do {
                guard item.kind == "url" || item.kind == "opml" else { throw ShareInbox.failure("Unrecognized shared podcast. Import its link directly in Podcasts.") }
                let body = AddPodcastsBody(url: item.kind == "url" ? item.value : nil, opml: item.kind == "opml" ? item.value : nil, batchId: item.id)
                let result: AddedPodcasts = try await post("add-podcasts", body)
                try FileManager.default.removeItem(at: file)
                if !result.warnings.isEmpty { error = result.warnings.joined(separator: " ") }
                } catch { failed = error.localizedDescription }
            }
            if let failed {
                if !shareNoticeShown { error = failed + " Retry or discard the shared import in Devices." }
                shareNoticeShown = true
            } else { shareNoticeShown = false }
            await refresh()
        } catch {
            if !shareNoticeShown { self.error = error.localizedDescription; shareNoticeShown = true }
        }
    }
}
