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
        .target(
            name: "DLbronzEngine",
            dependencies: ["CRuntime", "CppRenderer"]
        ),
        .testTarget(
            name: "DLbronzEngineTests",
            dependencies: ["DLbronzEngine"]
        )
    ],
    cxxLanguageStandard: .cxx17
)
