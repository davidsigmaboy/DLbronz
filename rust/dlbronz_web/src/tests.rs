//! Core tests. They use a tiny test-only HTML reader so they run without crates.

use crate::css::*;
use crate::doc::Doc;
use crate::layout::Item;
use crate::render::{doc_to_json, page_to_json, render};

const VOID: [&str; 6] = ["br", "img", "hr", "meta", "link", "input"];

/// Very small, well-formed-HTML reader for tests only.
fn read_html(src: &str) -> Doc {
    let mut doc = Doc::new();
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut stack: Vec<usize> = vec![0];
    while i < chars.len() {
        if chars[i] == '<' {
            let mut j = i;
            while j < chars.len() && chars[j] != '>' {
                j += 1;
            }
            let inner: String = chars[i + 1..j].iter().collect();
            i = j + 1;
            if let Some(name) = inner.strip_prefix('/') {
                let name = name.trim().to_ascii_lowercase();
                while stack.len() > 1 {
                    let top = stack.pop().unwrap();
                    if doc.tag(top) == name {
                        break;
                    }
                }
                continue;
            }
            let self_close = inner.ends_with('/');
            let inner = inner.trim_end_matches('/').to_string();
            let mut it = inner.splitn(2, char::is_whitespace);
            let tag = it.next().unwrap().to_ascii_lowercase();
            let mut attrs = Vec::new();
            if let Some(rest) = it.next() {
                for part in rest.split('"').collect::<Vec<_>>().chunks(2) {
                    if part.len() == 2 {
                        let k = part[0].trim().trim_end_matches('=').trim().to_string();
                        attrs.push((k, part[1].to_string()));
                    }
                }
            }
            let parent = *stack.last().unwrap();
            let id = doc.add_element(parent, &tag, attrs);
            if tag == "style" || tag == "script" {
                let mut k = i;
                let close = format!("</{}>", tag);
                let rest: String = chars[i..].iter().collect();
                if let Some(p) = rest.find(&close) {
                    let text: String = rest[..p].to_string();
                    doc.add_text(id, &text);
                    k = i + rest[..p].chars().count() + close.chars().count();
                }
                i = k;
                continue;
            }
            if !VOID.contains(&tag.as_str()) && !self_close {
                stack.push(id);
            }
        } else {
            let mut j = i;
            while j < chars.len() && chars[j] != '<' {
                j += 1;
            }
            let text: String = chars[i..j].iter().collect();
            i = j;
            let parent = *stack.last().unwrap();
            doc.add_text(parent, &text);
        }
    }
    doc
}

fn measure(text: &str, size: f32, _b: bool, _i: bool, _m: bool) -> f32 {
    text.chars().count() as f32 * size * 0.5
}

fn page(html: &str, css: &str, w: f32) -> crate::render::Page {
    render(&read_html(html), css, w, &measure)
}

fn texts(p: &crate::render::Page) -> Vec<(String, f32, f32)> {
    p.items
        .iter()
        .filter_map(|i| match i {
            Item::Text { text, x, y, .. } => Some((text.clone(), *x, *y)),
            _ => None,
        })
        .collect()
}

#[test]
fn colors() {
    assert_eq!(Color::parse("#fff"), Some(Color::rgb(255, 255, 255)));
    assert_eq!(Color::parse("#336699"), Some(Color::rgb(0x33, 0x66, 0x99)));
    assert_eq!(Color::parse("rgb(10, 20, 30)"), Some(Color::rgb(10, 20, 30)));
    assert_eq!(Color::parse("rgba(0,0,0,0.5)").unwrap().a, 128);
    assert_eq!(Color::parse("red"), Some(Color::rgb(255, 0, 0)));
    assert_eq!(Color::parse("hsl(0, 100%, 50%)"), Some(Color::rgb(255, 0, 0)));
    assert_eq!(Color::parse("nonsense"), None);
}

