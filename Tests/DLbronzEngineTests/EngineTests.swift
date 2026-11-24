import XCTest
@testable import DLbronzEngine

final class EngineTests: XCTestCase {
    func testVersion() {
        XCTAssertEqual(Engine.version, "DLbronz 0.1-multilang")
    }

    func testCppRenderer() {
        XCTAssertEqual(Engine.renderHTML("<p>Hi <b>there</b></p>"), "Hi there")
    }

    func testJavaScript() {
        XCTAssertEqual(Engine.runJavaScript("6 * 7"), "42")
        XCTAssertEqual(Engine.runJavaScript("console.log('a', 1)"), "a 1")
    }

    #if os(macOS)
    func testPython() {
        XCTAssertEqual(Engine.runPython("print('Python runtime works')"), "Python runtime works")
    }
    #endif
}
