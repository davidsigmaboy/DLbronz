//! Block / inline layout producing a flat display list.

use crate::css::{Color, Display, Len, Style, TextAlign};
use crate::doc::{Doc, Kind};

#[derive(Clone, Debug)]
pub enum Item {
    Rect { x: f32, y: f32, w: f32, h: f32, color: Color },
    Text {
        x: f32,
        y: f32,
        w: f32,
        size: f32,
        bold: bool,
        italic: bool,
        underline: bool,
        mono: bool,
        color: Color,
        text: String,
        href: Option<String>,
    },
}

/// (text, font size, bold, italic, mono) -> width in px
pub type Measure<'a> = &'a dyn Fn(&str, f32, bool, bool, bool) -> f32;

enum Atom {
    Word { text: String, node: usize, w: f32 },
    Space { node: usize, w: f32 },
    Br { node: usize },
    Box { node: usize, w: f32, h: f32, alt: String },
}

pub struct Layouter<'a> {
    doc: &'a Doc,
    styles: &'a [Style],
    measure: Measure<'a>,
    has_block: Vec<bool>,
    pub items: Vec<Item>,
}

impl<'a> Layouter<'a> {
    pub fn new(doc: &'a Doc, styles: &'a [Style], measure: Measure<'a>) -> Self {
        let mut has_block = vec![false; doc.nodes.len()];
        for id in (0..doc.nodes.len()).rev() {
            if doc.nodes[id].kind != Kind::Element {
                continue;
            }
            has_block[id] = doc.nodes[id].children.iter().any(|&c| {
                doc.nodes[c].kind == Kind::Element
                    && styles[c].display != Display::None
                    && (styles[c].display.is_block_level() || has_block[c])
            });
        }
        Layouter { doc, styles, measure, has_block, items: Vec::new() }
    }

    fn tw(&self, text: &str, st: &Style) -> f32 {
        (self.measure)(text, st.font_size, st.bold, st.italic, st.mono)
    }

    fn resolve(len: Len, base: f32) -> Option<f32> {
        match len {
            Len::Px(v) => Some(v),
            Len::Pct(p) => Some(base * p / 100.0),
            Len::Auto => None,
        }
    }

    fn margins(&self, id: usize, aw: f32) -> [f32; 4] {
        let s = &self.styles[id];
        if s.display == Display::Inline {
            // vertical margins do not apply to inline boxes
            return [0.0, 0.0, 0.0, 0.0];
        }
        let mut m = [0.0; 4];
        for (i, v) in m.iter_mut().enumerate() {
            *v = Self::resolve(s.margin[i], aw).unwrap_or(0.0);
        }
        m
    }

    fn href_of(&self, mut id: usize) -> Option<String> {
        loop {
            if self.doc.is_element(id) && self.doc.tag(id) == "a" {
                if let Some(h) = self.doc.attr(id, "href") {
                    return Some(h.to_string());
                }
            }
            match self.doc.nodes[id].parent {
                Some(p) => id = p,
                None => return None,
            }
        }
    }

    fn is_inline_level(&self, id: usize) -> bool {
        let n = &self.doc.nodes[id];
        match n.kind {
            Kind::Text => true,
            Kind::Element => {
                let d = self.styles[id].display;
                d == Display::Inline && !self.has_block[id]
            }
            Kind::Document => false,
        }
    }

    fn visible_children(&self, id: usize) -> Vec<usize> {
        self.doc.nodes[id]
            .children
            .iter()
            .copied()
            .filter(|&c| match self.doc.nodes[c].kind {
                Kind::Text => true,
                Kind::Element => self.styles[c].display != Display::None,
                Kind::Document => false,
            })
            .collect()
    }

    // ───────────────────────── block layout ─────────────────────────

