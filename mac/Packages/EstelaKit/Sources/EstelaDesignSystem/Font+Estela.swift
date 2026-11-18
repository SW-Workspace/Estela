import SwiftUI

extension Font {
    public static func estela(size: CGFloat, weight: Font.Weight = .regular) -> Font {
        .custom(FontFamily.primary, fixedSize: size).weight(weight)
    }
}
