//! DLbronz for Linux: the DLbronz web core (html5ever + CSS + layout) with a window,
//! QuickJS-backed JS DOM, and a headless PNG mode.

mod fetch;
mod gui;
mod js;
mod page;
mod paint;
mod text;

use std::io::BufWriter;
use std::process::exit;

const USAGE: &str = "\
DLbronz for Linux

usage: dlbronz-linux [URL|FILE] [options]

  --png OUT.png   render without a window and write a screenshot
  --width N       viewport width (default 1024)
  --height N      window height (default 768)
  --no-js         do not run page scripts
  -h, --help      show this help

In the window: Ctrl+L address bar, click links, Backspace back,
mouse wheel / arrows / PageUp / PageDown scroll, Esc quit.";

fn write_png(path: &str, surface: &paint::Surface) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| format!("{path}: {e}"))?;
    let mut enc = png::Encoder::new(BufWriter::new(file), surface.w as u32, surface.h as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().map_err(|e| e.to_string())?;
    let mut bytes = Vec::with_capacity(surface.w * surface.h * 3);
    for px in &surface.buf {
        bytes.extend_from_slice(&[(px >> 16) as u8, (px >> 8) as u8, *px as u8]);
    }
    writer.write_image_data(&bytes).map_err(|e| e.to_string())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut target: Option<String> = None;
    let mut png_out: Option<String> = None;
    let mut width = 1024usize;
    let mut height = 768usize;
    let mut scripts = true;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--png" => {
                i += 1;
                png_out = args.get(i).cloned();
            }
            "--width" => {
                i += 1;
                width = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(width);
            }
            "--height" => {
                i += 1;
                height = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(height);
            }
            "--no-js" => scripts = false,
            "-h" | "--help" => {
                println!("{USAGE}");
                return;
            }
            other => target = Some(other.to_string()),
        }
        i += 1;
    }

    let mut fonts = match text::Fonts::load() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{e}");
            exit(2);
        }
    };

    if let Some(out) = png_out {
        let Some(t) = target else {
            eprintln!("--png needs a URL or file\n\n{USAGE}");
            exit(2);
        };
        let loaded = match page::load(&t, scripts) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("{e}");
                exit(1);
            }
        };
        let laid = page::layout(&loaded, width as f32, &fonts);
        let h = (laid.height.ceil() as usize + 8).clamp(64, 20_000);
        let bg = laid.bg.unwrap_or(dlbronz_web::css::Color::rgb(255, 255, 255));
        let mut surface = paint::Surface::new(width, h, paint::pack(bg));
        paint::paint(&mut surface, &laid, &mut fonts, 0.0);
        if let Err(e) = write_png(&out, &surface) {
            eprintln!("{e}");
            exit(1);
        }
        println!("title: {}", laid.title);
        println!("size: {}x{}", width, h);
        println!("items: {}", laid.items.len());
        for l in loaded.logs.iter().take(20) {
            println!("js: {l}");
        }
        println!("wrote {out}");
        return;
    }

    if let Err(e) = gui::run(target, fonts, scripts, width, height) {
        eprintln!("{e}");
        exit(1);
    }
}
