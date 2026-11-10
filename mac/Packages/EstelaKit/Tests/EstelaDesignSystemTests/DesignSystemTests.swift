import AppKit
import SwiftUI
import Testing
@testable import EstelaDesignSystem

struct DesignSystemTests {
    @Test func registersInterSoItResolvesByFamilyName() throws {
        try FontRegistry.registerBundledFonts()
        let members = NSFontManager.shared.availableMembers(ofFontFamily: FontFamily.primary) ?? []
        #expect(!members.isEmpty)
    }

    @Test func registrationIsIdempotent() throws {
        try FontRegistry.registerBundledFonts()
        try FontRegistry.registerBundledFonts()
    }

    @Test func decodesHexIntoSRGBComponents() throws {
        let color = try #require(NSColor(Color(hex: 0xFF8000)).usingColorSpace(.sRGB))
        #expect(color.redComponent == 1)
        #expect(abs(color.greenComponent - 128.0 / 255.0) < 0.001)
        #expect(color.blueComponent == 0)
    }

    @Test(arguments: [Spacing.xs, Spacing.sm, Spacing.md, Spacing.lg, Spacing.xl, Radius.sm, Radius.md, Radius.lg])
    func gridTokensAreMultiplesOfFour(value: CGFloat) {
        #expect(value.truncatingRemainder(dividingBy: 4) == 0)
    }
}
