// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "DLbronz",
    platforms: [.macOS(.v13), .iOS(.v16)],
    products: [.library(name: "DLbronzEngine", targets: ["DLbronzEngine"])],
    targets: [
        .target(
            name: "CRuntime",
            publicHeadersPath: "include"
        ),
        .target(
            name: "CppRenderer",
            publicHeadersPath: "include"
        ),
        // Rust core (html5ever parser, CSS cascade, layout). Built by
        // scripts/build-rust-xcframework.sh before Swift builds.
        .binaryTarget(
            name: "DLbronzWeb",
            path: "Frameworks/DLbronzWeb.xcframework"
        ),
        .target(
            name: "DLbronzEngine",
            dependencies: ["CRuntime", "CppRenderer", "DLbronzWeb"]
        ),
        .testTarget(
            name: "DLbronzEngineTests",
            dependencies: ["DLbronzEngine"]
        )
    ],
    cxxLanguageStandard: .cxx17
)
