import MediaPlayer
import UIKit

extension HopperPlayer {
    var playbackChapters: [Chapter] { supplementalChapters ?? current?.chapters ?? [] }

    func updateArtwork(for item: MediaItem) {
        let episode = client.episodesByMedia[item.id]
        let chapterKey = [item.id, episode?.chaptersUrl ?? ""]
        if chapterRequest != chapterKey {
            chapterTask?.cancel()
            chapterRequest = chapterKey
            supplementalChapters = nil
            // Older hosts have the timing list but omit images, or cached a failed chapter request.
            if let episode, episode.originalId == item.id, episode.chaptersUrl != nil,
               !item.chapters.contains(where: { $0.artworkUrl != nil }) {
                chapterTask = Task {
                    guard let chapters: [Chapter] = try? await client.request("podcasts/chapters/\(episode.id)"),
                          !Task.isCancelled, currentId == item.id else { return }
                    supplementalChapters = chapters
                    nowPlaying()
                }
            }
        }
        let active = playbackChapters.last { $0.start <= position && ($0.end ?? .infinity) > position }
        let urls = [active?.artworkUrl].compactMap { $0 } + client.coverURLs(for: episode)
        guard artworkRequest != urls || artworkMedia != item.id else { return }
        artworkTask?.cancel()
        if artworkMedia != item.id { playbackCover = nil; systemCover = nil }
        artworkMedia = item.id
        artworkRequest = urls
        artworkTask = Task {
            let image = await client.artwork.image(from: urls, hello: client.hello)
            guard !Task.isCancelled, currentId == item.id else { return }
            playbackCover = image
            systemCover = image.map { image in MPMediaItemArtwork(boundsSize: image.size) { _ in image } }
            nowPlaying()
        }
    }
}
