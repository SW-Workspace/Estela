import SwiftUI

public enum Palette {
    public static let accent = Color(hex: 0x7C5CFF)
    public static let background = Color(hex: 0x0E0F12)
    public static let border = Color(hex: 0x2A2E36)
    public static let surface = Color(hex: 0x17191E)
    public static let surfaceRaised = Color(hex: 0x1F2228)
    public static let textPrimary = Color(hex: 0xF2F3F5)
    public static let textSecondary = Color(hex: 0xA1A6B0)
}

public enum Spacing {
    public static let lg: CGFloat = 24
    public static let md: CGFloat = 16
    public static let sm: CGFloat = 8
    public static let xl: CGFloat = 32
    public static let xs: CGFloat = 4
}

public enum Radius {
    public static let lg: CGFloat = 12
    public static let md: CGFloat = 8
    public static let sm: CGFloat = 4
}

public enum FontSize {
    public static let body: CGFloat = 13
    public static let caption: CGFloat = 11
    public static let display: CGFloat = 28
    public static let title: CGFloat = 20
}

public enum FontFamily {
    public static let primary = "Inter Variable"
}
