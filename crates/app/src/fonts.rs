//! Web Font Loading API guard — never draw before the real font is available
//! (AGENTS.md §4, rule 4). Awaited before *every* draw, preview and export.

use wasm_bindgen_futures::JsFuture;
use web_sys::window;

/// Trigger loads for every style variant of `family` the renderer may use,
/// then wait for the document's font set to settle.
pub async fn ensure_fonts_ready(family: &str) {
  let Some(document) = window().and_then(|w| w.document()) else {
    return;
  };
  let fonts = document.fonts();
  for spec in [
    format!("400 14px \"{family}\""),
    format!("700 14px \"{family}\""),
    format!("italic 400 14px \"{family}\""),
    format!("italic 700 14px \"{family}\""),
  ] {
    // A rejected future just means that variant doesn't exist (e.g.
    // Fira Code has no italic) — canvas will synthesize it.
    let _ = JsFuture::from(fonts.load(&spec)).await;
  }
  if let Ok(ready) = fonts.ready() {
    let _ = JsFuture::from(ready).await;
  }
}
