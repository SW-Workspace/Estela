import EstelaDesignSystem
import SwiftUI

public struct MainWindowView: View {
    public static let minimumWidth: CGFloat = 640
    public static let minimumHeight: CGFloat = 400

    public init() {}

    public var body: some View {
        VStack(spacing: Spacing.sm) {
            Text(Copy.mainTitle.key)
                .font(.estela(size: FontSize.display, weight: .bold))
                .foregroundStyle(Palette.textPrimary)
                .accessibilityIdentifier(Copy.mainTitle.rawValue)
            Text(Copy.mainTagline.key)
                .font(.estela(size: FontSize.body))
                .foregroundStyle(Palette.textSecondary)
                .accessibilityIdentifier(Copy.mainTagline.rawValue)
        }
        .padding(Spacing.xl)
        .frame(minWidth: Self.minimumWidth, maxWidth: .infinity, minHeight: Self.minimumHeight, maxHeight: .infinity)
        .background(Palette.background)
    }
}
