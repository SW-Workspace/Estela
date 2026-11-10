// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "EstelaKit",
    platforms: [.macOS(.v15)],
    products: [
        .library(name: "EstelaDomain", targets: ["EstelaDomain"]),
        .library(name: "EstelaData", targets: ["EstelaData"]),
    ],
    targets: [
        .target(name: "EstelaDomain"),
        .target(name: "EstelaData", dependencies: ["EstelaDomain"]),
        .testTarget(name: "EstelaDomainTests", dependencies: ["EstelaDomain"]),
        .testTarget(name: "EstelaDataTests", dependencies: ["EstelaData"]),
    ],
    swiftLanguageModes: [.v6]
)
