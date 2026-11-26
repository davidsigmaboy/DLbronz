//! CSS: values, selectors, stylesheet parsing and the cascade.

use crate::doc::{Doc, Kind};
use std::collections::HashMap;

// ───────────────────────────── colors & lengths ─────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const CLEAR: Color = Color { r: 0, g: 0, b: 0, a: 0 };

    pub fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b, a: 255 }
    }

    pub fn parse(s: &str) -> Option<Color> {
        let s = s.trim().to_ascii_lowercase();
        if s == "transparent" {
            return Some(Color::CLEAR);
        }
        if let Some(h) = s.strip_prefix('#') {
            return parse_hex(h);
        }
        if s.starts_with("rgb") {
            return parse_rgb(&s);
        }
        if s.starts_with("hsl") {
            return parse_hsl(&s);
        }
        named_color(&s)
    }

    pub fn hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
    }
}

fn parse_hex(h: &str) -> Option<Color> {
    if !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let nib = |i: usize| u8::from_str_radix(&h[i..i + 1], 16).ok().map(|v| v * 17);
    let byte = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok();
    match h.len() {
        3 => Some(Color::rgb(nib(0)?, nib(1)?, nib(2)?)),
        4 => Some(Color { r: nib(0)?, g: nib(1)?, b: nib(2)?, a: nib(3)? }),
        6 => Some(Color::rgb(byte(0)?, byte(2)?, byte(4)?)),
        8 => Some(Color { r: byte(0)?, g: byte(2)?, b: byte(4)?, a: byte(6)? }),
        _ => None,
    }
}

fn func_args(s: &str) -> Option<Vec<String>> {
    let open = s.find('(')?;
    let close = s.rfind(')')?;
    if close <= open {
        return None;
    }
    let inner = s[open + 1..close].replace([',', '/'], " ");
    Some(inner.split_whitespace().map(|x| x.to_string()).collect())
}

fn parse_rgb(s: &str) -> Option<Color> {
    let t = func_args(s)?;
    if t.len() < 3 {
        return None;
    }
    let comp = |x: &str| -> Option<u8> {
        if let Some(p) = x.strip_suffix('%') {
            let v: f32 = p.parse().ok()?;
            Some((v * 2.55).round().clamp(0.0, 255.0) as u8)
        } else {
            let v: f32 = x.parse().ok()?;
            Some(v.round().clamp(0.0, 255.0) as u8)
        }
    };
    let a = if t.len() >= 4 {
        let x = &t[3];
        if let Some(p) = x.strip_suffix('%') {
            (p.parse::<f32>().ok()? * 2.55).round().clamp(0.0, 255.0) as u8
        } else {
            (x.parse::<f32>().ok()? * 255.0).round().clamp(0.0, 255.0) as u8
        }
    } else {
        255
    };
    Some(Color { r: comp(&t[0])?, g: comp(&t[1])?, b: comp(&t[2])?, a })
}

