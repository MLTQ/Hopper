import Foundation

struct SharedPodcast: Codable, Identifiable {
    let id: String
    let kind: String
    let value: String
}
enum ShareInbox {
    static let group = "group.app.hopper.player"
    static var container: URL? { FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: group) }
    static func folder() throws -> URL {
        guard let root = container else {
            throw failure("Hopper sharing is not configured. Open Hopper and paste a podcast link or import an OPML file.")
        }
        let url = root.appendingPathComponent("PodcastInbox", isDirectory: true)
        try FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
        return url
    }
    static func save(kind: String, value: String) throws {
        guard value.utf8.count <= 2 * 1024 * 1024 else { throw failure("Choose a subscription list smaller than 2 MB.") }
        let root = try folder()
        guard try FileManager.default.contentsOfDirectory(at: root, includingPropertiesForKeys: nil).count < 500 else {
            throw failure("Open Hopper to finish your pending imports first.")
        }
        let item = SharedPodcast(id: UUID().uuidString, kind: kind, value: value)
        try JSONEncoder().encode(item).write(to: root.appendingPathComponent(item.id + ".json"), options: .atomic)
    }
    static func pending() throws -> [(URL, SharedPodcast)] {
        guard container != nil else { return [] }
        return try FileManager.default.contentsOfDirectory(at: folder(), includingPropertiesForKeys: nil).filter { $0.pathExtension == "json" }.sorted { $0.lastPathComponent < $1.lastPathComponent }.map { url in
            let data = try boundedData(url, limit: 16 * 1024 * 1024)
            return (url, try JSONDecoder().decode(SharedPodcast.self, from: data))
        }
    }
    static func readOPML(_ url: URL) throws -> String {
        let access = url.startAccessingSecurityScopedResource()
        defer { if access { url.stopAccessingSecurityScopedResource() } }
        let data = try boundedData(url)
        guard let text = String(data: data, encoding: .utf8), text.range(of: "<opml", options: .caseInsensitive) != nil else {
            throw failure("Choose an OPML subscription export.")
        }
        return text
    }
    private static func boundedData(_ url: URL, limit: Int = 2 * 1024 * 1024) throws -> Data {
        let file = try FileHandle(forReadingFrom: url); defer { try? file.close() }
        let data = try file.read(upToCount: limit + 1) ?? Data()
        guard data.count <= limit else { throw failure("Choose a subscription list smaller than 2 MB.") }
        return data
    }
    static func failure(_ message: String) -> Error { NSError(domain: "HopperShare", code: 1, userInfo: [NSLocalizedDescriptionKey: message]) }
}
