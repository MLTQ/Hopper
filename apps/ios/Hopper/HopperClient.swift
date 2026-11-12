import Foundation
import UIKit

@MainActor
final class HopperClient: ObservableObject {
    let artwork = PodcastArtworkStore()
    private(set) var episodesByMedia: [String: PodcastEpisode] = [:]
    private(set) var showsById: [String: PodcastShow] = [:]
    private(set) var mediaById: [String: MediaItem] = [:]
    @Published var library = Library.empty {
        didSet {
            mediaById = Dictionary(library.items.map { ($0.id, $0) }, uniquingKeysWith: { first, _ in first })
            showsById = Dictionary(podcasts.shows.map { ($0.id, $0) }, uniquingKeysWith: { first, _ in first })
            episodesByMedia.removeAll(keepingCapacity: true)
            for episode in podcasts.episodes {
                if let id = episode.originalId { episodesByMedia[id] = episode }
                if let id = episode.cleanedId { episodesByMedia[id] = episode }
            }
        }
    }
    func coverURLs(for episode: PodcastEpisode?) -> [String] {
        guard let episode else { return [] }
        return [episode.artworkUrl, showsById[episode.showId]?.artworkUrl].compactMap { $0 }
    }
    @Published var ready = false
    @Published var error: String?
    @Published var working = false
    private(set) var hello: Hello?
    var drainingShares = false
    var shareNoticeShown = false
    private var starting = false
    private var polling: Task<Void, Never>?
    let decoder: JSONDecoder = { let d = JSONDecoder(); d.keyDecodingStrategy = .convertFromSnakeCase; return d }()
    let encoder: JSONEncoder = { let e = JSONEncoder(); e.keyEncodingStrategy = .convertToSnakeCase; return e }()

    func start() async {
        guard !starting else { return }; starting = true
        do {
            let root = try FileManager.default.url(for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true).appendingPathComponent("Hopper", isDirectory: true)
            let name = UIDevice.current.name, path = root.path
            let data: Data = try await Task.detached(priority: .userInitiated) {
                try path.withCString { directory in
                    try name.withCString { deviceName in
                        guard let pointer = hopper_start(directory, deviceName) else { throw NSError(domain: "Hopper", code: 1, userInfo: [NSLocalizedDescriptionKey: "The library could not start."]) }
                        defer { hopper_free(pointer) }
                        return Data(String(cString: pointer).utf8)
                    }
                }
            }.value
            if let failure = try? decoder.decode(APIError.self, from: data) { throw failureError(failure.error) }
            hello = try decoder.decode(Hello.self, from: data)
            ready = true
            await refresh()
            await drainShareInbox()
            polling = Task { [weak self] in
                while !Task.isCancelled {
                    try? await Task.sleep(nanoseconds: 15_000_000_000)
                    guard let self else { return }; await self.refresh()
                }
            }
        } catch { self.error = error.localizedDescription; starting = false }
    }

    func request<T: Decodable>(_ path: String, method: String = "GET", body: Data? = nil) async throws -> T {
        guard let hello, let url = URL(string: "http://127.0.0.1:\(hello.port)/api/\(path)") else { throw failureError("The library is still starting.") }
        var request = URLRequest(url: url); request.httpMethod = method; request.httpBody = body
        request.timeoutInterval = path.hasPrefix("keep/") || path == "podcasts/command" ? 3700 : 90
        request.setValue("Bearer \(hello.token)", forHTTPHeaderField: "Authorization")
        if body != nil { request.setValue("application/json", forHTTPHeaderField: "Content-Type") }
        let (data, response) = try await URLSession.shared.data(for: request)
        guard let http = response as? HTTPURLResponse, (200..<300).contains(http.statusCode) else {
            throw failureError((try? decoder.decode(APIError.self, from: data).error) ?? "The device did not respond.")
        }
        return try decoder.decode(T.self, from: data)
    }
    func post<T: Decodable, B: Encodable>(_ path: String, _ body: B) async throws -> T {
        try await request(path, method: "POST", body: encoder.encode(body))
    }
    func refresh(sync: Bool = false) async {
        guard ready else { return }
        do { library = sync ? try await post("refresh", EmptyBody()) : try await request("library") }
        catch { self.error = error.localizedDescription }
    }
    func mediaURL(_ id: String) async throws -> URL {
        let grant: MediaGrant = try await post("play/\(id)", EmptyBody())
        guard let hello, let url = URL(string: "http://127.0.0.1:\(hello.port)/\(grant.path)") else { throw failureError("Invalid playback location") }
        return url
    }
    func pair(_ code: String) async {
        working = true; defer { working = false }
        do { library = try await post("pair", PairBody(code: code)) } catch { self.error = error.localizedDescription }
    }
    func keep(_ item: MediaItem) async {
        working = true; defer { working = false }
        do { let _: Ack = try await post("keep/\(item.id)", EmptyBody()); await refresh() } catch { self.error = error.localizedDescription }
    }
    func unpair(_ id: String) async {
        do { let _: Ack = try await request("peers/\(id)", method: "DELETE"); await refresh() } catch { self.error = error.localizedDescription }
    }
    func importFiles(_ urls: [URL]) async {
        working = true; defer { working = false }
        do {
            let root = try FileManager.default.url(for: .documentDirectory, in: .userDomainMask, appropriateFor: nil, create: true).appendingPathComponent("Imports", isDirectory: true)
            try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
            var paths: [String] = []
            for url in urls {
                let access = url.startAccessingSecurityScopedResource(); defer { if access { url.stopAccessingSecurityScopedResource() } }
                let folder = root.appendingPathComponent(UUID().uuidString, isDirectory: true)
                try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
                let dest = folder.appendingPathComponent(url.lastPathComponent)
                try FileManager.default.copyItem(at: url, to: dest); paths.append(dest.path)
            }
            let report: ImportReport = try await post("import", ImportBody(paths: paths))
            if let message = report.errors.first { error = message }
            await refresh()
        } catch { self.error = error.localizedDescription }
    }
    private func failureError(_ message: String) -> Error { NSError(domain: "Hopper", code: 1, userInfo: [NSLocalizedDescriptionKey: message]) }
}
