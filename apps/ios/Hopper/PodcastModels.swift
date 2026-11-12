import Foundation

struct CutOptions: Codable {
    var keepBanter = true
    var keepPromos = true
    var fadeMs = 20
    var minimumConfidence = 0.9
    var autoProcess = false
}
struct PodcastShow: Decodable, Identifiable {
    let id: String; let owner: Device; let title: String; let author: String; let description: String
    let artworkUrl: String?
    let feedUrl: String?
    let newestFirst: Bool; let cutOptions: CutOptions; let error: String?
}
struct PodcastEpisode: Decodable, Identifiable {
    let id: String; let showId: String; let title: String; let description: String
    let published: Int64; let duration: Double?; let originalId: String?; let cleanedId: String?
    let artworkUrl: String?; let chaptersUrl: String?
    let metadataWarnings: [String]
    var playableId: String? { cleanedId ?? originalId }
}
struct PodcastJob: Decodable, Identifiable {
    let id: String; let episodeId: String; let showId: String; let state: String; let message: String
    let createdAt: UInt64; let removedSeconds: Double
    var active: Bool { ["queued", "downloading", "analysing", "rendering", "waiting_for_agent"].contains(state) }
}
struct PodcastHost: Decodable, Identifiable {
    let device: Device; let online: Bool; let desktop: Bool; let agentAvailable: Bool
    var id: String { device.id }
}
struct PodcastCatalogue: Decodable {
    let shows: [PodcastShow]; let episodes: [PodcastEpisode]; let jobs: [PodcastJob]; let hosts: [PodcastHost]
    static let empty = PodcastCatalogue(shows: [], episodes: [], jobs: [], hosts: [])
    func episodes(for show: PodcastShow) -> [PodcastEpisode] {
        episodes.filter { $0.showId == show.id }.sorted { show.newestFirst ? $0.published > $1.published : $0.published < $1.published }
    }
    func job(for episode: PodcastEpisode) -> PodcastJob? {
        jobs.filter { $0.episodeId == episode.id }.max { $0.createdAt < $1.createdAt }
    }
}
struct PodcastCommand: Encodable {
    var ownerId: String; var action: String
    var url: String?; var showId: String?; var episodeId: String?; var episodeIds: [String]?
    var newestFirst: Bool?; var cutOptions: CutOptions?; var options: CutOptions?; var jobId: String?
}
struct PodcastResult: Decodable { let showId: String?; let mediaId: String?; let jobIds: [String] }

extension HopperClient {
    var podcasts: PodcastCatalogue { library.podcasts ?? .empty }
    func podcast(_ command: PodcastCommand) async throws -> PodcastResult {
        working = true; defer { working = false }
        let result: PodcastResult = try await post("podcasts/command", command)
        await refresh()
        return result
    }
}
