#!/usr/bin/env bash
# Builds Frameworks/DLbronzWeb.xcframework (iOS device, iOS simulator, macOS; arm64).
# Requires macOS with Xcode and rustup.
set -euo pipefail

cd "$(dirname "$0")/.."
CRATE=rust/dlbronz_web
OUT=Frameworks/DLbronzWeb.xcframework

export IPHONEOS_DEPLOYMENT_TARGET=16.0
export MACOSX_DEPLOYMENT_TARGET=13.0

TARGETS=(aarch64-apple-ios aarch64-apple-ios-sim aarch64-apple-darwin)
rustup target add "${TARGETS[@]}"

for t in "${TARGETS[@]}"; do
  cargo build --release --manifest-path "$CRATE/Cargo.toml" --target "$t"
done

rm -rf "$OUT"
mkdir -p Frameworks
ARGS=()
for t in "${TARGETS[@]}"; do
  ARGS+=(-library "$CRATE/target/$t/release/libdlbronz_web.a" -headers "$CRATE/include")
done
xcodebuild -create-xcframework "${ARGS[@]}" -output "$OUT"
echo "Built $OUT"
