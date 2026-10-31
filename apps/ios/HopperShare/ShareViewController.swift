import UIKit
import UniformTypeIdentifiers

final class ShareViewController: UIViewController {
    private let message = UILabel()
    private let save = UIButton(type: .system)
    private var payload: (String, String)?
    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .systemBackground
        let title = UILabel(); title.text = "Put it in the hopper"; title.font = .preferredFont(forTextStyle: .title1)
        message.text = "Reading shared podcast…"; message.numberOfLines = 0; message.font = .preferredFont(forTextStyle: .body)
        save.setTitle("Save to Hopper", for: .normal); save.isEnabled = false
        save.addTarget(self, action: #selector(store), for: .touchUpInside)
        let cancel = UIButton(type: .system); cancel.setTitle("Cancel", for: .normal); cancel.addTarget(self, action: #selector(close), for: .touchUpInside)
        let stack = UIStackView(arrangedSubviews: [title, message, save, cancel]); stack.axis = .vertical; stack.spacing = 24; stack.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(stack)
        NSLayoutConstraint.activate([stack.leadingAnchor.constraint(equalTo: view.layoutMarginsGuide.leadingAnchor), stack.trailingAnchor.constraint(equalTo: view.layoutMarginsGuide.trailingAnchor), stack.centerYAnchor.constraint(equalTo: view.centerYAnchor)])
        Task { await readInput() }
    }
    @MainActor private func readInput() async {
        let providers = (extensionContext?.inputItems as? [NSExtensionItem] ?? []).flatMap { $0.attachments ?? [] }
        for provider in providers {
            for type in [UTType.url, .fileURL, .xml, .plainText] where provider.hasItemConformingToTypeIdentifier(type.identifier) {
                do {
                    let item = try await load(provider, type.identifier)
                    if let url = item as? URL {
                        if url.isFileURL { payload = ("opml", try ShareInbox.readOPML(url)) }
                        else if ["http", "https"].contains(url.scheme?.lowercased() ?? "") { payload = ("url", url.absoluteString) }
                    } else if let text = (item as? String) ?? (item as? Data).flatMap({ String(data: $0, encoding: .utf8) }) {
                        guard text.utf8.count <= 2 * 1024 * 1024 else { continue }
                        if text.range(of: "<opml", options: .caseInsensitive) != nil { payload = ("opml", text) }
                        else {
                            let detector = try NSDataDetector(types: NSTextCheckingResult.CheckingType.link.rawValue)
                            if let url = detector.firstMatch(in: text, range: NSRange(text.startIndex..., in: text))?.url, ["http", "https"].contains(url.scheme?.lowercased() ?? "") { payload = ("url", url.absoluteString) }
                        }
                    }
                    if payload != nil {
                        message.text = "Save this podcast to Hopper, then open Hopper to send it to your Home desktop. Pending additions stay saved until they are accepted."
                        save.isEnabled = true; return
                    }
                } catch { message.text = error.localizedDescription }
            }
        }
        message.text = "Share an Apple Podcasts or RSS link, or an OPML subscription export. You can also paste a link directly in Hopper."
    }
    private func load(_ provider: NSItemProvider, _ type: String) async throws -> NSSecureCoding? {
        try await withCheckedThrowingContinuation { continuation in
            provider.loadItem(forTypeIdentifier: type, options: nil) { item, error in
                if let error { continuation.resume(throwing: error) } else { continuation.resume(returning: item) }
            }
        }
    }
    @objc private func store() {
        guard let payload else { return }
        do { try ShareInbox.save(kind: payload.0, value: payload.1); close() }
        catch { message.text = error.localizedDescription }
    }
    @objc private func close() { extensionContext?.completeRequest(returningItems: nil) }
}
