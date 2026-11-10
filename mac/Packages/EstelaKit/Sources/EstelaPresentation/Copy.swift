import SwiftUI

public enum Copy: String, CaseIterable, Sendable {
    case mainTitle = "main.title"
    case mainTagline = "main.tagline"

    var key: LocalizedStringKey { LocalizedStringKey(rawValue) }
}
