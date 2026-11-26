//! Runs page scripts against the JS DOM (same `dom.js` as the Apple build) on QuickJS.

use crate::fetch;
use rquickjs::{Context, Ctx, Function, Runtime, Value};
use std::cell::RefCell;
use std::rc::Rc;
use url::Url;

const DOM_JS: &str = include_str!("../js/dom.js");

pub struct Processed {
    pub html: String,
    pub sheets: Vec<Url>,
    pub logs: Vec<String>,
}

fn describe(ctx: &Ctx, e: rquickjs::Error) -> String {
    if matches!(e, rquickjs::Error::Exception) {
        let v = ctx.catch();
        if let Some(x) = v.as_exception() {
            return format!("{} {}", x.message().unwrap_or_default(), x.stack().unwrap_or_default());
        }
        return format!("{v:?}");
    }
    e.to_string()
}

fn call_str(ctx: &Ctx, name: &str) -> Option<String> {
    let f: Function = ctx.globals().get(name).ok()?;
    f.call::<_, String>(()).ok()
}

struct Out {
    html: String,
    sheets: Vec<String>,
}

fn run(ctx: &Ctx, dom: &str, base: &Url, html: &str, run_scripts: bool, logs: Rc<RefCell<Vec<String>>>) -> rquickjs::Result<Out> {
    let g = ctx.globals();
    let l = logs.clone();
    g.set(
        "__log",
        Function::new(ctx.clone(), move |level: String, msg: String| {
            let mut v = l.borrow_mut();
            if v.len() < 200 {
                v.push(format!("[{level}] {msg}"));
            }
        })?,
    )?;
    g.set(
        "__parseFragment",
        Function::new(ctx.clone(), |h: String| -> String {
            dlbronz_web::render::doc_to_json(&dlbronz_web::parse::parse_fragment(&h))
        })?,
    )?;
    g.set("__domJSON", dom)?;
    g.set("__baseURL", base.as_str())?;
    if let Err(e) = ctx.eval::<Value, _>(DOM_JS) {
        logs.borrow_mut().push(format!("[engine] dom.js init failed: {}", describe(ctx, e)));
        return Err(rquickjs::Error::Unknown);
    }

    let mut out_html = html.to_string();
    if run_scripts {
        let list = call_str(ctx, "__scripts").unwrap_or_else(|| "[]".into());
        let refs: Vec<serde_json::Value> = serde_json::from_str(&list).unwrap_or_default();
        for r in refs {
            let index = r["index"].as_u64().unwrap_or(0);
            let code: Option<String> = match r["src"].as_str() {
                Some(src) => base.join(src).ok().and_then(|u| fetch::get_optional(&u)),
                None => {
                    let f: Function = g.get("__scriptText")?;
                    f.call::<_, String>((index,)).ok()
                }
            };
            let Some(code) = code.filter(|c| !c.is_empty() && c.len() < 2_000_000) else { continue };
            let set: Function = g.get("__setCurrent")?;
            let _ = set.call::<_, Value>((index,));
            if let Err(e) = ctx.eval::<Value, _>(code) {
                let msg = describe(ctx, e);
                let mut v = logs.borrow_mut();
                if v.len() < 200 {
                    v.push(format!("[exception] script {index}: {msg}"));
                }
            }
        }
        let timers: Function = g.get("__runTimers")?;
        let _ = timers.call::<_, Value>((200,));
        let fire: Function = g.get("__fire")?;
        let _ = fire.call::<_, Value>(());
        let _ = timers.call::<_, Value>((200,));
        if let Some(s) = call_str(ctx, "__serialize").filter(|s| !s.is_empty()) {
            out_html = format!("<!DOCTYPE html>{s}");
        }
    }
    let sheets: Vec<String> = call_str(ctx, "__stylesheets")
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    Ok(Out { html: out_html, sheets })
}

/// Builds the DOM from `html`, optionally runs its scripts, and returns the resulting HTML.
pub fn process(html: &str, base: &Url, run_scripts: bool) -> Processed {
    let logs = Rc::new(RefCell::new(Vec::<String>::new()));
    let dom = dlbronz_web::render::doc_to_json(&dlbronz_web::parse::parse_document(html));

    let fallback = |msg: String, logs: &Rc<RefCell<Vec<String>>>| {
        logs.borrow_mut().push(msg);
        Processed { html: html.to_string(), sheets: vec![], logs: logs.borrow().clone() }
    };

    let Ok(rt) = Runtime::new() else { return fallback("[engine] QuickJS runtime failed".into(), &logs) };
    let Ok(ctx) = Context::full(&rt) else { return fallback("[engine] QuickJS context failed".into(), &logs) };

    let result = ctx.with(|ctx| run(&ctx, &dom, base, html, run_scripts, logs.clone()).map_err(|e| e.to_string()));
    match result {
        Ok(out) => Processed {
            html: out.html,
            sheets: out.sheets.iter().filter_map(|s| base.join(s).ok()).collect(),
            logs: logs.borrow().clone(),
        },
        Err(e) => fallback(format!("[engine] {e}"), &logs),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Url {
        Url::parse("https://example.com/").unwrap()
    }

    #[test]
    fn scripts_mutate_dom() {
        let html = "<html><body><div id='a'>old</div><script>document.getElementById('a').textContent='new'; console.log('ran');</script></body></html>";
        let p = process(html, &base(), true);
        assert!(p.html.contains(">new</div>"), "{}", p.html);
        assert!(p.logs.iter().any(|l| l == "[log] ran"), "{:?}", p.logs);
    }

    #[test]
    fn inner_html_uses_html5ever() {
        let html = "<html><body><div id='a'></div><script>document.getElementById('a').innerHTML='<p>x<p>y';</script></body></html>";
        let p = process(html, &base(), true);
        assert!(p.html.contains("<p>x</p><p>y</p>"), "{}", p.html);
    }

    #[test]
    fn scripts_can_be_disabled() {
        let html = "<html><body><div id='a'>old</div><script>document.getElementById('a').textContent='new'</script></body></html>";
        let p = process(html, &base(), false);
        assert!(p.html.contains("old"));
    }

    #[test]
    fn script_errors_are_logged_not_fatal() {
        let html = "<html><body><div id='a'>ok</div><script>nope.nope()</script><script>document.getElementById('a').textContent='after'</script></body></html>";
        let p = process(html, &base(), true);
        assert!(p.logs.iter().any(|l| l.contains("[exception]")), "{:?}", p.logs);
        assert!(p.html.contains(">after</div>"), "{}", p.html);
    }

    #[test]
    fn stylesheets_are_listed() {
        let html = "<html><head><link rel='stylesheet' href='/s.css'></head><body></body></html>";
        let p = process(html, &base(), false);
        assert_eq!(p.sheets.iter().map(|u| u.as_str()).collect::<Vec<_>>(), vec!["https://example.com/s.css"]);
    }
}
