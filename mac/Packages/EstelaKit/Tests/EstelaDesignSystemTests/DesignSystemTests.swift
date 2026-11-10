import AppKit
@testable import EstelaDesignSystem
import SwiftUI
import Testing

struct DesignSystemTests {
    @Test func resolvesInterFromTheBundledFiles() throws {
        try FontRegistry.registerBundledFonts()
        let font = try #require(NSFont(name: "InterVariable", size: FontSize.body))
        let fontURL = try #require(CTFontCopyAttribute(font, kCTFontURLAttribute) as? URL)
        #expect(font.familyName == FontFamily.primary)
        #expect(fontURL.standardizedFileURL.path().hasPrefix(Bundle.module.bundleURL.standardizedFileURL.path()))
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