    /// Lays out the border box of `id` at (x, y) inside a containing block of
    /// width `aw`. Returns the border-box height.
    pub fn block(&mut self, id: usize, x: f32, y: f32, aw: f32) -> f32 {
        let s = self.styles[id].clone();
        let mut pad = [0.0f32; 4];
        for (i, p) in pad.iter_mut().enumerate() {
            *p = Self::resolve(s.padding[i], aw).unwrap_or(0.0).max(0.0);
        }
        let bw = [s.border[0].0, s.border[1].0, s.border[2].0, s.border[3].0];
        let extra_h = pad[1] + pad[3] + bw[1] + bw[3];
        let extra_v = pad[0] + pad[2] + bw[0] + bw[2];

        let mut ml = Self::resolve(s.margin[3], aw);
        let mut mr = Self::resolve(s.margin[1], aw);
        let wset = Self::resolve(s.width, aw);

        let mut content_w = match wset {
            Some(v) => {
                if s.border_box {
                    (v - extra_h).max(0.0)
                } else {
                    v
                }
            }
            None => (aw - ml.unwrap_or(0.0) - mr.unwrap_or(0.0) - extra_h).max(0.0),
        };
        if let Some(mx) = Self::resolve(s.max_width, aw) {
            let lim = if s.border_box { mx - extra_h } else { mx };
            if content_w > lim {
                content_w = lim.max(0.0);
            }
        }
        let outer = content_w + extra_h;
        let free = (aw - outer).max(0.0);
        match (ml, mr) {
            (None, None) => {
                ml = Some(free / 2.0);
                mr = Some(free / 2.0);
            }
            (None, Some(r)) => ml = Some((free - r).max(0.0)),
            _ => {}
        }
        let ml = ml.unwrap_or(0.0);
        let _ = mr;

        let bx = x + ml;
        let content_x = bx + bw[3] + pad[3];
        let cy = y + bw[0] + pad[0];

        let bg_idx = self.items.len();
        if s.bg.a > 0 {
            self.items.push(Item::Rect { x: bx, y, w: outer, h: 0.0, color: s.bg });
        }

        if s.display == Display::ListItem && !s.list_none {
            self.list_marker(id, content_x, cy, &s);
        }

        let tag = self.doc.tag(id).to_string();
        let kids = self.visible_children(id);
        let ch = if tag == "img" {
            let (iw, ih) = self.replaced_size(id, content_w);
            self.items.push(Item::Rect { x: content_x, y: cy, w: iw, h: ih, color: Color::rgb(0xe4, 0xe4, 0xe4) });
            ih
        } else if (s.display == Display::Flex && !s.flex_col) || s.display == Display::TableRow {
            self.row(&kids, content_x, cy, content_w, s.gap)
        } else {
            self.flow(id, &kids, content_x, cy, content_w, &s)
        };

        let mut h = extra_v + ch;
        if let Some(v) = Self::resolve(s.height, aw) {
            let hv = if s.border_box { v } else { v + extra_v };
            h = h.max(hv);
        }
        h = h.max(s.min_height);

        if s.bg.a > 0 {
            if let Item::Rect { h: rh, .. } = &mut self.items[bg_idx] {
                *rh = h;
            }
        }
        // borders
        let edges = [
            (0, bx, y, outer, bw[0]),
            (2, bx, y + h - bw[2], outer, bw[2]),
            (3, bx, y, bw[3], h),
            (1, bx + outer - bw[1], y, bw[1], h),
        ];
        for (side, ex, ey, ew, eh) in edges {
            let (w, c) = s.border[side];
            if w > 0.0 && c.a > 0 {
                self.items.push(Item::Rect { x: ex, y: ey, w: ew, h: eh, color: c });
            }
        }
        h
    }

    fn replaced_size(&self, id: usize, avail: f32) -> (f32, f32) {
        let s = &self.styles[id];
        let attr = |n: &str| self.doc.attr(id, n).and_then(|v| v.trim_end_matches("px").parse::<f32>().ok());
        let w = Self::resolve(s.width, avail).or_else(|| attr("width"));
        let h = Self::resolve(s.height, avail).or_else(|| attr("height"));
        match (w, h) {
            (Some(w), Some(h)) => (w.min(avail.max(1.0)), h),
            (Some(w), None) => (w.min(avail.max(1.0)), w * 0.6),
            (None, Some(h)) => (h * 1.5, h),
            (None, None) => (avail.min(150.0), 90.0),
        }
    }

