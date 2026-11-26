import Foundation
import JavaScriptCore
import DLbronzWeb
#if canImport(UIKit)
import UIKit
typealias PlatformFont = UIFont
#else
import AppKit
typealias PlatformFont = NSFont
#endif

// MARK: - Models

public struct DisplayItem: Decodable {
    public let k: String       // "r" rect, "t" text
    public let x: Double?
    public let y: Double?
    public let w: Double?
    public let h: Double?
    public let s: Double?      // font size
    public let b: Int?
    public let i: Int?
    public let u: Int?
    public let m: Int?
    public let c: String?      // #rrggbbaa
    public let t: String?
    public let href: String?
}

public struct PageLayout: Decodable {
    public let height: Double
    public let bg: String?
    public let title: String
    public let items: [DisplayItem]
}

public struct RenderedPage {
    public let url: URL
    public let html: String
    public let css: String
    public let logs: [String]
}

// MARK: - Text measurement callback (called from Rust during layout)

private func platformFont(size: CGFloat, bold: Bool, mono: Bool) -> PlatformFont {
    let weight: PlatformFont.Weight = bold ? .bold : .regular
    return mono
        ? PlatformFont.monospacedSystemFont(ofSize: size, weight: weight)
        : PlatformFont.systemFont(ofSize: size, weight: weight)
}

private let measureCallback: dlbronz_measure_fn = { text, size, bold, _, mono in
    let str = text.map { String(cString: $0) } ?? ""
    let font = platformFont(size: CGFloat(size), bold: bold != 0, mono: mono != 0)
    return Float((str as NSString).size(withAttributes: [.font: font]).width)
}

// MARK: - Script host

private final class LogBox {
    var lines: [String] = []
    func add(_ s: String) { if lines.count < 200 { lines.append(s) } }
}

/// One JavaScriptCore context holding the JS DOM for a single page.
private final class PageJS {
    let ctx: JSContext
    let log = LogBox()

    init?(domJSON: String, baseURL: String) {
        guard let c = JSContext() else { return nil }
        ctx = c
        let box = log
        c.exceptionHandler = { _, e in
            let line = e?.objectForKeyedSubscript("line")?.toString() ?? "?"
            box.add("[exception] \(e?.toString() ?? "unknown") (line \(line))")
        }
        let logFn: @convention(block) (String, String) -> Void = { level, msg in
            box.add("[\(level)] \(msg)")
        }
        c.setObject(logFn, forKeyedSubscript: "__log" as NSString)
        let fragFn: @convention(block) (String) -> String = { html in
            WebEngine.parseJSON(html, fragment: true) ?? "{\"t\":\"d\",\"c\":[]}"
        }
        c.setObject(fragFn, forKeyedSubscript: "__parseFragment" as NSString)
        c.setObject(domJSON, forKeyedSubscript: "__domJSON" as NSString)
        c.setObject(baseURL, forKeyedSubscript: "__baseURL" as NSString)
        c.evaluateScript(JSDOM.source, withSourceURL: URL(string: "dlbronz://jsdom.js"))
    }

    @discardableResult
    func call(_ name: String, _ args: [Any] = []) -> JSValue? {
        ctx.objectForKeyedSubscript(name as NSString)?.call(withArguments: args)
    }

    func stringList(_ name: String) -> [String] {
        guard let s = call(name)?.toString(), let data = s.data(using: .utf8),
              let arr = try? JSONDecoder().decode([String].self, from: data) else { return [] }
        return arr
    }
}

private struct ScriptRef: Decodable {
    let index: Int
    let src: String?
}

// MARK: - Engine

public enum WebEngine {
    // Rust bridge

    public static func parseJSON(_ html: String, fragment: Bool) -> String? {
        let ptr = html.withCString { fragment ? dlbronz_web_parse_fragment_json($0) : dlbronz_web_parse_json($0) }
        guard let ptr else { return nil }
        defer { dlbronz_web_free(ptr) }
        return String(cString: ptr)
    }

    public static func layout(html: String, css: String, width: Double) -> PageLayout? {
        let ptr = html.withCString { h in
            css.withCString { c in dlbronz_web_render(h, c, Float(width), measureCallback) }
        }
        guard let ptr else { return nil }
        defer { dlbronz_web_free(ptr) }
        let data = Data(bytes: ptr, count: strlen(ptr))
        return try? JSONDecoder().decode(PageLayout.self, from: data)
    }

    public static func layout(page: RenderedPage, width: Double) -> PageLayout? {
        layout(html: page.html, css: page.css, width: width)
    }

    // Script processing (synchronous, no network)

    public struct Processed {
        public let html: String
        public let stylesheets: [URL]
        public let logs: [String]
    }

