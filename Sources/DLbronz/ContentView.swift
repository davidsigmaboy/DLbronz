import SwiftUI
import DLbronzEngine

enum Mode: String, CaseIterable, Identifiable {
    case page = "Page"
    case javascript = "JavaScript"
    case python = "Python"
    var id: String { rawValue }

    var placeholder: String {
        switch self {
        case .page: return "example.com"
        case .javascript: return "6 * 7"
        case .python: return "print('hello')"
        }
    }
}

@MainActor
final class Model: ObservableObject {
    @Published var mode: Mode = .page
    @Published var input = ""
    @Published var output = "\(Engine.version)\n\(Engine.banner)"
    @Published var busy = false

    func run() {
        let text = input.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty, !busy else { return }
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
            }
            output = result
            busy = false
        }
    }
}

struct ContentView: View {
    @StateObject private var model = Model()

    var body: some View {
        VStack(spacing: 12) {
            Picker("Mode", selection: $model.mode) {
                ForEach(Mode.allCases) { Text($0.rawValue).tag($0) }
            }
            .pickerStyle(.segmented)
            .labelsHidden()

            HStack {
                TextField(model.mode.placeholder, text: $model.input)
                    .textFieldStyle(.roundedBorder)
                    .font(.system(.body, design: .monospaced))
                    .onSubmit { model.run() }
                Button(model.mode == .page ? "Open" : "Run") { model.run() }
                    .keyboardShortcut(.return, modifiers: .command)
                    .disabled(model.busy)
            }

            ScrollView {
                Text(model.output)
                    .font(.system(.body, design: .monospaced))
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .topLeading)
                    .padding(8)
            }
            .background(Color(nsColor: .textBackgroundColor))
            .clipShape(RoundedRectangle(cornerRadius: 6))
            .overlay(alignment: .topTrailing) {
                if model.busy { ProgressView().controlSize(.small).padding(8) }
            }
        }
        .padding()
    }
}
