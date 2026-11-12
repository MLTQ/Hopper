import SwiftUI
import ImageIO

@MainActor
final class PodcastArtworkStore {
    private let images = NSCache<NSString, UIImage>()
    private let failures = NSCache<NSString, NSDate>()
    private let disk = URLCache(memoryCapacity: 0, diskCapacity: 80 * 1024 * 1024, diskPath: "PodcastCovers")
    private var pending: [String: Task<UIImage?, Never>] = [:]

    init() { images.totalCostLimit = 32 * 1024 * 1024; failures.countLimit = 256 }

    func image(from candidates: [String], hello: Hello?) async -> UIImage? {
        guard let hello else { return nil }
        for key in candidates {
            if Task.isCancelled { return nil }
            if let image = images.object(forKey: key as NSString) { return image }
            if let failure = failures.object(forKey: key as NSString), failure.timeIntervalSinceNow > 0 { continue }
            guard let url = URL(string: key), ["https", "http"].contains(url.scheme?.lowercased() ?? "") else { continue }
            if let task = pending[key] {
                if let image = await task.value { return image }
                continue
            }
            let task = Task { () -> UIImage? in
                let source = URLRequest(url: url)
                if let cached = disk.cachedResponse(for: source),
                   let date = cached.userInfo?["cachedAt"] as? Date, Date().timeIntervalSince(date) < 86400,
                   let image = Self.thumbnail(cached.data) { return image }
                guard let endpoint = URL(string: "http://127.0.0.1:\(hello.port)/api/podcasts/artwork") else { return nil }
                var request = URLRequest(url: endpoint)
                request.httpMethod = "POST"
                request.setValue("Bearer " + hello.token, forHTTPHeaderField: "Authorization")
                request.setValue("application/json", forHTTPHeaderField: "Content-Type")
                request.httpBody = try? JSONEncoder().encode(["url": key])
                guard let (data, response) = try? await URLSession.shared.data(for: request),
                      (response as? HTTPURLResponse)?.statusCode == 200, let image = Self.thumbnail(data) else {
                    return disk.cachedResponse(for: source).flatMap { Self.thumbnail($0.data) }
                }
                let result = URLResponse(url: url, mimeType: nil, expectedContentLength: data.count, textEncodingName: nil)
                disk.storeCachedResponse(CachedURLResponse(response: result, data: data, userInfo: ["cachedAt": Date()], storagePolicy: .allowed), for: source)
                return image
            }
            pending[key] = task
            let image = await task.value
            pending.removeValue(forKey: key)
            if let image {
                images.setObject(image, forKey: key as NSString, cost: Int(image.size.width * image.size.height * 4))
                return image
            }
            failures.setObject(Date(timeIntervalSinceNow: 45) as NSDate, forKey: key as NSString)
        }
        return nil
    }

    private static func thumbnail(_ data: Data) -> UIImage? {
        let options: [CFString: Any] = [kCGImageSourceCreateThumbnailFromImageAlways: true,
            kCGImageSourceCreateThumbnailWithTransform: true, kCGImageSourceThumbnailMaxPixelSize: 768]
        guard let source = CGImageSourceCreateWithData(data as CFData, nil),
              let cg = CGImageSourceCreateThumbnailAtIndex(source, 0, options as CFDictionary) else { return nil }
        return UIImage(cgImage: cg)
    }
}

struct PodcastCover<Placeholder: View>: View {
    @EnvironmentObject var client: HopperClient
    let urls: [String]
    let size: CGFloat
    @ViewBuilder let placeholder: () -> Placeholder
    @State private var image: UIImage?
    var body: some View {
        ZStack {
            if let image { Image(uiImage: image).resizable().scaledToFill() }
            else { placeholder() }
        }.frame(width: size, height: size).clipShape(RoundedRectangle(cornerRadius: size / 10))
            .accessibilityHidden(true)
            .task(id: urls) {
                image = nil
                let loaded = await client.artwork.image(from: urls, hello: client.hello)
                if !Task.isCancelled { image = loaded }
            }
    }
}
