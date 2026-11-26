//! Window, address bar, scrolling, links and history (minifb).

use crate::page::{self, Loaded};
use crate::paint::{self, Surface};
use crate::text::Fonts;
use dlbronz_web::css::Color;
use dlbronz_web::layout::Item;
use dlbronz_web::render::Page;
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};

const BAR_H: f32 = 30.0;

struct Browser {
    loaded: Option<Loaded>,
    page: Option<Page>,
    scroll: f32,
    history: Vec<String>,
    addr: String,
    editing: bool,
    status: String,
    scripts: bool,
    width: usize,
}

impl Browser {
    fn navigate(&mut self, input: &str, push: bool, fonts: &Fonts) {
        self.status = format!("Loading {input} ...");
        match page::load(input, self.scripts) {
            Ok(l) => {
                self.addr = l.url.to_string();
                if push {
                    self.history.push(self.addr.clone());
                }
                self.page = Some(page::layout(&l, self.width as f32, fonts));
                self.status = l.logs.last().cloned().unwrap_or_default();
                self.loaded = Some(l);
                self.scroll = 0.0;
            }
            Err(e) => self.status = e,
        }
    }

    fn relayout(&mut self, fonts: &Fonts) {
        if let Some(l) = &self.loaded {
            self.page = Some(page::layout(l, self.width as f32, fonts));
        }
    }

    fn back(&mut self, fonts: &Fonts) {
        if self.history.len() > 1 {
            self.history.pop();
            if let Some(prev) = self.history.last().cloned() {
                self.navigate(&prev, false, fonts);
            }
        }
    }

    fn link_at(&self, x: f32, y: f32) -> Option<String> {
        let page = self.page.as_ref()?;
        let base = &self.loaded.as_ref()?.url;
        let py = y - BAR_H + self.scroll;
        for item in page.items.iter().rev() {
            if let Item::Text { x: ix, y: iy, w, size, href: Some(h), .. } = item {
                if x >= *ix - 2.0 && x <= ix + w + 2.0 && py >= *iy - 3.0 && py <= iy + size * 1.2 + 3.0 {
                    if h.starts_with('#') || h.to_ascii_lowercase().starts_with("javascript:") {
                        return None;
                    }
                    return base.join(h).ok().map(|u| u.to_string());
                }
            }
        }
        None
    }

    fn max_scroll(&self, win_h: usize) -> f32 {
        let ph = self.page.as_ref().map(|p| p.height).unwrap_or(0.0);
        (ph + BAR_H - win_h as f32).max(0.0)
    }
}

fn key_char(k: Key, shift: bool) -> Option<char> {
    use Key::*;
    let c = match k {
        A => 'a', B => 'b', C => 'c', D => 'd', E => 'e', F => 'f', G => 'g', H => 'h', I => 'i',
        J => 'j', K => 'k', L => 'l', M => 'm', N => 'n', O => 'o', P => 'p', Q => 'q', R => 'r',
        S => 's', T => 't', U => 'u', V => 'v', W => 'w', X => 'x', Y => 'y', Z => 'z',
        Key0 => '0', Key1 => '1', Key2 => '2', Key3 => '3', Key4 => '4',
        Key5 => '5', Key6 => '6', Key7 => '7', Key8 => '8', Key9 => '9',
        Period => '.', Slash => '/', Minus => if shift { '_' } else { '-' },
        Semicolon => if shift { ':' } else { ';' },
        Space => ' ',
        _ => return None,
    };
    Some(c)
}

fn draw_bar(s: &mut Surface, fonts: &mut Fonts, b: &Browser) {
    let w = s.w as f32;
    s.fill_rect(0.0, 0.0, w, BAR_H, Color::rgb(0xee, 0xee, 0xee));
    s.fill_rect(0.0, BAR_H - 1.0, w, 1.0, Color::rgb(0xbb, 0xbb, 0xbb));
    let field = if b.editing { Color::rgb(0xff, 0xff, 0xdd) } else { Color::rgb(0xff, 0xff, 0xff) };
    s.fill_rect(6.0, 4.0, w - 12.0, BAR_H - 9.0, field);
    let shown = if b.editing { format!("{}|", b.addr) } else if b.status.is_empty() || b.status.starts_with('[') { b.addr.clone() } else { b.status.clone() };
    let shown = if shown.is_empty() { "Ctrl+L: address, Backspace: back, wheel/arrows: scroll".to_string() } else { shown };
    let item = Item::Text {
        x: 12.0,
        y: 6.0,
        w: 0.0,
        size: 14.0,
        bold: false,
        italic: false,
        underline: false,
        mono: false,
        color: Color::rgb(0x22, 0x22, 0x22),
        text: shown,
        href: None,
    };
    let page = Page { height: BAR_H, bg: None, title: String::new(), items: vec![item] };
    paint::paint(s, &page, fonts, 0.0);
}