    fn list_marker(&mut self, id: usize, content_x: f32, cy: f32, s: &Style) {
        let parent = self.doc.nodes[id].parent;
        let mut marker = "\u{2022}".to_string();
        if let Some(p) = parent {
            if self.doc.tag(p) == "ol" {
                let idx = self.doc.nodes[p]
                    .children
                    .iter()
                    .filter(|&&c| self.doc.is_element(c) && self.doc.tag(c) == "li")
                    .position(|&c| c == id)
                    .unwrap_or(0);
                marker = format!("{}.", idx + 1);
            }
        }
        let w = self.tw(&marker, s);
        let lh = s.line_px();
        self.items.push(Item::Text {
            x: content_x - w - 6.0,
            y: cy + (lh - s.font_size * 1.2) / 2.0,
            w,
            size: s.font_size,
            bold: s.bold,
            italic: false,
            underline: false,
            mono: s.mono,
            color: s.color,
            text: marker,
            href: None,
        });
    }

    /// Normal flow: block children stacked, inline runs wrapped into lines.
    fn flow(&mut self, parent: usize, kids: &[usize], cx: f32, cy: f32, cw: f32, ps: &Style) -> f32 {
        let mut cur = cy;
        let mut prev_mb = 0.0f32;
        let mut run: Vec<usize> = Vec::new();

        for &c in kids {
            if self.is_inline_level(c) {
                run.push(c);
                continue;
            }
            if !run.is_empty() {
                let h = self.inline(parent, &run, cx, cur + prev_mb, cw, ps.text_align);
                if h > 0.0 {
                    cur += prev_mb + h;
                    prev_mb = 0.0;
                }
                run.clear();
            }
            let m = self.margins(c, cw);
            let gap = prev_mb.max(m[0]);
            cur += gap;
            let h = self.block(c, cx, cur, cw);
            cur += h;
            prev_mb = m[2];
        }
        if !run.is_empty() {
            let h = self.inline(parent, &run, cx, cur + prev_mb, cw, ps.text_align);
            if h > 0.0 {
                cur += prev_mb + h;
                prev_mb = 0.0;
            }
        }
        cur += prev_mb;
        cur - cy
    }

    /// Side-by-side layout used for flex rows and table rows.
    fn row(&mut self, kids: &[usize], cx: f32, cy: f32, cw: f32, gap: f32) -> f32 {
        let cells: Vec<usize> = kids.iter().copied().filter(|&c| self.doc.is_element(c)).collect();
        if cells.is_empty() {
            return 0.0;
        }
        let max_cols = (((cw + gap) / (110.0 + gap)).floor() as usize).max(1);
        let mut total_h = 0.0;
        for chunk in cells.chunks(max_cols) {
            let n = chunk.len();
            let avail = (cw - gap * (n as f32 - 1.0)).max(0.0);
            let mut fixed = vec![None; n];
            let mut fixed_sum = 0.0;
            let mut auto = 0usize;
            for (i, &c) in chunk.iter().enumerate() {
                let s = &self.styles[c];
                let ml = Self::resolve(s.margin[3], cw).unwrap_or(0.0);
                let mr = Self::resolve(s.margin[1], cw).unwrap_or(0.0);
                if let Some(w) = Self::resolve(s.width, cw) {
                    let extra = if s.border_box {
                        0.0
                    } else {
                        Self::resolve(s.padding[1], cw).unwrap_or(0.0) + Self::resolve(s.padding[3], cw).unwrap_or(0.0)
                    };
                    let outer = w + extra + ml + mr;
                    fixed[i] = Some(outer);
                    fixed_sum += outer;
                } else {
                    auto += 1;
                }
            }
            let share = if auto > 0 { ((avail - fixed_sum) / auto as f32).max(0.0) } else { 0.0 };
            let mut x = cx;
            let mut row_h = 0.0f32;
            for (i, &c) in chunk.iter().enumerate() {
                let w = fixed[i].unwrap_or(share);
                let m = self.margins(c, w);
                let h = self.block(c, x, cy + total_h + m[0], w);
                row_h = row_h.max(h + m[0] + m[2]);
                x += w + gap;
            }
            total_h += row_h;
        }
        total_h
    }

    // ───────────────────────── inline layout ─────────────────────────