#[test]
fn lengths() {
    assert_eq!(parse_len("12px", 16.0), Some(Len::Px(12.0)));
    assert_eq!(parse_len("2em", 10.0), Some(Len::Px(20.0)));
    assert_eq!(parse_len("1.5rem", 10.0), Some(Len::Px(24.0)));
    assert_eq!(parse_len("50%", 16.0), Some(Len::Pct(50.0)));
    assert_eq!(parse_len("auto", 16.0), Some(Len::Auto));
    assert_eq!(parse_len("calc(1px + 2px)", 16.0), None);
}

#[test]
fn selectors_match() {
    let doc = read_html(r#"<div class="a b" id="x"><p class="c"><span>t</span></p></div>"#);
    let span = doc.find_tag("span").unwrap();
    let p = doc.find_tag("p").unwrap();
    for (sel, expect) in [
        ("span", true),
        ("p span", true),
        ("div span", true),
        ("div > span", false),
        ("p > span", true),
        (".a .c span", true),
        ("#x span", true),
        ("div.a.b > p.c > span", true),
        ("[class] span", true),
        ("div.z span", false),
    ] {
        let s = parse_selector(sel).unwrap_or_else(|| panic!("parse {sel}"));
        assert_eq!(matches(&doc, span, &s), expect, "selector {sel}");
    }
    assert!(parse_selector("a:hover").is_none());
    assert!(parse_selector("a + b").is_none());
    assert!(matches(&doc, p, &parse_selector("*").unwrap()));
}

#[test]
fn cascade_and_specificity() {
    let p = page(
        r#"<html><body><p id="a" class="b">hi</p></body></html>"#,
        "p { color: red } .b { color: blue } #a { color: #00ff00 }",
        400.0,
    );
    let color = p.items.iter().find_map(|i| if let Item::Text { color, .. } = i { Some(*color) } else { None }).unwrap();
    assert_eq!(color, Color::rgb(0, 255, 0));

    let p = page(r#"<html><body><p class="b" style="color:red">hi</p></body></html>"#, ".b { color: blue !important }", 400.0);
    let color = p.items.iter().find_map(|i| if let Item::Text { color, .. } = i { Some(*color) } else { None }).unwrap();
    assert_eq!(color, Color::rgb(0, 0, 255), "!important beats inline");
}

#[test]
fn css_variables_and_media() {
    let css = ":root { --c: #ff0000; --pad: 10px } p { color: var(--c); } @media (max-width: 300px) { p { color: #0000ff } } @media print { p { color: #00ff00 } }";
    let wide = page(r#"<html><body><p>x</p></body></html>"#, css, 800.0);
    let narrow = page(r#"<html><body><p>x</p></body></html>"#, css, 250.0);
    let col = |p: &crate::render::Page| p.items.iter().find_map(|i| if let Item::Text { color, .. } = i { Some(*color) } else { None }).unwrap();
    assert_eq!(col(&wide), Color::rgb(255, 0, 0));
    assert_eq!(col(&narrow), Color::rgb(0, 0, 255));
}

#[test]
fn block_stacking_and_margins() {
    let p = page(r#"<html><body style="margin:0"><div style="height:50px"></div><p style="margin:20px 0">a</p><p style="margin:20px 0">b</p></body></html>"#, "", 400.0);
    let t = texts(&p);
    let ya = t.iter().find(|x| x.0 == "a").unwrap().2;
    let yb = t.iter().find(|x| x.0 == "b").unwrap().2;
    // 20px margins collapse between the paragraphs: gap = 20 + one 19.2px line
    assert!((yb - ya - (20.0 + 19.2)).abs() < 0.6, "ya={ya} yb={yb}");
    assert!(ya >= 70.0 - 0.1, "first paragraph starts after the 50px div + 20px margin");
}

#[test]
fn text_wraps_at_width() {
    // 0.5 * 16 = 8px per char; "aaaa bbbb cccc" words 32px, space 8px; width 80 fits 2 words
    let p = page(r#"<html><body style="margin:0"><p style="margin:0">aaaa bbbb cccc dddd</p></body></html>"#, "", 80.0);
    let t = texts(&p);
    assert_eq!(t.len(), 2, "{t:?}");
    assert_eq!(t[0].0, "aaaa bbbb");
    assert_eq!(t[1].0, "cccc dddd");
    assert!(t[1].2 > t[0].2);
}

#[test]
fn inline_runs_merge_and_styles_split() {
    let p = page(r#"<html><body style="margin:0"><p style="margin:0">one <b>two</b> three</p></body></html>"#, "", 600.0);
    let t = texts(&p);
    let names: Vec<&str> = t.iter().map(|x| x.0.as_str()).collect();
    assert_eq!(names, vec!["one", "two", "three"]);
    // bold run starts after "one" + space
    assert!((t[1].1 - 32.0).abs() < 0.1, "{t:?}");
}

#[test]
fn links_carry_href() {
    let p = page(r#"<html><body><p><a href="/next">go</a></p></body></html>"#, "", 300.0);
    let href = p.items.iter().find_map(|i| if let Item::Text { href, .. } = i { href.clone() } else { None });
    assert_eq!(href.as_deref(), Some("/next"));
}

#[test]
fn display_none_and_script_hidden() {
    let p = page(r#"<html><head><title>T</title><style>.h{display:none}</style></head><body><p class="h">hidden</p><p>shown</p><script>alert(1)</script></body></html>"#, "", 300.0);
    let t = texts(&p);
    assert_eq!(t.len(), 1);
    assert_eq!(t[0].0, "shown");
    assert_eq!(p.title, "T");
}

#[test]
fn flex_row_splits_width() {
    let p = page(
        r#"<html><body style="margin:0"><div style="display:flex"><div>left</div><div>right</div></div></body></html>"#,
        "",
        400.0,
    );
    let t = texts(&p);
    let l = t.iter().find(|x| x.0 == "left").unwrap();
    let r = t.iter().find(|x| x.0 == "right").unwrap();
    assert!((l.2 - r.2).abs() < 0.1, "same row");
    assert!((r.1 - 200.0).abs() < 0.1, "{t:?}");
}

#[test]
fn margin_auto_centers() {
    let p = page(
        r#"<html><body style="margin:0"><div style="width:100px;margin:0 auto;background:#ff0000">x</div></body></html>"#,
        "",
        300.0,
    );
    let rect = p.items.iter().find_map(|i| if let Item::Rect { x, w, .. } = i { Some((*x, *w)) } else { None }).unwrap();
    assert!((rect.0 - 100.0).abs() < 0.1 && (rect.1 - 100.0).abs() < 0.1, "{rect:?}");
}

#[test]
fn background_and_border_rects() {
    let p = page(
        r#"<html><body style="margin:0"><div style="background:#00ff00;border:2px solid #000000;padding:5px">x</div></body></html>"#,
        "",
        200.0,
    );
    let rects: Vec<_> = p.items.iter().filter(|i| matches!(i, Item::Rect { .. })).collect();
    assert_eq!(rects.len(), 5, "bg + 4 borders");
}

#[test]
fn list_markers() {
    let p = page(r#"<html><body><ul><li>a</li></ul><ol><li>x</li><li>y</li></ol></body></html>"#, "", 300.0);
    let t: Vec<String> = texts(&p).into_iter().map(|x| x.0).collect();
    assert!(t.contains(&"\u{2022}".to_string()));
    assert!(t.contains(&"1.".to_string()) && t.contains(&"2.".to_string()), "{t:?}");
}

#[test]
fn block_inside_inline_link() {
    let p = page(r#"<html><body style="margin:0"><a href="/x"><div>one</div><div>two</div></a></body></html>"#, "", 300.0);
    let t = texts(&p);
    assert_eq!(t.len(), 2);
    assert!(t[1].2 > t[0].2);
}

#[test]
fn json_is_well_formed_enough() {
    let doc = read_html(r#"<html><body><p class="a">q"uote\ and
newline</p></body></html>"#);
    let j = doc_to_json(&doc);
    assert!(j.starts_with("{\"t\":\"d\""));
    assert!(j.contains("\\\"") && j.contains("\\n"));
    let p = page(r#"<html><body><p>x</p></body></html>"#, "", 100.0);
    let j = page_to_json(&p);
    assert!(j.starts_with("{\"height\":") && j.ends_with("]}"));
}
