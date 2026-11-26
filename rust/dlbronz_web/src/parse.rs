//! HTML5 parsing via html5ever (through `scraper`).

use crate::doc::Doc;
use scraper::{ElementRef, Html};

fn walk(el: ElementRef, parent: usize, doc: &mut Doc) {
    for child in el.children() {
        if let Some(ce) = ElementRef::wrap(child) {
            let v = ce.value();
            let attrs: Vec<(String, String)> = v.attrs().map(|(k, val)| (k.to_string(), val.to_string())).collect();
            let id = doc.add_element(parent, v.name(), attrs);
            walk(ce, id, doc);
        } else if let Some(t) = child.value().as_text() {
            let s: &str = t;
            doc.add_text(parent, s);
        }
    }
}

/// Parse a full document. The returned `Doc` has `<html>` as the only child of the root.
pub fn parse_document(html: &str) -> Doc {
    let parsed = Html::parse_document(html);
    let mut doc = Doc::new();
    let root = parsed.root_element();
    let v = root.value();
    let attrs: Vec<(String, String)> = v.attrs().map(|(k, val)| (k.to_string(), val.to_string())).collect();
    let id = doc.add_element(0, v.name(), attrs);
    walk(root, id, &mut doc);
    doc
}

/// Parse a fragment (for `innerHTML`). Children are attached directly to the root.
pub fn parse_fragment(html: &str) -> Doc {
    let parsed = Html::parse_fragment(html);
    let mut doc = Doc::new();
    walk(parsed.root_element(), 0, &mut doc);
    doc
}
