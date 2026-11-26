import SwiftUI
import DLbronzEngine

extension Color {
    /// "#rrggbbaa" (or "#rrggbb")
    init(hex: String) {
        var s = hex.trimmingCharacters(in: CharacterSet(charactersIn: "#"))
        if s.count == 6 { s += "ff" }
        let v = UInt64(s, radix: 16) ?? 0x000000ff
        self.init(
            .sRGB,
            red: Double((v >> 24) & 0xff) / 255,
            green: Double((v >> 16) & 0xff) / 255,
            blue: Double((v >> 8) & 0xff) / 255,
            opacity: Double(v & 0xff) / 255
        )
    }
}

/// Paints the display list produced by the DLbronz Rust core.
struct RenderView: View {
    let page: RenderedPage
    let onLink: (URL) -> Void

    @State private var layout: PageLayout?

    var body: some View {
        GeometryReader { geo in
            ScrollView {
                if let layout {
                    canvas(layout, width: geo.size.width)
                } else {
                    ProgressView().frame(maxWidth: .infinity).padding(.top, 40)
                }
            }
            .background(Color.white)
            .task(id: "\(page.url.absoluteString)|\(page.html.count)|\(page.css.count)|\(Int(geo.size.width))") {
                let page = page
                let width = geo.size.width
                layout = await Task.detached { WebEngine.layout(page: page, width: width) }.value
            }
        }
    }

    private func canvas(_ layout: PageLayout, width: CGFloat) -> some View {
        Canvas { ctx, size in
            if let bg = layout.bg {
                ctx.fill(Path(CGRect(origin: .zero, size: size)), with: .color(Color(hex: bg)))
            }
            for item in layout.items {
                let x = item.x ?? 0, y = item.y ?? 0
                if item.k == "r" {
                    let rect = CGRect(x: x, y: y, width: item.w ?? 0, height: item.h ?? 0)
                    ctx.fill(Path(rect), with: .color(Color(hex: item.c ?? "#00000000")))
                } else if let t = item.t {
                    var text = Text(t)
                        .font(.system(size: item.s ?? 16,
                                      weight: item.b == 1 ? .bold : .regular,
                                      design: item.m == 1 ? .monospaced : .default))
                        .foregroundColor(Color(hex: item.c ?? "#000000ff"))
                    if item.i == 1 { text = text.italic() }
                    if item.u == 1 { text = text.underline() }
                    ctx.draw(text, at: CGPoint(x: x, y: y), anchor: .topLeading)
                }
            }
        }
        .frame(width: width, height: max(layout.height, 1))
        .contentShape(Rectangle())
        .onTapGesture { point in
            if let url = link(at: point, in: layout) { onLink(url) }
        }
    }

    private func link(at p: CGPoint, in layout: PageLayout) -> URL? {
        for item in layout.items.reversed() where item.k == "t" {
            guard let href = item.href else { continue }
            let rect = CGRect(x: item.x ?? 0, y: item.y ?? 0, width: item.w ?? 0, height: (item.s ?? 16) * 1.2)
            guard rect.insetBy(dx: -2, dy: -4).contains(p) else { continue }
            if href.hasPrefix("#") || href.lowercased().hasPrefix("javascript:") { return nil }
            return URL(string: href, relativeTo: page.url)?.absoluteURL
        }
        return nil
    }
}
