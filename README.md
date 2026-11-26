# DLbronz

Экспериментальное iOS-приложение (SwiftUI) на нескольких языках. Без Chromium / WebKit / Gecko.

- **Swift / SwiftUI**: интерфейс, сеть, JavaScript через JavaScriptCore (`App/`, `Sources/DLbronzEngine`)
- **C++**: рендерер, который достаёт видимый текст из HTML (`Sources/CppRenderer`)
- **C**: проверка URL и версия движка (`Sources/CRuntime`)

Режимы: **Page** (загрузка страницы и вывод текста, оригинальное ядро), **Firefox** и **JavaScript**.

**Render** это наш собственный рендерер страниц. Ядро на Rust (`rust/dlbronz_web`): парсер HTML5 **html5ever** (тот же, что в Servo и Firefox), каскад CSS (селекторы, specificity, `!important`, `var()`, `@media`, flex-строки, таблицы, списки), блочная и строчная вёрстка. Рисует SwiftUI Canvas. Для скриптов страницы есть свой JS-DOM поверх JavaScriptCore (`Sources/DLbronzEngine/JSDOM.swift`): `document`, `querySelector`, `innerHTML` (через html5ever), события, таймеры. Скрипты выполняются при загрузке, после них страница верстается заново. Картинки пока серые заглушки, интерактивности после загрузки нет, `fetch` не поддерживается. Это не Gecko и не код Firefox, кроме html5ever.

**Firefox** это необязательный рендерер (`App/WebRenderView.swift`): WKWebView с User-Agent Firefox для iOS. Это не Gecko: Gecko нельзя встроить в iOS-приложение, и сам Firefox для iOS рисует страницы через WebKit. Ядро `DLbronzEngine` этим режимом не затронуто.

Под веб-видом в режиме Firefox есть консоль: **JS** выполняет код прямо в открытой странице, **🐍** выполняет Python через Pyodide (CPython в WebAssembly, работает и на iOS). Pyodide подгружается с jsdelivr при первом запуске, нужен интернет. Режим Python на iOS недоступен: в песочнице нет системного интерпретатора. На macOS он работает через `python3`.

## Сборка .ipa в GitHub Actions

Workflow `.github/workflows/build.yml` на каждый push и pull request:

1. прогоняет тесты движка (`swift test`),
2. генерирует Xcode-проект из `project.yml` (XcodeGen),
3. собирает приложение без подписи и пакует `DLbronz.ipa`,
4. выкладывает его как artifact `DLbronz-ipa`.

При пуше тега `v*` (например `v0.1.0`) `.ipa` публикуется в GitHub Release.

## Установка

`.ipa` **не подписан**, поэтому напрямую на iPhone он не встанет. Его нужно подписать своим Apple ID одним из способов: AltStore, Sideloadly, SideStore, либо установить через TrollStore.

## Локально

```bash
./scripts/test-core.sh               # тесты Rust-ядра, нужен только rustc
./scripts/build-rust-xcframework.sh  # Frameworks/DLbronzWeb.xcframework, нужен перед swift test / xcodegen
brew install xcodegen
xcodegen generate
open DLbronz.xcodeproj
swift test   # тесты движка на macOS
```
