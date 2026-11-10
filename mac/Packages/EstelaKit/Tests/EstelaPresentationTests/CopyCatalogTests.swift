import Foundation
import Testing
@testable import EstelaPresentation

struct CopyCatalogTests {
    private static let requiredLanguages = ["en", "es"]

    private static let catalogURL = URL(filePath: #filePath)
        .deletingLastPathComponent()
        .appending(path: "../../../../Estela/Localizable.xcstrings")
        .standardized

    private struct Catalog: Decodable {
        struct Entry: Decodable {
            let localizations: [String: Localization]?
        }

        struct Localization: Decodable {
            let stringUnit: StringUnit
        }

        struct StringUnit: Decodable {
            let state: String
            let value: String
        }

        let sourceLanguage: String
        let strings: [String: Entry]
    }

    private func loadCatalog() throws -> Catalog {
        try JSONDecoder().decode(Catalog.self, from: Data(contentsOf: Self.catalogURL))
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
