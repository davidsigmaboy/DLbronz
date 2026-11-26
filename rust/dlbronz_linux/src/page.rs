//! Page loading pipeline: fetch -> scripts -> stylesheets -> layout.

use crate::{fetch, js, text::Fonts};
use dlbronz_web::render::{render, Page};
use url::Url;

pub struct Loaded {
    pub url: Url,
    pub html: String,
    pub css: String,
    pub logs: Vec<String>,
}

pub fn load(input: &str, run_scripts: bool) -> Result<Loaded, String> {
    let url = fetch::normalize(input)?;
    let (html, final_url) = fetch::get_text(&url)?;
    let processed = js::process(&html, &final_url, run_scripts);
    let css = processed
        .sheets
        .iter()
        .take(40)
        .filter_map(fetch::get_optional)
        .collect::<Vec<_>>()
        .join("\n");
    Ok(Loaded { url: final_url, html: processed.html, css, logs: processed.logs })
}

pub fn layout(l: &Loaded, width: f32, fonts: &Fonts) -> Page {
    let doc = dlbronz_web::parse::parse_document(&l.html);
    render(&doc, &l.css, width.max(50.0), &|t, size, bold, _italic, mono| fonts.measure(t, size, bold, mono))
}
