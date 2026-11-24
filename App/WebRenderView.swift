import SwiftUI
import WebKit

/// Optional "Firefox" renderer.
///
/// This is NOT Gecko: Gecko cannot be embedded in an iOS app, and Firefox for iOS
/// itself renders with WebKit. This view does the same thing: WKWebView plus the
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

    static func makeWebView() -> WKWebView {
        let view = WKWebView(frame: .zero, configuration: WKWebViewConfiguration())
        view.customUserAgent = userAgent
        return view
    }

    static func load(_ url: URL?, in view: WKWebView) {
        guard let url, view.url != url else { return }
        view.load(URLRequest(url: url))
    }
}

#if os(iOS)
struct WebRenderView: UIViewRepresentable {
    let url: URL?
    func makeUIView(context: Context) -> WKWebView { FirefoxRenderer.makeWebView() }
    func updateUIView(_ view: WKWebView, context: Context) { FirefoxRenderer.load(url, in: view) }
}
#else
struct WebRenderView: NSViewRepresentable {
    let url: URL?
    func makeNSView(context: Context) -> WKWebView { FirefoxRenderer.makeWebView() }
    func updateNSView(_ view: WKWebView, context: Context) { FirefoxRenderer.load(url, in: view) }
}
#endif
