import SwiftUI
import WebKit

/// Optional "Firefox" renderer.
///
/// This is NOT Gecko: Gecko cannot be embedded in an iOS app, and Firefox for iOS
/// itself renders with WebKit. This does the same thing: WKWebView plus the
/// Firefox for iOS user agent. The original DLbronz engine is untouched.
enum FirefoxRenderer {
    static let userAgent =
        "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 "
        + "(KHTML, like Gecko) FxiOS/133.0 Mobile/15E148 Safari/605.1.15"

    static func url(from raw: String) -> URL? {
        let trimmed = raw.trimmingCharacters(in: .whitespacesAndNewlines)
        let normalized = trimmed.hasPrefix("http://") || trimmed.hasPrefix("https://")
            ? trimmed : "https://" + trimmed
        return URL(string: normalized)
    }
}

/// Owns the page web view (JS runs in the page) and a hidden Pyodide web view (Python).
@MainActor
final class WebController: ObservableObject {
    let webView: WKWebView = {
        let view = WKWebView(frame: .zero, configuration: WKWebViewConfiguration())
        view.customUserAgent = FirefoxRenderer.userAgent
        return view
    }()

    // Python via Pyodide (CPython compiled to WebAssembly). Needs network on first use.
    private static let pyodideHTML = """
    <!doctype html><html><head><meta charset="utf-8">
    <script src="https://cdn.jsdelivr.net/pyodide/v0.26.4/full/pyodide.js"></script>
    <script>
    window.pyReady = loadPyodide();
    window.runPy = async function (code) {
      const out = [];
      try {
        const py = await window.pyReady;
        py.setStdout({ batched: s => out.push(s) });
        py.setStderr({ batched: s => out.push(s) });
        const r = await py.runPythonAsync(code);
        if (r !== undefined && r !== null) out.push(String(r));
      } catch (e) { out.push(String(e)); }
      return out.join("\\n");
    };
    </script></head><body></body></html>
    """

    private lazy var pyView: WKWebView = {
        let view = WKWebView(frame: .zero, configuration: WKWebViewConfiguration())
        view.loadHTMLString(Self.pyodideHTML, baseURL: URL(string: "https://cdn.jsdelivr.net"))
        return view
    }()

    private var currentURL: URL?

    func load(_ url: URL?) {
        guard let url, url != currentURL else { return }
        currentURL = url
        webView.load(URLRequest(url: url))
    }

    func runJS(_ code: String) async -> String {
        do {
            let result = try await webView.evaluateJavaScript(code)
            return result.map { "\($0)" } ?? "undefined"
        } catch {
            return "JavaScript error: \(error.localizedDescription)"
        }
    }

    func runPython(_ code: String) async -> String {
        do {
            let result = try await pyView.callAsyncJavaScript(
                "return await window.runPy(code);",
                arguments: ["code": code],
                in: nil,
                contentWorld: .page
            )
            return (result as? String) ?? ""
        } catch {
            return "Python error: \(error.localizedDescription)"
        }
    }
}

#if os(iOS)
struct WebRenderView: UIViewRepresentable {
    @ObservedObject var controller: WebController
    let url: URL?
    func makeUIView(context: Context) -> WKWebView { controller.webView }
    func updateUIView(_ view: WKWebView, context: Context) { controller.load(url) }
}
#else
struct WebRenderView: NSViewRepresentable {
    @ObservedObject var controller: WebController
    let url: URL?
    func makeNSView(context: Context) -> WKWebView { controller.webView }
    func updateNSView(_ view: WKWebView, context: Context) { controller.load(url) }
}
#endif
