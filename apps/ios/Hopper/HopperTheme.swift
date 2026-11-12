import SwiftUI
import UIKit

/// Shared scene, accessible ink and material; no library or playback state lives here.
enum HopperTheme {
    private static func adaptive(_ light: UInt32, _ dark: UInt32) -> Color {
        Color(UIColor { traits in
            let rgb = traits.userInterfaceStyle == .dark ? dark : light
            return UIColor(red: CGFloat((rgb >> 16) & 255) / 255, green: CGFloat((rgb >> 8) & 255) / 255, blue: CGFloat(rgb & 255) / 255, alpha: 1)
        })
    }
    static let background = adaptive(0xf7f8f2, 0x101c18)
    static let forest = adaptive(0x315843, 0xb5d2ae)
    static let accent = adaptive(0x315843, 0x527650)
    static let muted = adaptive(0x5d6c60, 0xb3c1b3)
    static let surface = adaptive(0xe9efdf, 0x23352a)
    static let ink = adaptive(0x263a31, 0xedf2e9)

    static var scene: some View {
        GeometryReader { geometry in
            ZStack {
                background
                RadialGradient(colors: [forest.opacity(0.18), .clear], center: .topLeading, startRadius: 0, endRadius: geometry.size.height * 0.8)
                RadialGradient(colors: [Color(red: 0.83, green: 0.64, blue: 0.43).opacity(0.16), .clear], center: .topTrailing, startRadius: 0, endRadius: geometry.size.height * 0.65)
                RadialGradient(colors: [Color(red: 0.42, green: 0.66, blue: 0.61).opacity(0.18), .clear], center: .bottom, startRadius: 0, endRadius: geometry.size.height * 0.65)
            }
        }.ignoresSafeArea().allowsHitTesting(false).accessibilityHidden(true)
    }
}

private struct HopperGlass: ViewModifier {
    var radius: CGFloat
    @Environment(\.colorScheme) private var scheme
    @Environment(\.accessibilityReduceTransparency) private var reduceTransparency
    @Environment(\.colorSchemeContrast) private var contrast
    @AppStorage("hopper.glass.light") private var lightTint = 1.0
    @AppStorage("hopper.glass.dark") private var darkTint = 1.0

    func body(content: Content) -> some View {
        let dark = scheme == .dark
        let tint = min(1.5, max(0, dark ? darkTint : lightTint))
        let shape = RoundedRectangle(cornerRadius: radius)
        content.background {
            if reduceTransparency || contrast == .increased {
                shape.fill(HopperTheme.background)
            } else {
                shape.fill(.ultraThinMaterial)
                    .overlay(shape.fill((dark ? Color(red: 0.08, green: 0.08, blue: 0.1) : .white).opacity((dark ? 0.38 : 0.22) * tint)))
                    .overlay(shape.fill(RadialGradient(colors: [.white.opacity(dark ? 0.1 : 0.3), .clear], center: .topLeading, startRadius: 0, endRadius: 220)))
                    .overlay(shape.strokeBorder(LinearGradient(colors: [.white.opacity(dark ? 0.32 : 0.95), .white.opacity(dark ? 0.08 : 0.3), .white.opacity(dark ? 0.18 : 0.55)], startPoint: .topLeading, endPoint: .bottomTrailing), lineWidth: 0.7))
                    .shadow(color: .black.opacity(dark ? 0.16 : 0.08), radius: 12, y: 4)
            }
        }.foregroundColor(HopperTheme.ink)
    }
}

extension View {
    func hopperGlass(radius: CGFloat = 0) -> some View { modifier(HopperGlass(radius: radius)) }
    func hopperScene() -> some View { background { HopperTheme.scene } }
}
