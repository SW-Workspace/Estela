@testable import EstelaPresentation
import Foundation
import Testing

private struct StringCatalog: Decodable {
    let sourceLanguage: String
    let strings: [String: StringCatalogEntry]
}

private struct StringCatalogEntry: Decodable {
    let localizations: [String: StringCatalogLocalization]?
}

private struct StringCatalogLocalization: Decodable {
    let stringUnit: StringCatalogUnit
}

private struct StringCatalogUnit: Decodable {
    let state: String
    let value: String
}

struct CopyCatalogTests {
    private static let requiredLanguages = ["en", "es"]

    private static let catalogURL = URL(filePath: #filePath)
        .deletingLastPathComponent()
        .appending(path: "../../../../Estela/Localizable.xcstrings")
        .standardized

    private static let catalog = try? JSONDecoder().decode(StringCatalog.self, from: Data(contentsOf: catalogURL))

    private func loadCatalog() throws -> StringCatalog {
        try #require(Self.catalog)
    }

    @Test func sourceLanguageIsEnglish() throws {
        #expect(try loadCatalog().sourceLanguage == "en")
    }

    @Test(arguments: Copy.allCases)
    func everyKeyIsTranslated(copy: Copy) throws {
        let entry = try #require(try loadCatalog().strings[copy.rawValue])
        for language in Self.requiredLanguages {
            let unit = try #require(entry.localizations?[language]?.stringUnit)
            #expect(unit.state == "translated")
            #expect(!unit.value.isEmpty)
        }
    }
}
