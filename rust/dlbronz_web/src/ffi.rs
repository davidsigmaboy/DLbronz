//! C ABI used by Swift.

use crate::{parse, render};
use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

pub type MeasureFn = extern "C" fn(*const c_char, f32, i32, i32, i32) -> f32;

fn to_c(s: String) -> *mut c_char {
    CString::new(s.replace('\0', ""))
        .map(CString::into_raw)
        .unwrap_or(ptr::null_mut())
}

unsafe fn from_c(p: *const c_char) -> String {
    if p.is_null() {
        return String::new();
    }
    CStr::from_ptr(p).to_string_lossy().into_owned()
}

/// Free a string returned by any `dlbronz_web_*` function.
#[no_mangle]
pub extern "C" fn dlbronz_web_free(p: *mut c_char) {
    if !p.is_null() {
        unsafe { drop(CString::from_raw(p)) };
    }
}

/// Parse a full HTML document; returns the DOM as JSON (see `render::doc_to_json`).
#[no_mangle]
pub extern "C" fn dlbronz_web_parse_json(html: *const c_char) -> *mut c_char {
    let html = unsafe { from_c(html) };
    catch_unwind(AssertUnwindSafe(|| to_c(render::doc_to_json(&parse::parse_document(&html)))))
        .unwrap_or(ptr::null_mut())
}

/// Parse an HTML fragment; returns the DOM as JSON, children under a `{"t":"d"}` root.
#[no_mangle]
pub extern "C" fn dlbronz_web_parse_fragment_json(html: *const c_char) -> *mut c_char {
    let html = unsafe { from_c(html) };
    catch_unwind(AssertUnwindSafe(|| to_c(render::doc_to_json(&parse::parse_fragment(&html)))))
        .unwrap_or(ptr::null_mut())
}

/// Parse + style + lay out. `measure` returns the width of a text run in px.
#[no_mangle]
pub extern "C" fn dlbronz_web_render(html: *const c_char, css: *const c_char, width: f32, measure: MeasureFn) -> *mut c_char {
    let html = unsafe { from_c(html) };
    let css = unsafe { from_c(css) };
    catch_unwind(AssertUnwindSafe(|| {
        let doc = parse::parse_document(&html);
        let m = |text: &str, size: f32, bold: bool, italic: bool, mono: bool| -> f32 {
            let c = CString::new(text.replace('\0', "")).unwrap_or_default();
            measure(c.as_ptr(), size, bold as i32, italic as i32, mono as i32)
        };
        let page = render::render(&doc, &css, width.max(50.0), &m);
        to_c(render::page_to_json(&page))
    }))
    .unwrap_or(ptr::null_mut())
}
