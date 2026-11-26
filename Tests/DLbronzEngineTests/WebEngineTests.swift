import XCTest
@testable import DLbronzEngine

final class WebEngineTests: XCTestCase {
    private let base = URL(string: "https://example.com/")!

    private func texts(_ l: PageLayout) -> [String] {
        l.items.filter { $0.k == "t" }.compactMap { $0.t }
    }

    func testLayoutProducesText() throws {
        let l = try XCTUnwrap(WebEngine.layout(html: "<html><body><h1>Title</h1><p>Hello <b>world</b></p></body></html>", css: "", width: 320))
        XCTAssertTrue(texts(l).contains("Title"))
        XCTAssertTrue(texts(l).contains("Hello"))
        XCTAssertTrue(texts(l).contains("world"))
        XCTAssertGreaterThan(l.height, 30)
    }

    func testCSSApplies() throws {
        let l = try XCTUnwrap(WebEngine.layout(html: "<html><body><p class='x'>hi</p></body></html>", css: ".x{color:#ff0000;font-size:30px}", width: 320))
        let item = try XCTUnwrap(l.items.first { $0.t == "hi" })
        XCTAssertEqual(item.c, "#ff0000ff")
        XCTAssertEqual(item.s, 30)
    }

    func testHtml5ParserRepairsMarkup() throws {
        // unclosed tags and missing html/body: handled by html5ever
        let l = try XCTUnwrap(WebEngine.layout(html: "<p>one<p>two<b>bold", css: "", width: 320))
        XCTAssertEqual(texts(l), ["one", "two", "bold"])
    }

    func testScriptsMutateDOM() {
        let html = """
        <html><head><title>T</title></head><body><div id="a">old</div>
        <script>document.getElementById('a').textContent = 'new'; console.log('ran');</script>
        </body></html>
        """
        let p = WebEngine.process(html: html, base: base, runScripts: true)
        XCTAssertTrue(p.html.contains(">new</div>"), p.html)
        XCTAssertTrue(p.logs.contains("[log] ran"), "\(p.logs)")
    }

    func testInnerHTMLUsesRustParser() {
        let html = "<html><body><div id='a'></div><script>document.getElementById('a').innerHTML='<p>x<p>y';</script></body></html>"
        let p = WebEngine.process(html: html, base: base, runScripts: true)
        XCTAssertTrue(p.html.contains("<p>x</p><p>y</p>"), p.html)
    }

    func testScriptsCanBeDisabled() {
        let html = "<html><body><div id='a'>old</div><script>document.getElementById('a').textContent='new'</script></body></html>"
        let p = WebEngine.process(html: html, base: base, runScripts: false)
        XCTAssertTrue(p.html.contains("old"))
    }

    func testStylesheetsAreListed() {
        let html = "<html><head><link rel='stylesheet' href='/s.css'></head><body></body></html>"
        let p = WebEngine.process(html: html, base: base, runScripts: false)
        XCTAssertEqual(p.stylesheets.map(\.absoluteString), ["https://example.com/s.css"])
    }

    func testExternalScriptsAreResolved() {
        let html = "<html><body><script src='/a.js'></script></body></html>"
        XCTAssertEqual(WebEngine.scriptURLs(html: html, base: base).map(\.absoluteString), ["https://example.com/a.js"])
    }
}
