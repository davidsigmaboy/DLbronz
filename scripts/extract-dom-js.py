#!/usr/bin/env python3
"""Keeps rust/dlbronz_linux/js/dom.js in sync with the JS DOM embedded in JSDOM.swift.

  extract-dom-js.py          rewrite dom.js
  extract-dom-js.py --check  exit 1 if dom.js is out of date
"""
import re, sys, pathlib

root = pathlib.Path(__file__).resolve().parent.parent
swift = (root / "Sources/DLbronzEngine/JSDOM.swift").read_text()
m = re.search(r'#"""\n(.*?)\n"""#', swift, re.S)
if not m:
    sys.exit("JS block not found in JSDOM.swift")
out = root / "rust/dlbronz_linux/js/dom.js"
js = m.group(1) + "\n"
if "--check" in sys.argv:
    if not out.exists() or out.read_text() != js:
        sys.exit("dom.js is out of date: run scripts/extract-dom-js.py")
    print("dom.js in sync")
else:
    out.write_text(js)
    print("wrote", out)
