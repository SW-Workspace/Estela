# Estela for macOS

Native Swift app (SwiftUI + AppKit), independent of the Rust workspace in `../wl/`.
Rules for contributors and agents: [AGENTS.md](AGENTS.md).

## Requirements

- macOS 15 or later.
- Xcode 16 or later (Swift 6).
- SwiftLint 0.65.1 (the version CI pins).

## Layout

| Path | Contents |
| --- | --- |
| `Estela/` | App target: composition root and `Localizable.xcstrings` (en, es). |
| `EstelaUITests/` | XCUITest interface tests. |
| `Packages/EstelaKit/` | Layers: `EstelaDomain`, `EstelaData`, `EstelaDesignSystem`, `EstelaPresentation`. |
| `Scripts/generate-tokens.swift` | Generates design tokens from `../assets/design-tokens.json`. |

## Commands

```sh
cd mac
swiftlint --strict
swift test --package-path Packages/EstelaKit
xcodebuild -project Estela.xcodeproj -scheme Estela -destination 'platform=macOS' build test
swift Scripts/generate-tokens.swift
```
