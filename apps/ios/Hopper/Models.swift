import Foundation

struct Device: Codable, Identifiable { let id: String; let name: String }
struct Hello: Decodable { let port: Int; let token: String; let device: Device }
struct Source: Decodable { let deviceId: String; let name: String; let local: Bool; let online: Bool }
struct Peer: Decodable, Identifiable { let id: String; let name: String; let online: Bool; let error: String? }
struct Chapter: Decodable, Identifiable {
    let title: String; let start: Double; let end: Double?
    let artworkUrl: String?; let hidden: Bool?
    var id: String { "\(start)-\(title)" }
}
struct MediaProgress: Decodable {
    let position: Double; let duration: Double?; let completed: Bool; let updatedAt: UInt64
}
struct MediaItem: Decodable, Identifiable {
    let id: String; let title: String; let creator: String; let album: String
    let kind: String; let filename: String; let folder: String; let mime: String
    let bytes: UInt64; let duration: Double?; let chapters: [Chapter]
    let sources: [Source]; let progress: MediaProgress?
    let automaticDownload: Bool?
    var canKeep: Bool { !isLocal || automaticDownload == true }
    var isLocal: Bool { sources.contains { $0.local && $0.online } }
    var subtitle: String { !creator.isEmpty ? creator : !folder.isEmpty ? folder : filename }
    var symbol: String { kind == "video" ? "play.rectangle.fill" : kind == "audiobook" ? "book.closed.fill" : "waveform" }
    var initials: String { title.split(separator: " ").prefix(2).compactMap(\.first).map(String.init).joined().uppercased() }
}
struct QueueState: Codable { var items: [String] = []; var current: String? }
struct Library: Decodable {
    let device: Device; let items: [MediaItem]; let peers: [Peer]; let queue: QueueState
    var home: Device? = nil
    var deliveries: [HomeDelivery]? = nil
    var podcasts: PodcastCatalogue? = nil
    static let empty = Library(device: Device(id: "", name: "This iPhone"), items: [], peers: [], queue: QueueState())
}
struct APIError: Decodable { let error: String }
struct Ack: Decodable { let ok: Bool }
struct Invite: Decodable { let code: String; let expiresIn: Int }
struct PlaybackLocation: Decodable { let path: String?; let localPath: String? }
struct PairBody: Encodable { let code: String }
struct ImportBody: Encodable { let paths: [String] }
struct ImportReport: Decodable { let added: Int; let skipped: Int; let errors: [String] }
struct EmptyBody: Encodable {}
struct ProgressBody: Codable {
    let itemId: String; let position: Double; let duration: Double?; let completed: Bool
}

func mediaTime(_ seconds: Double) -> String {
    guard seconds.isFinite else { return "0:00" }
    let total = max(0, Int(seconds)), hours = total / 3600, minutes = total / 60 % 60
    return hours > 0 ? String(format: "%d:%02d:%02d", hours, minutes, total % 60) : String(format: "%d:%02d", minutes, total % 60)
}
