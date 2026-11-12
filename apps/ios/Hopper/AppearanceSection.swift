import SwiftUI

/// Device-local appearance preferences; each system appearance remembers its own tint.
struct AppearanceSection: View {
    @AppStorage("hopper.glass.light") private var lightTint = 1.0
    @AppStorage("hopper.glass.dark") private var darkTint = 1.0
    var body: some View {
        Section {
            VStack(alignment: .leading) {
                HStack { Text("Light glass tint"); Spacer(); Text(lightTint, format: .percent.precision(.fractionLength(0))).foregroundColor(HopperTheme.muted).monospacedDigit() }
                Slider(value: $lightTint, in: 0...1.5, step: 0.05).accessibilityLabel("Light glass tint")
            }
            VStack(alignment: .leading) {
                HStack { Text("Dark glass tint"); Spacer(); Text(darkTint, format: .percent.precision(.fractionLength(0))).foregroundColor(HopperTheme.muted).monospacedDigit() }
                Slider(value: $darkTint, in: 0...1.5, step: 0.05).accessibilityLabel("Dark glass tint")
            }
            Button("Reset appearance") { lightTint = 1; darkTint = 1 }
        } header: { Text("Appearance") } footer: {
            Text("Glass follows your device’s appearance. Reduce Transparency uses solid surfaces.")
        }
    }
}