    /// URLs of external scripts in the page, in execution order.
    public static func scriptURLs(html: String, base: URL) -> [URL] {
        guard let dom = parseJSON(html, fragment: false),
              let js = PageJS(domJSON: dom, baseURL: base.absoluteString) else { return [] }
        return scriptRefs(js).compactMap { ref in
            ref.src.flatMap { URL(string: $0, relativeTo: base)?.absoluteURL }
        }
    }

    private static func scriptRefs(_ js: PageJS) -> [ScriptRef] {
        guard let s = js.call("__scripts")?.toString(), let d = s.data(using: .utf8) else { return [] }
        return (try? JSONDecoder().decode([ScriptRef].self, from: d)) ?? []
    }

    /// Builds the DOM, optionally runs scripts, and returns the resulting HTML.
    /// `fetched` maps absolute script URLs to their source text.
    public static func process(html: String, base: URL, runScripts: Bool, fetched: [String: String] = [:]) -> Processed {
        guard let dom = parseJSON(html, fragment: false),
              let js = PageJS(domJSON: dom, baseURL: base.absoluteString) else {
            return Processed(html: html, stylesheets: [], logs: ["[engine] DOM setup failed"])
        }
        var outHTML = html
        if runScripts {
            for ref in scriptRefs(js) {
                let code: String?
                if let src = ref.src {
                    let abs = URL(string: src, relativeTo: base)?.absoluteString ?? src
                    code = fetched[abs]
                } else {
                    code = js.call("__scriptText", [ref.index])?.toString()
                }
                guard let code, !code.isEmpty else { continue }
                js.call("__setCurrent", [ref.index])
                js.ctx.evaluateScript(code, withSourceURL: URL(string: ref.src.map { _ in "script\(ref.index).js" } ?? "inline\(ref.index).js"))
            }
            js.call("__runTimers", [200])
            js.call("__fire")
            js.call("__runTimers", [200])
            if let s = js.call("__serialize")?.toString(), !s.isEmpty { outHTML = "<!DOCTYPE html>" + s }
        }
        let sheets = js.stringList("__stylesheets").compactMap { URL(string: $0, relativeTo: base)?.absoluteURL }
        return Processed(html: outHTML, stylesheets: sheets, logs: js.log.lines)
    }

    // Network

    static func fetchText(_ url: URL, limit: Int) async -> String? {
        var request = URLRequest(url: url, timeoutInterval: 15)
        request.setValue("Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) DLbronz/0.2", forHTTPHeaderField: "User-Agent")
        guard let (data, response) = try? await URLSession.shared.data(for: request) else { return nil }
        if let http = response as? HTTPURLResponse, !(200..<400).contains(http.statusCode) { return nil }
        if data.count > limit { return nil }
        return String(data: data, encoding: .utf8) ?? String(data: data, encoding: .isoLatin1)
    }

    /// Downloads a page, runs its scripts against the JS DOM, and fetches stylesheets.
    public static func loadPage(_ raw: String, runScripts: Bool) async throws -> RenderedPage {
        let trimmed = raw.trimmingCharacters(in: .whitespacesAndNewlines)
        let normalized = trimmed.hasPrefix("http://") || trimmed.hasPrefix("https://") ? trimmed : "https://" + trimmed
        guard let url = URL(string: normalized), url.host != nil else { throw EngineError.invalidURL }

        var request = URLRequest(url: url, timeoutInterval: 20)
        request.setValue("Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) DLbronz/0.2", forHTTPHeaderField: "User-Agent")
        let (data, response) = try await URLSession.shared.data(for: request)
        if let http = response as? HTTPURLResponse, !(200..<400).contains(http.statusCode) {
            throw EngineError.http(http.statusCode)
        }
        let finalURL = response.url ?? url
        let html = String(data: data, encoding: .utf8) ?? String(decoding: data, as: UTF8.self)

        var fetched: [String: String] = [:]
        if runScripts {
            let urls = scriptURLs(html: html, base: finalURL)
            fetched = await fetchAll(urls, limit: 2_000_000)
        }
        let processed = process(html: html, base: finalURL, runScripts: runScripts, fetched: fetched)
        let sheets = await fetchAll(processed.stylesheets, limit: 1_000_000)
        let css = processed.stylesheets.compactMap { sheets[$0.absoluteString] }.joined(separator: "\n")
        return RenderedPage(url: finalURL, html: processed.html, css: css, logs: processed.logs)
    }

    private static func fetchAll(_ urls: [URL], limit: Int) async -> [String: String] {
        await withTaskGroup(of: (String, String?).self) { group in
            for u in Set(urls.prefix(40)) {
                group.addTask { (u.absoluteString, await fetchText(u, limit: limit)) }
            }
            var out: [String: String] = [:]
            for await (k, v) in group { if let v { out[k] = v } }
            return out
        }
    }
}
