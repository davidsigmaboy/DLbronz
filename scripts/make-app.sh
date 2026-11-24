#!/usr/bin/env bash
# Builds dist/DLbronz.app (universal, ad-hoc signed) plus zip and dmg.
set -euo pipefail

cd "$(dirname "$0")/.."

swift build -c release --arch arm64 --arch x86_64
BIN="$(swift build -c release --arch arm64 --arch x86_64 --show-bin-path)/DLbronz"

APP="dist/DLbronz.app"
rm -rf dist
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/DLbronz"
cp Resources/Info.plist "$APP/Contents/Info.plist"

codesign --force --deep --sign - "$APP"

ditto -c -k --keepParent "$APP" dist/DLbronz-macos.zip
hdiutil create -volname DLbronz -srcfolder "$APP" -ov -format UDZO dist/DLbronz.dmg

echo "Built:"
ls -la dist