fn parse_hsl(s: &str) -> Option<Color> {
    let t = func_args(s)?;
    if t.len() < 3 {
        return None;
    }
    let h: f32 = t[0].trim_end_matches("deg").parse().ok()?;
    let sat: f32 = t[1].trim_end_matches('%').parse::<f32>().ok()? / 100.0;
    let l: f32 = t[2].trim_end_matches('%').parse::<f32>().ok()? / 100.0;
    let a = if t.len() >= 4 {
        (t[3].parse::<f32>().ok()? * 255.0).round().clamp(0.0, 255.0) as u8
    } else {
        255
    };
    let c = (1.0 - (2.0 * l - 1.0).abs()) * sat;
    let hp = (h.rem_euclid(360.0)) / 60.0;
    let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match hp as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    let f = |v: f32| ((v + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    Some(Color { r: f(r1), g: f(g1), b: f(b1), a })
}

fn named_color(s: &str) -> Option<Color> {
    let (r, g, b) = match s {
        "black" => (0, 0, 0),
        "white" => (255, 255, 255),
        "red" => (255, 0, 0),
        "green" => (0, 128, 0),
        "blue" => (0, 0, 255),
        "gray" | "grey" => (128, 128, 128),
        "silver" => (192, 192, 192),
        "yellow" => (255, 255, 0),
        "orange" => (255, 165, 0),
        "purple" => (128, 0, 128),
        "pink" => (255, 192, 203),
        "brown" => (165, 42, 42),
        "cyan" | "aqua" => (0, 255, 255),
        "magenta" | "fuchsia" => (255, 0, 255),
        "navy" => (0, 0, 128),
        "teal" => (0, 128, 128),
        "maroon" => (128, 0, 0),
        "lime" => (0, 255, 0),
        "olive" => (128, 128, 0),
        "gold" => (255, 215, 0),
        "indigo" => (75, 0, 130),
        "violet" => (238, 130, 238),
        "coral" => (255, 127, 80),
        "crimson" => (220, 20, 60),
        "darkgray" | "darkgrey" => (169, 169, 169),
        "lightgray" | "lightgrey" => (211, 211, 211),
        "whitesmoke" => (245, 245, 245),
        "darkblue" => (0, 0, 139),
        "darkgreen" => (0, 100, 0),
        "darkred" => (139, 0, 0),
        "lightblue" => (173, 216, 230),
        "lightgreen" => (144, 238, 144),
        "skyblue" => (135, 206, 235),
        "tomato" => (255, 99, 71),
        "royalblue" => (65, 105, 225),
        "steelblue" => (70, 130, 180),
        _ => return None,
    };
    Some(Color::rgb(r, g, b))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Len {
    Auto,
    Px(f32),
    Pct(f32),
}

fn split_num(s: &str) -> Option<(f32, &str)> {
    let end = s
        .char_indices()
        .find(|&(i, c)| !(c.is_ascii_digit() || c == '.' || ((c == '-' || c == '+') && i == 0)))
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    if end == 0 {
        return None;
    }
    let n: f32 = s[..end].parse().ok()?;
    Some((n, &s[end..]))
}

/// `fs` is the font size that `em` resolves against.
pub fn parse_len(s: &str, fs: f32) -> Option<Len> {
    let s = s.trim().to_ascii_lowercase();
    if s == "auto" {
        return Some(Len::Auto);
    }
    let (n, unit) = split_num(&s)?;
    match unit {
        "px" => Some(Len::Px(n)),
        "" if n == 0.0 => Some(Len::Px(0.0)),
        "em" => Some(Len::Px(n * fs)),
        "rem" => Some(Len::Px(n * 16.0)),
        "%" => Some(Len::Pct(n)),
        "pt" => Some(Len::Px(n * 4.0 / 3.0)),
        "ch" | "ex" => Some(Len::Px(n * fs * 0.5)),
        _ => None,
    }
}

// ───────────────────────────── computed style ─────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Display {
    None,
    Block,
    Inline,
    Flex,
    ListItem,
    TableRow,
    TableCell,
}

impl Display {
    pub fn is_block_level(self) -> bool {
        matches!(self, Display::Block | Display::ListItem | Display::Flex | Display::TableRow | Display::TableCell)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LineH {
    Normal,
    Factor(f32),
    Px(f32),
}

#[derive(Clone, Debug)]
pub struct Style {
    pub display: Display,
    pub color: Color,
    pub bg: Color,
    pub font_size: f32,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub mono: bool,
    pub pre: bool,
    pub text_align: TextAlign,
    pub line_height: LineH,
    pub list_none: bool,
    /// margin / padding / border order: top, right, bottom, left
    pub margin: [Len; 4],
    pub padding: [Len; 4],
    pub border: [(f32, Color); 4],
    pub width: Len,
    pub height: Len,
    pub max_width: Len,
    pub min_height: f32,
    pub border_box: bool,
    pub flex_col: bool,
    pub gap: f32,
}

impl Style {
    pub fn root() -> Style {
        Style {
            display: Display::Block,
            color: Color::BLACK,
            bg: Color::CLEAR,
            font_size: 16.0,
            bold: false,
            italic: false,
            underline: false,
            mono: false,
            pre: false,
            text_align: TextAlign::Left,
            line_height: LineH::Normal,
            list_none: false,
            margin: [Len::Px(0.0); 4],
            padding: [Len::Px(0.0); 4],
            border: [(0.0, Color::BLACK); 4],
            width: Len::Auto,
            height: Len::Auto,
            max_width: Len::Auto,
            min_height: 0.0,
            border_box: false,
            flex_col: false,
            gap: 0.0,
        }
    }

    /// Style for a child: inherited properties copied, the rest reset.
    pub fn inherit(&self) -> Style {
        let mut s = Style::root();
        s.display = Display::Inline;
        s.color = self.color;
        s.font_size = self.font_size;
        s.bold = self.bold;
        s.italic = self.italic;
        s.underline = self.underline;
        s.mono = self.mono;
        s.pre = self.pre;
        s.text_align = self.text_align;
        s.line_height = self.line_height;
        s.list_none = self.list_none;
        s
    }

    pub fn line_px(&self) -> f32 {
        match self.line_height {
            LineH::Normal => self.font_size * 1.2,
            LineH::Factor(f) => self.font_size * f,
            LineH::Px(p) => p,
        }
    }
}

// ───────────────────────────── declarations ─────────────────────────────

#[derive(Clone, Debug)]
pub struct Decl {
    pub name: String,
    pub value: String,
    pub important: bool,
}

/// Split on a separator at top level (outside parentheses and quotes).
fn split_top(s: &str, sep: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    for c in s.chars() {
        if let Some(q) = quote {
            cur.push(c);
            if c == q {
                quote = None;
            }
            continue;
        }
        match c {
            '"' | '\'' => {
                quote = Some(c);
                cur.push(c);
            }
            '(' => {
                depth += 1;
                cur.push(c);
            }
            ')' => {
                depth -= 1;
                cur.push(c);
            }
            _ if c == sep && depth <= 0 => {
                out.push(std::mem::take(&mut cur));
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur);
    }
    out
}

fn split_ws_top(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    for c in s.chars() {
        match c {
            '(' => {
                depth += 1;
                cur.push(c);
            }
            ')' => {
                depth -= 1;
                cur.push(c);
            }
            c if c.is_whitespace() && depth <= 0 => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

pub fn parse_decls(block: &str) -> Vec<Decl> {
    let mut out = Vec::new();
    for part in split_top(block, ';') {
        let Some(i) = part.find(':') else { continue };
        let name = part[..i].trim().to_ascii_lowercase();
        let mut value = part[i + 1..].trim().to_string();
        if name.is_empty() || value.is_empty() {
            continue;
        }
        let mut important = false;
        let lower = value.to_ascii_lowercase();
        if let Some(p) = lower.rfind("!important") {
            important = true;
            value = value[..p].trim().to_string();
        }
        out.push(Decl { name, value, important });
    }
    out
}

// ───────────────────────────── selectors ─────────────────────────────

#[derive(Clone, Debug, Default)]
pub struct Compound {
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub attrs: Vec<(String, Option<String>)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Comb {
    Descendant,
    Child,
}

#[derive(Clone, Debug)]
pub struct Selector {
    /// Left to right. The `Comb` relates a part to the part before it.
    pub parts: Vec<(Comb, Compound)>,
    pub spec: u32,
}

fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_' || !c.is_ascii()
}

#[allow(unused_assignments)]
pub fn parse_selector(src: &str) -> Option<Selector> {
    let s: Vec<char> = src.trim().chars().collect();
    if s.is_empty() {
        return None;
    }
    let mut parts: Vec<(Comb, Compound)> = Vec::new();
    let mut cur = Compound::default();
    let mut cur_nonempty = false;
    let mut comb = Comb::Descendant;
    let mut pending: Option<Comb> = None; // combinator seen after a finished compound
    let mut i = 0;

    macro_rules! finish {
        () => {
            if cur_nonempty {
                parts.push((comb, std::mem::take(&mut cur)));
                cur_nonempty = false;
            }
        };
    }

    while i < s.len() {
        let c = s[i];
        if c.is_whitespace() {
            finish!();
            if pending.is_none() && !parts.is_empty() {
                pending = Some(Comb::Descendant);
            }
            i += 1;
            continue;
        }
        if c == '>' {
            finish!();
            if parts.is_empty() {
                return None;
            }
            pending = Some(Comb::Child);
            i += 1;
            continue;
        }
        if let Some(p) = pending.take() {
            comb = p;
        }
        match c {
            '*' => {
                cur_nonempty = true;
                i += 1;
            }
            '.' | '#' => {
                let mut j = i + 1;
                while j < s.len() && is_ident(s[j]) {
                    j += 1;
                }
                if j == i + 1 {
                    return None;
                }
                let name: String = s[i + 1..j].iter().collect();
                if c == '.' {
                    cur.classes.push(name);
                } else {
                    cur.id = Some(name);
                }
                cur_nonempty = true;
                i = j;
            }
            '[' => {
                let mut j = i + 1;
                while j < s.len() && s[j] != ']' {
                    j += 1;
                }
                if j >= s.len() {
                    return None;
                }
                let inner: String = s[i + 1..j].iter().collect();
                if inner.contains("~=") || inner.contains("^=") || inner.contains("$=") || inner.contains("*=") || inner.contains("|=") {
                    return None;
                }
                if let Some(eq) = inner.find('=') {
                    let name = inner[..eq].trim().to_ascii_lowercase();
                    let val = inner[eq + 1..].trim().trim_matches(|c| c == '"' || c == '\'').to_string();
                    cur.attrs.push((name, Some(val)));
                } else {
                    cur.attrs.push((inner.trim().to_ascii_lowercase(), None));
                }
                cur_nonempty = true;
                i = j + 1;
            }
            ':' => {
                // only :root is supported
                let rest: String = s[i..].iter().collect();
                if rest.starts_with(":root") {
                    cur.tag = Some("html".into());
                    cur_nonempty = true;
                    i += 5;
                } else {
                    return None;
                }
            }
            c if is_ident(c) => {
                let mut j = i;
                while j < s.len() && is_ident(s[j]) {
                    j += 1;
                }
                let name: String = s[i..j].iter().collect();
                cur.tag = Some(name.to_ascii_lowercase());
                cur_nonempty = true;
                i = j;
            }
            _ => return None, // + ~ , ( ) etc. are unsupported
        }
    }
    finish!();
    if parts.is_empty() || pending.is_some() && matches!(pending, Some(Comb::Child)) {
        return None;
    }
    let mut spec = 0u32;
    for (_, c) in &parts {
        if c.id.is_some() {
            spec += 10_000;
        }
        spec += (c.classes.len() + c.attrs.len()) as u32 * 100;
        if c.tag.is_some() {
            spec += 1;
        }
    }
    Some(Selector { parts, spec })
}

fn match_compound(doc: &Doc, id: usize, c: &Compound) -> bool {
    if !doc.is_element(id) {
        return false;
    }
    if let Some(t) = &c.tag {
        if doc.tag(id) != t {
            return false;
        }
    }
    if let Some(i) = &c.id {
        if doc.attr(id, "id") != Some(i.as_str()) {
            return false;
        }
    }
    for cl in &c.classes {
        if !doc.has_class(id, cl) {
            return false;
        }
    }
    for (name, val) in &c.attrs {
        match (doc.attr(id, name), val) {
            (None, _) => return false,
            (Some(a), Some(v)) if a != v => return false,
            _ => {}
        }
    }
    true
}

pub fn matches(doc: &Doc, id: usize, sel: &Selector) -> bool {
    fn go(doc: &Doc, id: usize, parts: &[(Comb, Compound)]) -> bool {
        let Some(((comb, comp), rest)) = parts.split_last().map(|(l, r)| (l, r)) else {
            return true;
        };
        if !match_compound(doc, id, comp) {
            return false;
        }
        if rest.is_empty() {
            return true;
        }
        match comb {
            Comb::Child => match doc.nodes[id].parent {
                Some(p) => go(doc, p, rest),
                None => false,
            },
            Comb::Descendant => {
                let mut cur = doc.nodes[id].parent;
                while let Some(p) = cur {
                    if doc.is_element(p) && go(doc, p, rest) {
                        return true;
                    }
                    cur = doc.nodes[p].parent;
                }
                false
            }
        }
    }
    go(doc, id, &sel.parts)
}

// ───────────────────────────── stylesheets ─────────────────────────────

#[derive(Clone, Debug)]
pub struct Rule {
    pub sels: Vec<Selector>,
    pub decls: Vec<Decl>,
}

#[derive(Clone, Debug, Default)]
pub struct Sheet {
    pub rules: Vec<Rule>,
}

fn strip_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let b: Vec<char> = css.chars().collect();
    let mut i = 0;
    while i < b.len() {
        if b[i] == '/' && i + 1 < b.len() && b[i + 1] == '*' {
            i += 2;
            while i + 1 < b.len() && !(b[i] == '*' && b[i + 1] == '/') {
                i += 1;
            }
            i += 2;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    out
}

/// Evaluate a media query against a viewport width. Unknown features count as true.
fn media_matches(query: &str, vw: f32) -> bool {
    let q = query.to_ascii_lowercase();
    for alt in q.split(',') {
        let mut ok = true;
        for part in alt.split(" and ") {
            let p = part.trim();
            if p == "print" || p.contains("not all") || p == "speech" {
                ok = false;
            } else if p.starts_with("not ") || p.starts_with("only print") {
                ok = false;
            } else if let Some(inner) = p.strip_prefix('(').and_then(|x| x.strip_suffix(')')) {
                if let Some((k, v)) = inner.split_once(':') {
                    let k = k.trim();
                    let v = v.trim();
                    let px = parse_len(v, 16.0).and_then(|l| if let Len::Px(x) = l { Some(x) } else { None });
                    match (k, px) {
                        ("min-width", Some(x)) => ok &= vw >= x,
                        ("max-width", Some(x)) => ok &= vw <= x,
                        ("prefers-color-scheme", _) => ok &= v == "light",
                        ("prefers-reduced-motion", _) => ok &= v != "reduce",
                        ("orientation", _) => ok &= v == "portrait",
                        _ => {}
                    }
                }
            }
        }
        if ok {
            return true;
        }
    }
    false
}

pub fn parse_sheet(css: &str, viewport_w: f32) -> Sheet {
    let css = strip_comments(css);
    let chars: Vec<char> = css.chars().collect();
    let mut sheet = Sheet::default();
    parse_into(&chars, viewport_w, &mut sheet, 0);
    sheet
}

fn find_block_end(chars: &[char], open: usize) -> usize {
    let mut depth = 0;
    let mut i = open;
    while i < chars.len() {
        match chars[i] {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            '"' | '\'' => {
                let q = chars[i];
                i += 1;
                while i < chars.len() && chars[i] != q {
                    if chars[i] == '\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    chars.len().saturating_sub(1)
}

fn parse_into(chars: &[char], vw: f32, sheet: &mut Sheet, depth: u32) {
    if depth > 4 {
        return;
    }
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }
        if chars[i] == '@' {
            let mut j = i;
            while j < chars.len() && chars[j] != ';' && chars[j] != '{' {
                j += 1;
            }
            if j >= chars.len() {
                return;
            }
            if chars[j] == ';' {
                i = j + 1;
                continue;
            }
            let header: String = chars[i + 1..j].iter().collect();
            let end = find_block_end(chars, j);
            let name = header.split_whitespace().next().unwrap_or("").to_ascii_lowercase();
            let query = header[name.len().min(header.len())..].trim().to_string();
            if (name == "media" && media_matches(&query, vw)) || name == "supports" || name == "layer" {
                let inner = &chars[(j + 1).min(chars.len())..end.min(chars.len())];
                parse_into(inner, vw, sheet, depth + 1);
            }
            i = end + 1;
            continue;
        }
        // normal rule
        let mut j = i;
        while j < chars.len() && chars[j] != '{' {
            j += 1;
        }
        if j >= chars.len() {
            return;
        }
        let selector_text: String = chars[i..j].iter().collect();
        let end = find_block_end(chars, j);
        let block: String = chars[(j + 1).min(chars.len())..end.min(chars.len())].iter().collect();
        let sels: Vec<Selector> = split_top(&selector_text, ',')
            .iter()
            .filter_map(|s| parse_selector(s))
            .collect();
        let decls = parse_decls(&block);
        if !decls.is_empty() {
            // keep rules with only custom properties even if selectors are unsupported
            if !sels.is_empty() {
                sheet.rules.push(Rule { sels, decls });
            } else {
                let vars: Vec<Decl> = decls.into_iter().filter(|d| d.name.starts_with("--")).collect();
                if !vars.is_empty() {
                    sheet.rules.push(Rule { sels: vec![], decls: vars });
                }
            }
        }
        i = end + 1;
    }
}

pub const UA_CSS: &str = r#"
[hidden], head, script, style, title, meta, link, noscript, template, base, input, textarea, select, svg, canvas, video, audio, iframe, object, embed, option, datalist, dialog { display: none }
html, body, div, section, article, aside, nav, header, footer, main, p, h1, h2, h3, h4, h5, h6, ul, ol, dl, dt, dd, form, fieldset, figure, figcaption, blockquote, pre, address, hr, table, tbody, thead, tfoot, caption, details, summary, center, menu { display: block }
li { display: list-item }
tr { display: table-row }
td, th { display: table-cell }
body { margin: 8px }
p, dl, ul, ol, pre, blockquote, figure { margin: 1em 0 }
ul, ol { padding-left: 40px }
ul ul, ul ol, ol ul, ol ol { margin: 0 }
blockquote, figure { margin: 1em 40px }
dd { margin-left: 40px }
h1 { font-size: 2em; margin: 0.67em 0; font-weight: bold }
h2 { font-size: 1.5em; margin: 0.83em 0; font-weight: bold }
h3 { font-size: 1.17em; margin: 1em 0; font-weight: bold }
h4 { margin: 1.33em 0; font-weight: bold }
h5 { font-size: 0.83em; margin: 1.67em 0; font-weight: bold }
h6 { font-size: 0.67em; margin: 2.33em 0; font-weight: bold }
b, strong, th, dt, summary { font-weight: bold }
i, em, cite, var, address, dfn { font-style: italic }
u, ins, a { text-decoration: underline }
a { color: #0000ee }
small { font-size: 0.83em }
big { font-size: 1.17em }
sub, sup { font-size: 0.83em }
code, kbd, samp, tt, pre { font-family: monospace }
pre { white-space: pre }
center { text-align: center }
th { text-align: center }
td, th { padding: 1px }
caption { text-align: center }
hr { margin: 8px 0; height: 1px; background-color: #999999 }
mark { background-color: #ffff00 }
button { background-color: #efefef; padding: 1px 6px }
"#;

// ───────────────────────────── cascade ─────────────────────────────

fn subst_vars(value: &str, vars: &HashMap<String, String>) -> String {
    let mut v = value.to_string();
    for _ in 0..6 {
        let Some(start) = v.find("var(") else { break };
        let bytes: Vec<char> = v.chars().collect();
        // work on char indices
        let start_c = v[..start].chars().count();
        let mut depth = 0;
        let mut end_c = None;
        for (k, ch) in bytes.iter().enumerate().skip(start_c + 3) {
            if *ch == '(' {
                depth += 1;
            } else if *ch == ')' {
                depth -= 1;
                if depth == 0 {
                    end_c = Some(k);
                    break;
                }
            }
        }
        let Some(end_c) = end_c else { break };
        let inner: String = bytes[start_c + 4..end_c].iter().collect();
        let (name, fallback) = match inner.find(',') {
            Some(c) => (inner[..c].trim().to_string(), Some(inner[c + 1..].trim().to_string())),
            None => (inner.trim().to_string(), None),
        };
        let rep = vars.get(&name).cloned().or(fallback).unwrap_or_default();
        let before: String = bytes[..start_c].iter().collect();
        let after: String = bytes[end_c + 1..].iter().collect();
        v = format!("{}{}{}", before, rep, after);
    }
    v
}

fn box4<T: Copy>(vals: &[T]) -> Option<[T; 4]> {
    match vals.len() {
        1 => Some([vals[0]; 4]),
        2 => Some([vals[0], vals[1], vals[0], vals[1]]),
        3 => Some([vals[0], vals[1], vals[2], vals[1]]),
        4 => Some([vals[0], vals[1], vals[2], vals[3]]),
        _ => None,
    }
}

fn lens(value: &str, fs: f32) -> Option<Vec<Len>> {
    let v: Option<Vec<Len>> = split_ws_top(value).iter().map(|t| parse_len(t, fs)).collect();
    v
}

fn border_width(tok: &str, fs: f32) -> Option<f32> {
    match tok {
        "thin" => Some(1.0),
        "medium" => Some(3.0),
        "thick" => Some(5.0),
        _ => match parse_len(tok, fs) {
            Some(Len::Px(v)) => Some(v),
            _ => None,
        },
    }
}

fn first_color_in(value: &str) -> Option<Color> {
    for tok in split_ws_top(value) {
        if let Some(c) = Color::parse(&tok) {
            return Some(c);
        }
        // gradients: look inside the arguments
        if let (Some(a), Some(b)) = (tok.find('('), tok.rfind(')')) {
            if tok[..a].contains("gradient") {
                for arg in split_top(&tok[a + 1..b], ',') {
                    for t in split_ws_top(&arg) {
                        if let Some(c) = Color::parse(&t) {
                            return Some(c);
                        }
                    }
                }
            }
        }
    }
    None
}

fn apply(st: &mut Style, name: &str, raw: &str, parent_fs: f32) {
    let value = raw.trim();
    let lower = value.to_ascii_lowercase();
    let fs = st.font_size;
    match name {
        "display" => {
            st.display = match lower.as_str() {
                "none" => Display::None,
                "block" | "table" | "grid" | "inline-grid" | "flow-root" | "table-row-group" | "table-header-group" | "table-footer-group" | "table-caption" => Display::Block,
                "inline" | "inline-block" | "contents" | "inline-table" => Display::Inline,
                "flex" | "inline-flex" => Display::Flex,
                "list-item" => Display::ListItem,
                "table-row" => Display::TableRow,
                "table-cell" => Display::TableCell,
                _ => st.display,
            }
        }
        "position" => {
            if lower == "fixed" || lower == "absolute" {
                st.display = Display::None;
            }
        }
        "visibility" => {
            if lower == "hidden" || lower == "collapse" {
                st.display = Display::None;
            }
        }
        "color" => {
            if let Some(c) = Color::parse(value) {
                st.color = c;
            }
        }
        "background" | "background-color" | "background-image" => {
            if lower == "none" || lower == "transparent" {
                if name != "background-image" {
                    st.bg = Color::CLEAR;
                }
            } else if let Some(c) = first_color_in(value) {
                st.bg = c;
            }
        }
        "font-size" => {
            let px = match lower.as_str() {
                "xx-small" => Some(9.0),
                "x-small" => Some(10.0),
                "small" => Some(13.0),
                "medium" => Some(16.0),
                "large" => Some(18.0),
                "x-large" => Some(24.0),
                "xx-large" => Some(32.0),
                "smaller" => Some(parent_fs * 0.83),
                "larger" => Some(parent_fs * 1.2),
                _ => match parse_len(value, parent_fs) {
                    Some(Len::Px(v)) => Some(v),
                    Some(Len::Pct(p)) => Some(parent_fs * p / 100.0),
                    _ => None,
                },
            };
            if let Some(v) = px {
                st.font_size = v.max(1.0);
            }
        }
        "font-weight" => {
            st.bold = match lower.as_str() {
                "bold" | "bolder" => true,
                "normal" | "lighter" => false,
                n => n.parse::<u32>().map(|v| v >= 600).unwrap_or(st.bold),
            }
        }
        "font-style" => st.italic = lower == "italic" || lower == "oblique",
        "font-family" => {
            st.mono = lower.contains("monospace") || lower.contains("courier") || lower.contains("consolas") || lower.contains("menlo") || lower.contains("mono")
        }
        "font" => {
            // shorthand: only pick up weight/style/size/family
            for t in split_ws_top(value) {
                let tl = t.to_ascii_lowercase();
                if tl == "bold" {
                    st.bold = true;
                } else if tl == "italic" {
                    st.italic = true;
                } else if tl.contains('/') || tl.ends_with("px") || tl.ends_with("em") || tl.ends_with('%') || tl.ends_with("pt") {
                    let size = tl.split('/').next().unwrap_or("");
                    if let Some(Len::Px(v)) = parse_len(size, parent_fs) {
                        st.font_size = v;
                    }
                }
            }
            st.mono = lower.contains("monospace");
        }
        "text-decoration" | "text-decoration-line" => {
            if lower.contains("underline") {
                st.underline = true;
            } else if lower.contains("none") {
                st.underline = false;
            }
        }
        "text-align" => {
            st.text_align = match lower.as_str() {
                "center" => TextAlign::Center,
                "right" | "end" => TextAlign::Right,
                _ => TextAlign::Left,
            }
        }
        "white-space" => st.pre = lower.starts_with("pre"),
        "line-height" => {
            st.line_height = if lower == "normal" {
                LineH::Normal
            } else if let Ok(f) = lower.parse::<f32>() {
                LineH::Factor(f)
            } else {
                match parse_len(value, fs) {
                    Some(Len::Px(p)) => LineH::Px(p),
                    Some(Len::Pct(p)) => LineH::Factor(p / 100.0),
                    _ => st.line_height,
                }
            }
        }
        "list-style" | "list-style-type" => st.list_none = lower.contains("none"),
        "box-sizing" => st.border_box = lower == "border-box",
        "width" => {
            if let Some(l) = parse_len(value, fs) {
                st.width = l;
            }
        }
        "height" => {
            if let Some(l) = parse_len(value, fs) {
                st.height = l;
            }
        }
        "max-width" => {
            if let Some(l) = parse_len(value, fs) {
                st.max_width = l;
            }
        }
        "min-height" => {
            if let Some(Len::Px(v)) = parse_len(value, fs) {
                st.min_height = v;
            }
        }
        "flex-direction" => st.flex_col = lower.starts_with("column"),
        "gap" | "column-gap" => {
            if let Some(Len::Px(v)) = split_ws_top(value).last().and_then(|t| parse_len(t, fs)) {
                st.gap = v;
            }
        }
        "margin" => {
            if let Some(v) = lens(value, fs).and_then(|v| box4(&v)) {
                st.margin = v;
            }
        }
        "padding" => {
            if let Some(v) = lens(value, fs).and_then(|v| box4(&v)) {
                st.padding = v;
            }
        }
        "margin-top" | "margin-right" | "margin-bottom" | "margin-left" | "padding-top" | "padding-right" | "padding-bottom" | "padding-left" => {
            if let Some(l) = parse_len(value, fs) {
                let idx = match name.rsplit('-').next().unwrap_or("") {
                    "top" => 0,
                    "right" => 1,
                    "bottom" => 2,
                    _ => 3,
                };
                if name.starts_with("margin") {
                    st.margin[idx] = l;
                } else {
                    st.padding[idx] = if l == Len::Auto { Len::Px(0.0) } else { l };
                }
            }
        }
        "border" | "border-top" | "border-right" | "border-bottom" | "border-left" => {
            let mut w: Option<f32> = None;
            let mut color: Option<Color> = None;
            let mut none = false;
            for t in split_ws_top(value) {
                let tl = t.to_ascii_lowercase();
                if tl == "none" || tl == "hidden" {
                    none = true;
                } else if matches!(tl.as_str(), "solid" | "dashed" | "dotted" | "double" | "groove" | "ridge" | "inset" | "outset") {
                    if w.is_none() {
                        w = Some(3.0);
                    }
                } else if let Some(x) = border_width(&tl, fs) {
                    w = Some(x);
                } else if let Some(c) = Color::parse(&tl) {
                    color = Some(c);
                }
            }
            let sides: Vec<usize> = match name {
                "border" => vec![0, 1, 2, 3],
                "border-top" => vec![0],
                "border-right" => vec![1],
                "border-bottom" => vec![2],
                _ => vec![3],
            };
            for s in sides {
                if none {
                    st.border[s].0 = 0.0;
                } else {
                    if let Some(w) = w {
                        st.border[s].0 = w;
                    }
                    st.border[s].1 = color.unwrap_or(st.color);
                }
            }
        }
        "border-width" => {
            let v: Option<Vec<f32>> = split_ws_top(value).iter().map(|t| border_width(&t.to_ascii_lowercase(), fs)).collect();
            if let Some(a) = v.and_then(|v| box4(&v)) {
                for i in 0..4 {
                    st.border[i].0 = a[i];
                }
            }
        }
        "border-color" => {
            let v: Option<Vec<Color>> = split_ws_top(value).iter().map(|t| Color::parse(t)).collect();
            if let Some(a) = v.and_then(|v| box4(&v)) {
                for i in 0..4 {
                    st.border[i].1 = a[i];
                }
            }
        }
        _ => {}
    }
}

pub fn compute_styles(doc: &Doc, sheets: &[(&Sheet, u8)], viewport_w: f32) -> Vec<Style> {
    let _ = viewport_w;
    let mut vars: HashMap<String, String> = HashMap::new();
    for (sh, _) in sheets {
        for r in &sh.rules {
            for d in &r.decls {
                if d.name.starts_with("--") {
                    vars.insert(d.name.clone(), d.value.clone());
                }
            }
        }
    }

    let mut offsets = Vec::new();
    let mut total = 0usize;
    for (sh, _) in sheets {
        offsets.push(total);
        total += sh.rules.len();
    }

    let mut out: Vec<Style> = Vec::with_capacity(doc.nodes.len());
    for id in 0..doc.nodes.len() {
        let n = &doc.nodes[id];
        match n.kind {
            Kind::Document => out.push(Style::root()),
            Kind::Text => {
                let p = n.parent.unwrap_or(0);
                let s = out[p].clone();
                out.push(s);
            }
            Kind::Element => {
                let parent_style = n.parent.map(|p| out[p].clone()).unwrap_or_else(Style::root);
                let mut st = parent_style.inherit();
                let parent_fs = parent_style.font_size;

                let mut matched: Vec<(bool, u8, u32, usize, &Decl)> = Vec::new();
                for (si, (sh, origin)) in sheets.iter().enumerate() {
                    for (ri, rule) in sh.rules.iter().enumerate() {
                        for sel in &rule.sels {
                            if matches(doc, id, sel) {
                                for d in &rule.decls {
                                    matched.push((d.important, *origin, sel.spec, offsets[si] + ri, d));
                                }
                            }
                        }
                    }
                }
                let inline_decls = doc.attr(id, "style").map(parse_decls).unwrap_or_default();
                for d in &inline_decls {
                    matched.push((d.important, 2, u32::MAX, usize::MAX, d));
                }
                matched.sort_by_key(|m| (m.0, m.1, m.2, m.3));

                for m in &matched {
                    if m.4.name == "font-size" {
                        let v = subst_vars(&m.4.value, &vars);
                        apply(&mut st, "font-size", &v, parent_fs);
                    }
                }
                for m in &matched {
                    if m.4.name != "font-size" && !m.4.name.starts_with("--") {
                        let v = subst_vars(&m.4.value, &vars);
                        apply(&mut st, &m.4.name, &v, parent_fs);
                    }
                }
                // presentational attributes
                if st.bg == Color::CLEAR {
                    if let Some(c) = doc.attr(id, "bgcolor").and_then(Color::parse) {
                        st.bg = c;
                    }
                }
                out.push(st);
            }
        }
    }
    out
}
