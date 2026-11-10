# AGENTS.md (macOS)

Rules for any agent or contributor working on the Estela macOS app.
The Mac app is **Swift only**, lives entirely in `mac/` and has no dependency
on the Rust workspace in `wl/`. Never edit `wl/` during a Mac task.

## 1. Scope

| Path | Status for Mac tasks |
| --- | --- |
| `mac/` | Main work area. |
| `assets/` | Shared design tokens and fonts. Editable when the task needs it. |
| `.github/workflows/mac-*.yml` | Mac CI. Editable. |
| `wl/`, other workflows | **Off limits.** |

## 2. Platform

- Minimum deployment target: macOS 15.
- Swift 6 language mode with complete strict concurrency.
- Apple frameworks only. A third-party dependency needs explicit approval.
- Distribution: Developer ID and notarization, no App Sandbox.

## 3. Architecture

Clean architecture. The layers are modules of `Packages/EstelaKit`, so the
compiler enforces the dependency rule.

| Module | Contains | May import |
| --- | --- | --- |
| `EstelaDomain` | Entities, value types, use cases, repository protocols | `Foundation` |
| `EstelaData` | Repository implementations, system adapters (ScreenCaptureKit, AVFoundation, filesystem) | `EstelaDomain` |
| `EstelaDesignSystem` | Generated tokens, fonts, reusable visual components | SwiftUI |
| `EstelaPresentation` | SwiftUI views and `@Observable` view models | `EstelaDomain`, `EstelaDesignSystem` |
| `Estela` (app target) | Composition root: wires Data into Presentation | Everything |

- Dependencies point inward. Presentation never imports Data.
- Use cases depend on protocols declared in Domain; Data implements them.
- Time is `Time` (nanoseconds since recording start). Coordinates are physical source pixels.
- The `.estela` folder is the source of truth. The Library scans folders; there is no database.
- Edits are never destructive: original media is never rewritten.

## 4. Code style

- Follow the [Swift API Design Guidelines](https://www.swift.org/documentation/api-design-guidelines/).
- `swiftlint --strict` must pass with `mac/.swiftlint.yml`.
- 4 spaces, max line width 120, trailing commas in multi-line collections.
- Names say what things are; no abbreviations except `id`, `url`, `gpu`, `fps`.
- Small types and functions; one concept per file.
- Enums over booleans in public APIs. No magic numbers.
- No force unwraps, `try!` or `fatalError` outside tests.
- Errors: one `Error` enum per module.

## 5. Comments

No comments unless strictly necessary. Code must explain itself. When a comment
is unavoidable, it is a single line explaining **why**, never what.

## 6. Interface

- User-facing text lives only in `Estela/Localizable.xcstrings`: English base, Spanish translation.
- Colors, spacing, radii and type come from `EstelaDesignSystem` tokens, never literals.
- Tokens are generated from `assets/design-tokens.json` with `Scripts/generate-tokens.swift`; commit the output.
- Dark theme first. Inter is bundled. 4 px grid.

## 7. Tests

- Swift Testing (`@Test`) for unit and integration tests, one test target per module.
- XCUITest for interface flows in `EstelaUITests/`.
- Every phase adds tests for what it builds.

## 8. Phases

Work is delivered in phases. A phase is done only when all of this passes:

1. `swift test` in `Packages/EstelaKit`.
2. `swiftlint --strict` in `mac/`.
3. `xcodebuild ... build test` for the `Estela` scheme.
4. `/code-review high` on the phase branch, with confirmed findings fixed.
5. Automatic QA of the running app; iterate until it works.

## 9. Git

- Branch from `develop`: `features/mac-...`, `fixed/mac-...`, `refactor/mac-...`, `docs/mac-...`.
- Pull requests target `develop`, never `main`.
- Commit format: `<type>(mac): <summary>` with type `feat`, `fixed`, `refactor` or `docs`.
- Imperative, lowercase, no final period, at most 72 characters.
- No `Co-Authored-By` lines.