    fn gather(&self, id: usize, out: &mut Vec<Atom>) {
        let n = &self.doc.nodes[id];
        match n.kind {
            Kind::Text => {
                let pid = n.parent.unwrap_or(0);
                let st = &self.styles[pid];
                if st.pre {
                    let text = n.text.replace('\t', "    ");
                    let lines: Vec<&str> = text.split('\n').collect();
                    for (i, line) in lines.iter().enumerate() {
                        if !line.is_empty() {
                            let w = self.tw(line, st);
                            out.push(Atom::Word { text: line.to_string(), node: pid, w });
                        }
                        if i + 1 < lines.len() {
                            out.push(Atom::Br { node: pid });
                        }
                    }
                } else {
                    let t = &n.text;
                    let sp = self.tw(" ", st);
                    let starts_ws = t.chars().next().map(|c| c.is_whitespace()).unwrap_or(false);
                    let ends_ws = t.chars().last().map(|c| c.is_whitespace()).unwrap_or(false);
                    let words: Vec<&str> = t.split_whitespace().collect();
                    if words.is_empty() {
                        if !t.is_empty() {
                            out.push(Atom::Space { node: pid, w: sp });
                        }
                        return;
                    }
                    if starts_ws {
                        out.push(Atom::Space { node: pid, w: sp });
                    }
                    for (i, w) in words.iter().enumerate() {
                        if i > 0 {
                            out.push(Atom::Space { node: pid, w: sp });
                        }
                        let width = self.tw(w, st);
                        out.push(Atom::Word { text: w.to_string(), node: pid, w: width });
                    }
                    if ends_ws {
                        out.push(Atom::Space { node: pid, w: sp });
                    }
                }
            }
            Kind::Element => {
                if self.styles[id].display == Display::None {
                    return;
                }
                match n.tag.as_str() {
                    "br" => out.push(Atom::Br { node: id }),
                    "img" => {
                        let (w, h) = self.replaced_size(id, 10_000.0);
                        let alt = self.doc.attr(id, "alt").unwrap_or("").to_string();
                        out.push(Atom::Box { node: id, w, h, alt });
                    }
                    _ => {
                        for &c in &n.children {
                            self.gather(c, out);
                        }
                    }
                }
            }
            Kind::Document => {}
        }
    }

    fn inline(&mut self, parent: usize, run: &[usize], x: f32, y: f32, w: f32, align: TextAlign) -> f32 {
        let mut atoms: Vec<Atom> = Vec::new();
        for &c in run {
            self.gather(c, &mut atoms);
        }
        if atoms.is_empty() {
            return 0.0;
        }
        let nowrap = self.styles[parent].pre;

        // break into lines
        let mut lines: Vec<(Vec<Atom>, bool)> = Vec::new(); // (atoms, ended by <br>)
        let mut cur: Vec<Atom> = Vec::new();
        let mut cur_w = 0.0f32;
        let mut has_content = false;
        for atom in atoms {
            match atom {
                Atom::Br { .. } => {
                    cur.push(atom);
                    lines.push((std::mem::take(&mut cur), true));
                    cur_w = 0.0;
                    has_content = false;
                }
                Atom::Space { w: sw, .. } => {
                    if !cur.is_empty() && !matches!(cur.last(), Some(Atom::Space { .. })) {
                        cur_w += sw;
                        cur.push(atom);
                    }
                }
                Atom::Word { w: aw_, .. } | Atom::Box { w: aw_, .. } => {
                    if !nowrap && has_content && cur_w + aw_ > w + 0.5 {
                        while matches!(cur.last(), Some(Atom::Space { .. })) {
                            cur.pop();
                        }
                        lines.push((std::mem::take(&mut cur), false));
                        cur_w = 0.0;
                    }
                    cur_w += aw_;
                    cur.push(atom);
                    has_content = true;
                }
            }
        }
        if cur.iter().any(|a| !matches!(a, Atom::Space { .. })) {
            lines.push((cur, false));
        }
        if lines.is_empty() {
            return 0.0;
        }

        let mut total = 0.0;
        for (atoms, _) in lines {
            total += self.emit_line(&atoms, x, y + total, w, align, parent);
        }
        total
    }

