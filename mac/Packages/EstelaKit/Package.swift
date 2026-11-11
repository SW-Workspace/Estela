// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "EstelaKit",
    platforms: [.macOS(.v15)],
    products: [
        .library(name: "EstelaDomain", targets: ["EstelaDomain"]),
        .library(name: "EstelaData", targets: ["EstelaData"]),
        .library(name: "EstelaDesignSystem", targets: ["EstelaDesignSystem"]),
        .library(name: "EstelaPresentation", targets: ["EstelaPresentation"]),
    ],
    targets: [
        .target(name: "EstelaDomain"),
        .target(name: "EstelaData", dependencies: ["EstelaDomain"]),
        .target(name: "EstelaDesignSystem", resources: [.copy("Resources/Fonts")]),
        .target(name: "EstelaPresentation", dependencies: ["EstelaDesignSystem"]),
        .testTarget(name: "EstelaDomainTests", dependencies: ["EstelaDomain"]),
        .testTarget(name: "EstelaDataTests", dependencies: ["EstelaData"]),
        .testTarget(name: "EstelaDesignSystemTests", dependencies: ["EstelaDesignSystem"]),
        .testTarget(name: "EstelaPresentationTests", dependencies: ["EstelaPresentation"]),
    ],
    swiftLanguageModes: [.v6]
)
