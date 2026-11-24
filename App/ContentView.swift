import SwiftUI
import DLbronzEngine

enum Mode: String, CaseIterable, Identifiable {
    case page = "Page"
    case firefox = "Firefox"
    case javascript = "JavaScript"
    case python = "Python"
    var id: String { rawValue }

    /// The original Python mode needs a system interpreter, which iOS does not have.
    /// (Firefox mode has its own Python via Pyodide that works everywhere.)
    static var available: [Mode] {
        #if os(macOS)
        return [.page, .firefox, .javascript, .python]
        #else
        return [.page, .firefox, .javascript]
        #endif
    }

    var placeholder: String {
        switch self {
        case .page, .firefox: return "example.com"
        case .javascript: return "6 * 7"
        case .python: return "print('hello')"
        }
    }
}

enum ConsoleKind: String, CaseIterable, Identifiable {
    case js = "JS"
    case py = "🐍"
    var id: String { rawValue }
}

@MainActor
final class Model: ObservableObject {
    @Published var mode: Mode = .page
    @Published var input = ""
    @Published var output = "\(Engine.version)\n\(Engine.banner)"
    @Published var busy = false
    @Published var webURL: URL?

    // Firefox-mode console
    let web = WebController()
    @Published var consoleKind: ConsoleKind = .js
    @Published var consoleInput = ""
    @Published var consoleOutput = ""
    @Published var consoleBusy = false

    func run() {
        let text = input.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty, !busy else { return }
        if mode == .firefox {
            webURL = FirefoxRenderer.url(from: text)
            return
        }
        busy = true
        let mode = mode
        Task {
            let result: String
            switch mode {
            case .page:
                do { result = try await Engine.openPage(text) }
                catch { result = "Load error: \(error.localizedDescription)" }
            case .javascript:
                result = await Task.detached { Engine.runJavaScript(text) }.value
            case .python:
                result = await Task.detached { Engine.runPython(text) }.value
            case .firefox:
                result = ""
            }
            output = result
            busy = false
        }
    }

    func runConsole() {
        let code = consoleInput.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !code.isEmpty, !consoleBusy else { return }
        consoleBusy = true
        let kind = consoleKind
        Task {
            switch kind {
            case .js: consoleOutput = await web.runJS(code)
            case .py:
                consoleOutput = "Running Python (first run downloads Pyodide)..."
                consoleOutput = await web.runPython(code)
            }
            consoleBusy = false
        }
    }
}

struct ContentView: View {
    @StateObject private var model = Model()

    var body: some View {
        VStack(spacing: 12) {
            Picker("Mode", selection: $model.mode) {
                ForEach(Mode.available) { Text($0.rawValue).tag($0) }
            }
            .pickerStyle(.segmented)
            .labelsHidden()

            HStack {
                TextField(model.mode.placeholder, text: $model.input)
                    .textFieldStyle(.roundedBorder)
                    .font(.system(.body, design: .monospaced))
                    .autocorrectionDisabled()
                    #if os(iOS)
                    .textInputAutocapitalization(.never)
                    #endif
                    .submitLabel(.go)
                    .onSubmit { model.run() }
                Button(model.mode == .page || model.mode == .firefox ? "Open" : "Run") { model.run() }
                    .buttonStyle(.borderedProminent)
                    .disabled(model.busy)
            }

            if model.mode == .firefox {
                WebRenderView(controller: model.web, url: model.webURL)
                    .clipShape(RoundedRectangle(cornerRadius: 8))
                console
            } else {
                ScrollView {
                    Text(model.output)
                        .font(.system(.footnote, design: .monospaced))
                        .textSelection(.enabled)
                        .frame(maxWidth: .infinity, alignment: .topLeading)
                        .padding(8)
                }
                .background(.quaternary)
                .clipShape(RoundedRectangle(cornerRadius: 8))
                .overlay(alignment: .topTrailing) {
                    if model.busy { ProgressView().padding(8) }
                }
            }
        }
        .padding()
    }

    private var console: some View {
        VStack(spacing: 8) {
            HStack {
                Picker("Console", selection: $model.consoleKind) {
                    ForEach(ConsoleKind.allCases) { Text($0.rawValue).tag($0) }
                }
                .pickerStyle(.segmented)
                .labelsHidden()
                .frame(maxWidth: 120)

                TextField(model.consoleKind == .js ? "document.title" : "print(2 ** 10)",
                          text: $model.consoleInput)
                    .textFieldStyle(.roundedBorder)
                    .font(.system(.body, design: .monospaced))
                    .autocorrectionDisabled()
                    #if os(iOS)
                    .textInputAutocapitalization(.never)
                    #endif
                    .onSubmit { model.runConsole() }

                Button("Run") { model.runConsole() }
                    .buttonStyle(.bordered)
                    .disabled(model.consoleBusy)
            }
            if !model.consoleOutput.isEmpty {
                ScrollView {
                    Text(model.consoleOutput)
                        .font(.system(.footnote, design: .monospaced))
                        .textSelection(.enabled)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(6)
                }
                .frame(maxHeight: 110)
                .background(.quaternary)
                .clipShape(RoundedRectangle(cornerRadius: 6))
            }
        }
    }
}
