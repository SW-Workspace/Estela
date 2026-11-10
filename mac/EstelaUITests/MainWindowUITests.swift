import XCTest

final class MainWindowUITests: XCTestCase {
    private static let launchTimeout: TimeInterval = 10

    override func setUp() {
        continueAfterFailure = false
    }

    @MainActor
    func testShowsEnglishTitleAndTagline() {
        let app = launch(language: "en")
        XCTAssertEqual(text(app, "main.title"), "Estela")
        XCTAssertEqual(text(app, "main.tagline"), "Record. Estela does the rest.")
    }

    @MainActor
    func testShowsSpanishTagline() {
        let app = launch(language: "es")
        XCTAssertEqual(text(app, "main.tagline"), "Graba. Estela hace el resto.")
    }

    @MainActor
    private func text(_ app: XCUIApplication, _ identifier: String) -> String? {
        app.staticTexts[identifier].value as? String
    }

    @MainActor
    private func launch(language: String) -> XCUIApplication {
        let app = XCUIApplication()
        app.launchArguments = [
            "-AppleLanguages", "(\(language))",
            "-AppleLocale", language,
            "-ApplePersistenceIgnoreState", "YES",
        ]
        app.launch()
        XCTAssertTrue(app.staticTexts["main.title"].waitForExistence(timeout: Self.launchTimeout), app.debugDescription)
        attachScreenshot(of: app, named: "main-window-\(language)")
        return app
    }

    @MainActor
    private func attachScreenshot(of app: XCUIApplication, named name: String) {
        let attachment = XCTAttachment(screenshot: app.windows.firstMatch.screenshot())
        attachment.name = name
        attachment.lifetime = .keepAlways
        add(attachment)
    }
}