pub fn run(start: Option<String>, mut fonts: Fonts, scripts: bool, width: usize, height: usize) -> Result<(), String> {
    let mut win = Window::new("DLbronz", width, height, WindowOptions { resize: true, ..WindowOptions::default() })
        .map_err(|e| format!("cannot open window: {e}"))?;
    win.set_target_fps(60);

    let mut b = Browser {
        loaded: None,
        page: None,
        scroll: 0.0,
        history: vec![],
        addr: String::new(),
        editing: false,
        status: String::new(),
        scripts,
        width,
    };
    if let Some(s) = start {
        b.navigate(&s, true, &fonts);
    } else {
        b.editing = true;
    }

    let (mut w, mut h) = (width, height);
    let mut was_down = false;
    let mut last_title = String::new();

    while win.is_open() && !win.is_key_down(Key::Escape) {
        let (nw, nh) = win.get_size();
        if (nw, nh) != (w, h) && nw > 0 && nh > 0 {
            w = nw;
            h = nh;
            if b.width != w {
                b.width = w;
                b.relayout(&fonts);
            }
        }

        let ctrl = win.is_key_down(Key::LeftCtrl) || win.is_key_down(Key::RightCtrl);
        let shift = win.is_key_down(Key::LeftShift) || win.is_key_down(Key::RightShift);
        for k in win.get_keys_pressed(KeyRepeat::Yes) {
            if b.editing {
                match k {
                    Key::Enter => {
                        b.editing = false;
                        let target = b.addr.clone();
                        b.navigate(&target, true, &fonts);
                    }
                    Key::Backspace => {
                        b.addr.pop();
                    }
                    Key::Tab => {}
                    k => {
                        if let Some(c) = key_char(k, shift) {
                            b.addr.push(c);
                        }
                    }
                }
            } else {
                match k {
                    Key::L if ctrl => {
                        b.editing = true;
                        b.addr.clear();
                    }
                    Key::Backspace => b.back(&fonts),
                    Key::Down => b.scroll += 40.0,
                    Key::Up => b.scroll -= 40.0,
                    Key::PageDown | Key::Space => b.scroll += h as f32 * 0.9,
                    Key::PageUp => b.scroll -= h as f32 * 0.9,
                    Key::Home => b.scroll = 0.0,
                    Key::End => b.scroll = f32::MAX,
                    _ => {}
                }
            }
        }
        if let Some((_, dy)) = win.get_scroll_wheel() {
            b.scroll -= dy * 40.0;
        }
        let max = b.max_scroll(h);
        b.scroll = b.scroll.clamp(0.0, max);

        let down = win.get_mouse_down(MouseButton::Left);
        if down && !was_down {
            if let Some((mx, my)) = win.get_mouse_pos(MouseMode::Discard) {
                if my < BAR_H {
                    b.editing = true;
                    b.addr.clear();
                } else if let Some(target) = b.link_at(mx, my) {
                    b.navigate(&target, true, &fonts);
                }
            }
        }
        was_down = down;

        let bg = b.page.as_ref().and_then(|p| p.bg).unwrap_or(Color::rgb(255, 255, 255));
        let mut surface = Surface::new(w, h, paint::pack(bg));
        if let Some(p) = &b.page {
            paint::paint(&mut surface, p, &mut fonts, BAR_H - b.scroll);
        }
        draw_bar(&mut surface, &mut fonts, &b);

        let title = b.page.as_ref().map(|p| p.title.clone()).filter(|t| !t.is_empty()).unwrap_or_else(|| "DLbronz".into());
        if title != last_title {
            win.set_title(&format!("{title} - DLbronz"));
            last_title = title;
        }
        win.update_with_buffer(&surface.buf, w, h).map_err(|e| e.to_string())?;
    }
    Ok(())
}
