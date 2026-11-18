import CoreText
import Foundation

public enum DesignSystemError: Error, Equatable {
    case missingFontResources
    case fontRegistrationFailed(String)
}

public enum FontRegistry {
    public static func registerBundledFonts() throws {
        let fontURLs = Bundle.module.urls(forResourcesWithExtension: "ttf", subdirectory: "Fonts") ?? []
        guard !fontURLs.isEmpty else { throw DesignSystemError.missingFontResources }
        try fontURLs.forEach(register)
    }

    private static func register(_ url: URL) throws {
        var error: Unmanaged<CFError>?
        guard !CTFontManagerRegisterFontsForURL(url as CFURL, .process, &error) else { return }
        guard let failure = error?.takeRetainedValue() else { return }
        guard CFErrorGetCode(failure) != CTFontManagerError.alreadyRegistered.rawValue else { return }
        throw DesignSystemError.fontRegistrationFailed(url.lastPathComponent)
    }
}
