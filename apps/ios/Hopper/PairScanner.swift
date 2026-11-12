import SwiftUI
@preconcurrency import AVFoundation

struct PairScanner: UIViewControllerRepresentable {
    let onInvitation: (String) -> Void

    func makeUIViewController(context: Context) -> ScannerController {
        let controller = ScannerController()
        controller.onInvitation = onInvitation
        return controller
    }
    func updateUIViewController(_ controller: ScannerController, context: Context) {}
}

final class ScannerController: UIViewController, AVCaptureMetadataOutputObjectsDelegate {
    var onInvitation: ((String) -> Void)?
    private let session = AVCaptureSession()
    private let cameraQueue = DispatchQueue(label: "app.hopper.pairing.camera")
    private lazy var preview = AVCaptureVideoPreviewLayer(session: session)
    private let hint = UILabel()
    private var active = false
    private var consumed = false

    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .black
        preview.videoGravity = .resizeAspectFill
        view.layer.addSublayer(preview)
        hint.text = "Point the camera at the QR code in Hopper on your desktop."
        hint.textColor = .white
        hint.backgroundColor = .black.withAlphaComponent(0.72)
        hint.textAlignment = .center
        hint.numberOfLines = 0
        hint.font = .preferredFont(forTextStyle: .body)
        hint.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(hint)
        NSLayoutConstraint.activate([
            hint.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            hint.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            hint.bottomAnchor.constraint(equalTo: view.safeAreaLayoutGuide.bottomAnchor),
            hint.heightAnchor.constraint(greaterThanOrEqualToConstant: 64)
        ])
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()
        preview.frame = view.bounds
        if let orientation = view.window?.windowScene?.interfaceOrientation,
           let video = AVCaptureVideoOrientation(rawValue: orientation.rawValue),
           preview.connection?.isVideoOrientationSupported == true {
            preview.connection?.videoOrientation = video
        }
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        active = true
        Task { [weak self] in
            let allowed = await AVCaptureDevice.requestAccess(for: .video)
            guard let self, active else { return }
            guard allowed else {
                hint.text = "Allow camera access in Settings to scan, or close this screen and paste an invitation."
                return
            }
            let session = self.session
            cameraQueue.async { [weak self] in
                if session.inputs.isEmpty {
                    guard let camera = AVCaptureDevice.default(for: .video),
                          let input = try? AVCaptureDeviceInput(device: camera),
                          session.canAddInput(input) else { self?.cameraUnavailable(); return }
                    session.beginConfiguration()
                    session.addInput(input)
                    let output = AVCaptureMetadataOutput()
                    guard session.canAddOutput(output) else {
                        session.removeInput(input); session.commitConfiguration(); self?.cameraUnavailable(); return
                    }
                    session.addOutput(output)
                    guard output.availableMetadataObjectTypes.contains(.qr) else {
                        session.removeOutput(output); session.removeInput(input); session.commitConfiguration(); self?.cameraUnavailable(); return
                    }
                    output.setMetadataObjectsDelegate(self, queue: .main)
                    output.metadataObjectTypes = [.qr]
                    session.commitConfiguration()
                }
                session.startRunning()
            }
        }
    }

    nonisolated private func cameraUnavailable() {
        Task { @MainActor in hint.text = "Camera unavailable. Close this screen and paste the desktop invitation." }
    }

    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)
        active = false
        let session = self.session
        cameraQueue.async { session.stopRunning() }
    }

    func metadataOutput(_ output: AVCaptureMetadataOutput, didOutput objects: [AVMetadataObject], from connection: AVCaptureConnection) {
        guard active, !consumed else { return }
        let codes = objects.compactMap { ($0 as? AVMetadataMachineReadableCodeObject)?.stringValue }
        guard let code = codes.first(where: { $0.hasPrefix("hopper1:") && $0.utf8.count <= 16384 }) else {
            hint.text = "Scan the invitation from Hopper’s Pair a device screen."
            return
        }
        consumed = true
        onInvitation?(code)
    }
}

struct ScanPairButton: View {
    @EnvironmentObject var client: HopperClient
    @State private var scanning = false
    @State private var pendingScan: String?

    var body: some View {
        Button { scanning = true } label: { Label("Scan pairing QR code", systemImage: "qrcode.viewfinder") }
            .disabled(!client.ready || client.working)
            .sheet(isPresented: $scanning, onDismiss: {
                    if let invitation = pendingScan {
                        pendingScan = nil
                        Task { await client.pair(invitation) }
                    }
                }) {
                    NavigationStack {
                        PairScanner { invitation in
                            pendingScan = invitation
                            scanning = false
                        }
                        .navigationTitle("Scan desktop code").navigationBarTitleDisplayMode(.inline)
                        .toolbar { ToolbarItem(placement: .cancellationAction) { Button("Cancel") { scanning = false } } }
                    }
                }
    }
}
