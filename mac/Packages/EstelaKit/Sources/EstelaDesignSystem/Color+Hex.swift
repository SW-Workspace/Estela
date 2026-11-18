import SwiftUI

extension Color {
    private static let channelMax: Double = 255

    init(hex: UInt32) {
        self.init(
            .sRGB,
            red: Double((hex >> 16) & 0xFF) / Self.channelMax,
            green: Double((hex >> 8) & 0xFF) / Self.channelMax,
            blue: Double(hex & 0xFF) / Self.channelMax
        )
    }
}
