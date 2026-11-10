import EstelaDesignSystem
import EstelaPresentation
import OSLog
import SwiftUI

@main
struct EstelaApp: App {
    private static let logger = Logger(subsystem: "com.estela.app", category: "launch")

    init() {
        do {
            try FontRegistry.registerBundledFonts()
        } catch {
            Self.logger.error("Bundled font registration failed: \(String(describing: error))")
        }
    }

    var body: some Scene {
        Window("main.title", id: "main") {
            MainWindowView()
                .preferredColorScheme(.dark)
        }
        .windowStyle(.hiddenTitleBar)
    }
}
