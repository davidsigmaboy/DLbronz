#!/usr/bin/env bash
# Runs the dependency-free core tests (css / layout / render) with plain rustc.
set -euo pipefail
cd "$(dirname "$0")/../rust/dlbronz_web"
OUT="${TMPDIR:-/tmp}/dlbronz_core_tests"
rustc --edition 2021 --test --cfg dlbronz_core_only src/lib.rs -o "$OUT"
"$OUT"
