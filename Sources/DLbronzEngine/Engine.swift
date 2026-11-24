import Foundation
import JavaScriptCore
import CRuntime
import CppRenderer

public enum EngineError: LocalizedError {
    case invalidURL
    case http(Int)
    case renderer

    public var errorDescription: String? {
        switch self {
        case .invalidURL: return "Invalid URL"
        case .http(let code): return "HTTP status: \(code)"
        case .renderer: return "Renderer allocation failed"
        }
    }
}

public enum Engine {
    public static var version: String {
        String(cString: dlbronz_engine_version())
    }

    public static let banner = "Independent experimental engine. No Chromium/WebKit/Gecko."

    // MARK: C++ renderer

    public static func renderHTML(_ html: String) -> String? {
        guard let ptr = html.withCString({ dlbronz_render_html_text($0) }) else { return nil }
        defer { free(ptr) }
        return String(cString: ptr)
    }

    // MARK: Page loading

    public static func openPage(_ raw: String) async throws -> String {
        let trimmed = raw.trimmingCharacters(in: .whitespacesAndNewlines)
        let normalized = trimmed.hasPrefix("http://") || trimmed.hasPrefix("https://")
            ? trimmed : "https://" + trimmed
        guard dlbronz_is_http_url(normalized) == 1, let url = URL(string: normalized) else {
            throw EngineError.invalidURL
        }

        var request = URLRequest(url: url, timeoutInterval: 20)
        request.setValue("DLbronz/0.1", forHTTPHeaderField: "User-Agent")
        let (data, response) = try await URLSession.shared.data(for: request)
        if let http = response as? HTTPURLResponse, !(200..<400).contains(http.statusCode) {
            throw EngineError.http(http.statusCode)
        }
        let html = String(decoding: data, as: UTF8.self)
        guard let visible = renderHTML(html) else { throw EngineError.renderer }
        return "--- \(url.absoluteString) ---\n\n" + String(visible.prefix(12000))
            + "\n\n[Prototype renderer: text extraction only; no CSS layout or image painting yet]"
    }

    // MARK: JavaScript (JavaScriptCore)

    public static func runJavaScript(_ source: String) -> String {
        guard let context = JSContext() else { return "JavaScript context unavailable" }
        var lines: [String] = []

        context.exceptionHandler = { _, error in
            if let error { lines.append("JavaScript error: \(error)") }
        }

        let console = JSValue(newObjectIn: context)
        let log: @convention(block) () -> Void = {
            let args = (JSContext.currentArguments() as? [JSValue]) ?? []
            lines.append(args.map { $0.toString() ?? "" }.joined(separator: " "))
        }
        console?.setObject(log, forKeyedSubscript: "log" as NSString)
        context.setObject(console, forKeyedSubscript: "console" as NSString)

        let result = context.evaluateScript(source)
        if let result, !result.isUndefined { lines.append(result.toString() ?? "") }
        return lines.joined(separator: "\n")
    }

    // MARK: Python (system python3, macOS only)

    public static func runPython(_ source: String) -> String {
        // Invokes the system Python 3 interpreter. Not available in an iOS sandbox.
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
        process.arguments = ["python3", "-c", source]
        let output = Pipe()
        process.standardOutput = output
        process.standardError = output
        do {
            try process.run()
            let data = output.fileHandleForReading.readDataToEndOfFile()
            process.waitUntilExit()
            var text = String(decoding: data, as: UTF8.self)
            if process.terminationStatus != 0 {
                text += "\nPython exited with status \(process.terminationStatus)"
            }
            return text.trimmingCharacters(in: .whitespacesAndNewlines)
        } catch {
            return "Python 3 unavailable: \(error.localizedDescription)"
        }
    }

    // MARK: Self test (used by CI)

    public static func selfTest() -> Bool {
        var ok = true
        func check(_ name: String, _ actual: String, _ expected: String) {
            let pass = actual == expected
            print("[\(pass ? "PASS" : "FAIL")] \(name): \(actual)")
            if !pass { ok = false }
        }
        check("version", version, "DLbronz 0.1-multilang")
        check("c++ renderer", renderHTML("<p>Hi <b>there</b></p>") ?? "nil", "Hi there")
        check("javascript", runJavaScript("6 * 7"), "42")
        check("python", runPython("print('Python runtime works')"), "Python runtime works")
        return ok
    }
}
