//! Fonts: loading, measuring and glyph rasterization (fontdue).

use fontdue::{Font, FontSettings, Metrics};
use std::collections::HashMap;

const REGULAR: &[&str] = &[
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
    "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
    "/usr/share/fonts/noto/NotoSans-Regular.ttf",
];
const BOLD: &[&str] = &[
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    "/usr/share/fonts/dejavu/DejaVuSans-Bold.ttf",
    "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
    "/usr/share/fonts/liberation/LiberationSans-Bold.ttf",
    "/usr/share/fonts/truetype/noto/NotoSans-Bold.ttf",
];
const MONO: &[&str] = &[
    "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
    "/usr/share/fonts/dejavu/DejaVuSansMono.ttf",
    "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
    "/usr/share/fonts/liberation/LiberationMono-Regular.ttf",
];
const MONO_BOLD: &[&str] = &[
    "/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf",
    "/usr/share/fonts/dejavu/DejaVuSansMono-Bold.ttf",
    "/usr/share/fonts/TTF/DejaVuSansMono-Bold.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationMono-Bold.ttf",
];

fn load_first(paths: &[&str]) -> Option<Font> {
    for p in paths {
        if let Ok(bytes) = std::fs::read(p) {
            if let Ok(f) = Font::from_bytes(bytes, FontSettings::default()) {
                return Some(f);
            }
        }
    }
    None
}

pub struct Fonts {
    regular: Font,
    bold: Font,
    mono: Font,
    mono_bold: Font,
    cache: HashMap<(usize, char, u32), (Metrics, Vec<u8>)>,
}

impl Fonts {
    /// Looks for DejaVu / Liberation / Noto fonts. `DLBRONZ_FONT` overrides the regular face.
    pub fn load() -> Result<Fonts, String> {
        let custom = std::env::var("DLBRONZ_FONT").ok().and_then(|p| {
            let bytes = std::fs::read(p).ok()?;
            Font::from_bytes(bytes, FontSettings::default()).ok()
        });
        let regular = custom
            .or_else(|| load_first(REGULAR))
            .ok_or("No font found. Install fonts-dejavu-core (apt) or set DLBRONZ_FONT=/path/to/font.ttf")?;
        let bold = load_first(BOLD).unwrap_or_else(|| clone_font(&regular));
        let mono = load_first(MONO).unwrap_or_else(|| clone_font(&regular));
        let mono_bold = load_first(MONO_BOLD).unwrap_or_else(|| clone_font(&mono));
        Ok(Fonts { regular, bold, mono, mono_bold, cache: HashMap::new() })
    }

    fn index(bold: bool, mono: bool) -> usize {
        (bold as usize) | ((mono as usize) << 1)
    }

    fn face(&self, idx: usize) -> &Font {
        match idx {
            0 => &self.regular,
            1 => &self.bold,
            2 => &self.mono,
            _ => &self.mono_bold,
        }
    }

    /// Width of a text run in px.
    pub fn measure(&self, text: &str, size: f32, bold: bool, mono: bool) -> f32 {
        let f = self.face(Self::index(bold, mono));
        text.chars().map(|c| f.metrics(c, size).advance_width).sum()
    }

    /// (ascent, descent) in px; descent is negative.
    pub fn line(&self, size: f32, bold: bool, mono: bool) -> (f32, f32) {
        self.face(Self::index(bold, mono))
            .horizontal_line_metrics(size)
            .map(|m| (m.ascent, m.descent))
            .unwrap_or((size * 0.9, -size * 0.25))
    }

    /// Rasterized glyph (coverage bitmap), cached.
    pub fn glyph(&mut self, c: char, size: f32, bold: bool, mono: bool) -> &(Metrics, Vec<u8>) {
        let idx = Self::index(bold, mono);
        let key = (idx, c, size.to_bits());
        if !self.cache.contains_key(&key) {
            let g = self.face(idx).rasterize(c, size);
            if self.cache.len() > 20_000 {
                self.cache.clear();
            }
            self.cache.insert(key, g);
        }
        &self.cache[&key]
    }
}

fn clone_font(f: &Font) -> Font {
    // Font has no Clone; rebuild from the same data is not possible, so reuse by re-reading.
    // Callers only need a usable face, so load the first available regular font again.
    let _ = f;
    load_first(REGULAR).expect("regular font vanished")
}
