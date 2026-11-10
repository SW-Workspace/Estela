#!/usr/bin/env swift

import Foundation

enum GenerationError: Error, CustomStringConvertible {
    case missingGroup(String)
    case invalidColor(String)
    case offGrid(String, Int)

    var description: String {
        switch self {
        case .missingGroup(let group): "Missing token group: \(group)"
        case .invalidColor(let name): "Invalid hex color for token: \(name)"
        case .offGrid(let name, let value): "Token \(name) = \(value) is not on the 4 px grid"
        }
    }
}

let gridUnit = 4
let scriptDirectory = URL(filePath: CommandLine.arguments[0]).deletingLastPathComponent()
let repositoryRoot = scriptDirectory.appending(path: "../..").standardized
let tokensFile = repositoryRoot.appending(path: "assets/design-tokens.json")
let outputFile = repositoryRoot.appending(
    path: "mac/Packages/EstelaKit/Sources/EstelaDesignSystem/Generated/Tokens.swift"
)

func group<Value>(_ name: String, in tokens: [String: Any]) throws -> [(String, Value)] {
    guard let values = tokens[name] as? [String: Value] else { throw GenerationError.missingGroup(name) }
    return values.sorted { $0.key < $1.key }
}

func colorLine(name: String, hex: String) throws -> String {
    let digits = hex.hasPrefix("#") ? String(hex.dropFirst()) : hex
    guard digits.count == 6, UInt32(digits, radix: 16) != nil else { throw GenerationError.invalidColor(name) }
    return "    public static let \(name) = Color(hex: 0x\(digits.uppercased()))"
}

func gridLine(name: String, value: Int) throws -> String {
    guard value.isMultiple(of: gridUnit) else { throw GenerationError.offGrid(name, value) }
    return "    public static let \(name): CGFloat = \(value)"
}

func enumBlock(_ name: String, lines: [String]) -> String {
    (["public enum \(name) {"] + lines + ["}"]).joined(separator: "\n")
}

func generate() throws -> String {
    let data = try Data(contentsOf: tokensFile)
    guard let tokens = try JSONSerialization.jsonObject(with: data) as? [String: Any] else {
        throw GenerationError.missingGroup("root")
    }
    let colors: [(String, String)] = try group("color", in: tokens)
    let spacing: [(String, Int)] = try group("spacing", in: tokens)
    let radii: [(String, Int)] = try group("radius", in: tokens)
    let fontSizes: [(String, Int)] = try group("fontSize", in: tokens)
    let families: [(String, String)] = try group("fontFamily", in: tokens)
    let blocks = [
        enumBlock("Palette", lines: try colors.map(colorLine)),
        enumBlock("Spacing", lines: try spacing.map(gridLine)),
        enumBlock("Radius", lines: try radii.map(gridLine)),
        enumBlock("FontSize", lines: fontSizes.map { "    public static let \($0.0): CGFloat = \($0.1)" }),
        enumBlock("FontFamily", lines: families.map { "    public static let \($0.0) = \"\($0.1)\"" }),
    ]
    return (["import SwiftUI"] + blocks).joined(separator: "\n\n") + "\n"
}

do {
    try generate().write(to: outputFile, atomically: true, encoding: .utf8)
    print("Wrote \(outputFile.path())")
} catch {
    FileHandle.standardError.write(Data("\(error)\n".utf8))
    exit(1)
}