    fn emit_line(&mut self, atoms: &[Atom], x: f32, top: f32, w: f32, align: TextAlign, parent: usize) -> f32 {
        // trim trailing spaces for width
        let mut end = atoms.len();
        while end > 0 && matches!(atoms[end - 1], Atom::Space { .. } | Atom::Br { .. }) {
            end -= 1;
        }
        let mut line_w = 0.0;
        let mut content_h = 0.0f32;
        let mut line_h = 0.0f32;
        for (i, a) in atoms.iter().enumerate() {
            match a {
                Atom::Word { node, w, .. } => {
                    let st = &self.styles[*node];
                    if i < end {
                        line_w += w;
                    }
                    content_h = content_h.max(st.font_size * 1.2);
                    line_h = line_h.max(st.line_px());
                }
                Atom::Space { node, w } => {
                    if i < end {
                        line_w += w;
                    }
                    let st = &self.styles[*node];
                    content_h = content_h.max(st.font_size * 1.2);
                    line_h = line_h.max(st.line_px());
                }
                Atom::Box { w, h, .. } => {
                    if i < end {
                        line_w += w;
                    }
                    content_h = content_h.max(*h);
                    line_h = line_h.max(*h);
                }
                Atom::Br { node } => {
                    let st = &self.styles[*node];
                    content_h = content_h.max(st.font_size * 1.2);
                    line_h = line_h.max(st.line_px());
                }
            }
        }
        if content_h == 0.0 {
            let st = &self.styles[parent];
            content_h = st.font_size * 1.2;
            line_h = st.line_px();
        }
        let line_h = line_h.max(content_h);
        let y_off = (line_h - content_h) / 2.0;

        let mut cx = x + match align {
            TextAlign::Left => 0.0,
            TextAlign::Center => ((w - line_w) / 2.0).max(0.0),
            TextAlign::Right => (w - line_w).max(0.0),
        };

        struct Run {
            node: usize,
            text: String,
            x: f32,
        }
        let mut run: Option<Run> = None;
        let mut pending_space: Option<f32> = None;

        // flush helper as closure-free inline function via macro
        macro_rules! flush {
            ($self:ident, $run:ident, $cx:ident) => {
                if let Some(r) = $run.take() {
                    let st = $self.styles[r.node].clone();
                    let width = ($cx - r.x).max(0.0);
                    let ty = top + y_off + (content_h - st.font_size * 1.2);
                    if st.display == Display::Inline && st.bg.a > 0 {
                        $self.items.push(Item::Rect { x: r.x, y: ty, w: width, h: st.font_size * 1.2, color: st.bg });
                    }
                    let href = $self.href_of(r.node);
                    $self.items.push(Item::Text {
                        x: r.x,
                        y: ty,
                        w: width,
                        size: st.font_size,
                        bold: st.bold,
                        italic: st.italic,
                        underline: st.underline,
                        mono: st.mono,
                        color: st.color,
                        text: r.text,
                        href,
                    });
                }
            };
        }

        for a in &atoms[..end] {
            match a {
                Atom::Space { w, .. } => pending_space = Some(*w),
                Atom::Word { text, node, w } => {
                    match &mut run {
                        Some(r) if r.node == *node => {
                            if let Some(sw) = pending_space.take() {
                                r.text.push(' ');
                                cx += sw;
                            }
                            r.text.push_str(text);
                            cx += w;
                        }
                        _ => {
                            flush!(self, run, cx);
                            if let Some(sw) = pending_space.take() {
                                cx += sw;
                            }
                            run = Some(Run { node: *node, text: text.clone(), x: cx });
                            cx += w;
                        }
                    }
                }
                Atom::Box { node, w, h, alt } => {
                    flush!(self, run, cx);
                    if let Some(sw) = pending_space.take() {
                        cx += sw;
                    }
                    let by = top + y_off + (content_h - h);
                    self.items.push(Item::Rect { x: cx, y: by, w: *w, h: *h, color: Color::rgb(0xe4, 0xe4, 0xe4) });
                    if !alt.is_empty() && *h >= 14.0 {
                        let size = 11.0;
                        let tw = (self.measure)(alt, size, false, false, false).min(*w);
                        self.items.push(Item::Text {
                            x: cx + 2.0,
                            y: by + 2.0,
                            w: tw,
                            size,
                            bold: false,
                            italic: false,
                            underline: false,
                            mono: false,
                            color: Color::rgb(0x77, 0x77, 0x77),
                            text: alt.clone(),
                            href: None,
                        });
                    }
                    let _ = node;
                    cx += w;
                }
                Atom::Br { .. } => {}
            }
        }
        flush!(self, run, cx);
        line_h
    }
}
