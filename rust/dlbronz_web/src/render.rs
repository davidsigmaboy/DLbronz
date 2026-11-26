//! Glue: Doc + CSS -> display list (JSON).

use crate::css::{compute_styles, parse_sheet, Color, Display, UA_CSS};
use crate::doc::{Doc, Kind};
use crate::layout::{Item, Layouter, Measure};

pub struct Page {
    pub height: f32,
    pub bg: Option<Color>,
    pub title: String,
    pub items: Vec<Item>,
}

pub fn render(doc: &Doc, extra_css: &str, width: f32, measure: Measure) -> Page {
    // external CSS first, then <style> elements in document order
    let mut author_css = String::from(extra_css);
    author_css.push('\n');
    let mut title = String::new();
    for id in 0..doc.nodes.len() {
        if doc.nodes[id].kind != Kind::Element {
            continue;
        }
        match doc.nodes[id].tag.as_str() {
            "style" => {
                author_css.push_str(&doc.text_content(id));
                author_css.push('\n');
            }
            "title" if title.is_empty() => title = doc.text_content(id).trim().to_string(),
            _ => {}
        }
    }

    let ua = parse_sheet(UA_CSS, width);
    let author = parse_sheet(&author_css, width);
    let styles = compute_styles(doc, &[(&ua, 0), (&author, 1)], width);

    let mut layouter = Layouter::new(doc, &styles, measure);
    let mut height = 0.0;
    let mut bg = None;
    if let Some(html) = doc.find_tag("html") {
        if styles[html].display != Display::None {
            height = layouter.block(html, 0.0, 0.0, width);
        }
        if styles[html].bg.a > 0 {
            bg = Some(styles[html].bg);
        } else if let Some(body) = doc.find_tag("body") {
            if styles[body].bg.a > 0 {
                bg = Some(styles[body].bg);
            }
        }
    }
    Page { height, bg, title, items: layouter.items }
}

fn esc(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
}

fn num(v: f32) -> String {
    if v.is_finite() {
        format!("{:.2}", v)
    } else {
        "0".into()
    }
}

pub fn page_to_json(p: &Page) -> String {
    let mut o = String::with_capacity(4096);
    o.push_str(&format!("{{\"height\":{},", num(p.height)));
    match p.bg {
        Some(c) => o.push_str(&format!("\"bg\":\"{}\",", c.hex())),
        None => o.push_str("\"bg\":null,"),
    }
    o.push_str("\"title\":\"");
    esc(&p.title, &mut o);
    o.push_str("\",\"items\":[");
    for (i, it) in p.items.iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        match it {
            Item::Rect { x, y, w, h, color } => {
                o.push_str(&format!(
                    "{{\"k\":\"r\",\"x\":{},\"y\":{},\"w\":{},\"h\":{},\"c\":\"{}\"}}",
                    num(*x),
                    num(*y),
                    num(*w),
                    num(*h),
                    color.hex()
                ));
            }
            Item::Text { x, y, w, size, bold, italic, underline, mono, color, text, href } => {
                o.push_str(&format!(
                    "{{\"k\":\"t\",\"x\":{},\"y\":{},\"w\":{},\"s\":{},\"b\":{},\"i\":{},\"u\":{},\"m\":{},\"c\":\"{}\",\"t\":\"",
                    num(*x),
                    num(*y),
                    num(*w),
                    num(*size),
                    *bold as u8,
                    *italic as u8,
                    *underline as u8,
                    *mono as u8,
                    color.hex()
                ));
                esc(text, &mut o);
                o.push('"');
                if let Some(h) = href {
                    o.push_str(",\"href\":\"");
                    esc(h, &mut o);
                    o.push('"');
                }
                o.push('}');
            }
        }
    }
    o.push_str("]}");
    o
}

/// DOM as JSON for the JavaScript DOM: element `{"t":"e","n":tag,"a":{..},"c":[..]}`,
/// text `{"t":"x","v":".."}`, document `{"t":"d","c":[..]}`.
pub fn doc_to_json(doc: &Doc) -> String {
    fn walk(doc: &Doc, id: usize, o: &mut String) {
        let n = &doc.nodes[id];
        match n.kind {
            Kind::Text => {
                o.push_str("{\"t\":\"x\",\"v\":\"");
                esc(&n.text, o);
                o.push_str("\"}");
            }
            Kind::Element | Kind::Document => {
                if n.kind == Kind::Document {
                    o.push_str("{\"t\":\"d\"");
                } else {
                    o.push_str("{\"t\":\"e\",\"n\":\"");
                    esc(&n.tag, o);
                    o.push_str("\",\"a\":{");
                    for (i, (k, v)) in n.attrs.iter().enumerate() {
                        if i > 0 {
                            o.push(',');
                        }
                        o.push('"');
                        esc(k, o);
                        o.push_str("\":\"");
                        esc(v, o);
                        o.push('"');
                    }
                    o.push('}');
                }
                o.push_str(",\"c\":[");
                for (i, &c) in n.children.iter().enumerate() {
                    if i > 0 {
                        o.push(',');
                    }
                    walk(doc, c, o);
                }
                o.push_str("]}");
            }
        }
    }
    let mut o = String::new();
    walk(doc, 0, &mut o);
    o
}
