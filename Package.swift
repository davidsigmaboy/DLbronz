// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "DLbronz",
    platforms: [.macOS(.v13)],
    products: [.executable(name: "DLbronz", targets: ["DLbronz"])],
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
        .executableTarget(
            name: "DLbronz",
            dependencies: ["DLbronzEngine"]
        )
    ],
    cxxLanguageStandard: .cxx17
)
