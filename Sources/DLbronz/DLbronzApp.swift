import SwiftUI
import DLbronzEngine

@main
struct DLbronzApp: App {
    init() {
        // Headless smoke test for CI: `DLbronz --selftest`
        if CommandLine.arguments.contains("--selftest") {
            exit(Engine.selfTest() ? 0 : 1)
        }
    }

    var body: some Scene {
        WindowGroup("DLbronz") {
            ContentView()
                .frame(minWidth: 640, minHeight: 440)
        }
    }
}
