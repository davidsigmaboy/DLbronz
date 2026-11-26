//! Loading pages and resources over HTTP(S) or from local files.

use std::time::Duration;
use url::Url;

const UA: &str = "Mozilla/5.0 (X11; Linux x86_64) DLbronz/0.2";

/// Turns user input (URL, host name or file path) into a URL.
pub fn normalize(input: &str) -> Result<Url, String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty address".into());
    }
    if let Ok(path) = std::fs::canonicalize(s) {
        return Url::from_file_path(&path).map_err(|_| format!("bad path: {s}"));
    }
    if s.contains("://") {
        return Url::parse(s).map_err(|e| format!("invalid URL: {e}"));
    }
    Url::parse(&format!("https://{s}")).map_err(|e| format!("invalid URL: {e}"))
}

/// Returns the body text and the final URL (after redirects).
pub fn get_text(url: &Url) -> Result<(String, Url), String> {
    if url.scheme() == "file" {
        let path = url.to_file_path().map_err(|_| "bad file URL".to_string())?;
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        return Ok((text, url.clone()));
    }
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(20))
        .user_agent(UA)
        .build();
    match agent.get(url.as_str()).call() {
        Ok(resp) => {
            let final_url = Url::parse(resp.get_url()).unwrap_or_else(|_| url.clone());
            let text = resp.into_string().map_err(|e| format!("read error: {e}"))?;
            Ok((text, final_url))
        }
        Err(ureq::Error::Status(code, _)) => Err(format!("HTTP status: {code}")),
        Err(e) => Err(format!("load error: {e}")),
    }
}

/// Like `get_text` but returns `None` on any failure (for sub-resources).
pub fn get_optional(url: &Url) -> Option<String> {
    get_text(url).ok().map(|(t, _)| t)
}
