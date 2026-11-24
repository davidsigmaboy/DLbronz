# DLbronz

Экспериментальное macOS-приложение (SwiftUI) на нескольких языках. Без Chromium / WebKit / Gecko.

- **Swift / SwiftUI**: окно приложения, сеть, JavaScript через JavaScriptCore
- **C++**: рендерер, который достаёт видимый текст из HTML (`Sources/CppRenderer`)
- **C**: проверка URL и версия движка (`Sources/CRuntime`)
- **Python**: запуск системного `python3` (только macOS)

В приложении три режима: **Page** (загрузка страницы и вывод текста), **JavaScript** и **Python**.

## Сборка

Локально, на macOS 13+ с Xcode / Swift 5.9+:

```bash
swift run DLbronz            # запуск приложения
swift run DLbronz --selftest # быстрая проверка без окна
./scripts/make-app.sh        # dist/DLbronz.app, .zip и .dmg
```

## GitHub Actions

Workflow `.github/workflows/build.yml` на каждый push и pull request собирает universal-приложение (arm64 + x86_64), прогоняет self-test и выкладывает `DLbronz.zip` и `DLbronz.dmg` как artifacts. При пуше тега `v*` (например `v0.1.0`) файлы публикуются в GitHub Release.

## Запуск неподписанного приложения

Приложение подписано ad-hoc, а не Apple Developer ID, поэтому при первом запуске macOS может его заблокировать. Откройте его через правый клик → **Открыть**, или снимите карантин:

```bash
xattr -dr com.apple.quarantine DLbronz.app
```
