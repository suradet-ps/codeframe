//! High-resolution PNG export: build a fresh off-screen canvas at the full
//! export scale, draw once, then `toBlob("image/png")` → object URL →
//! download (AGENTS.md §4, rules 3 and 5).

use codeshot_highlighter::{highlight, theme_palette};
use codeshot_renderer::render_to_canvas;
use js_sys::{Function, Promise};
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{Blob, HtmlAnchorElement, HtmlCanvasElement, Url};

use crate::fonts::ensure_fonts_ready;
use crate::state::Settings;

/// Kick off an export with the current settings. `set_exporting` /
/// `set_error` report progress back to the header.
pub fn export_png(
  settings: Settings,
  set_exporting: WriteSignal<bool>,
  set_error: WriteSignal<Option<String>>,
) {
  set_exporting.set(true);
  spawn_local(async move {
    let result = do_export(settings).await;
    set_exporting.set(false);
    set_error.set(result.err());
  });
}

async fn do_export(settings: Settings) -> Result<(), String> {
  let code = settings.code.get_untracked();
  let language = settings.language.get_untracked();
  let theme = settings.theme.get_untracked();
  let options = settings.export_options();

  ensure_fonts_ready(options.font_family.css_family()).await;

  let tokens = highlight(&code, language, theme).map_err(|e| e.to_string())?;
  let palette = theme_palette(theme).map_err(|e| e.to_string())?;

  let window = web_sys::window().ok_or_else(|| "no window object".to_string())?;
  let document = window
    .document()
    .ok_or_else(|| "no document object".to_string())?;
  // A brand-new canvas at full scale - never the preview element.
  let canvas: HtmlCanvasElement = document
    .create_element("canvas")
    .map_err(|e| format!("{e:?}"))?
    .unchecked_into();

  render_to_canvas(&canvas, &tokens, &palette, &options).map_err(|e| e.to_string())?;

  let blob = canvas_to_blob(&canvas).await?;
  let url = Url::create_object_url_with_blob(&blob).map_err(|e| format!("{e:?}"))?;
  let anchor: HtmlAnchorElement = document
    .create_element("a")
    .map_err(|e| format!("{e:?}"))?
    .unchecked_into();
  anchor.set_href(&url);
  anchor.set_download(&format!("codeshot-{}x.png", options.scale));
  anchor.click();
  let _ = Url::revoke_object_url(&url);
  Ok(())
}

/// Wrap `canvas.toBlob` (callback-based) in a future.
async fn canvas_to_blob(canvas: &HtmlCanvasElement) -> Result<Blob, String> {
  let promise = Promise::new(&mut |resolve: Function, _reject: Function| {
    let resolve_for_callback = resolve.clone();
    let callback = Closure::once(move |blob: Option<Blob>| {
      let value: JsValue = match blob {
        Some(blob) => blob.into(),
        None => JsValue::NULL,
      };
      let _ = resolve_for_callback.call1(&JsValue::UNDEFINED, &value);
    });
    if canvas
      .to_blob_with_type(callback.as_ref().unchecked_ref(), "image/png")
      .is_err()
    {
      // Settle the promise even if the call itself threw.
      let _ = resolve.call1(&JsValue::UNDEFINED, &JsValue::NULL);
    }
    // The browser holds the callback until it fires; leak is one-shot.
    callback.forget();
  });
  let value = JsFuture::from(promise)
    .await
    .map_err(|e| format!("{e:?}"))?;
  value
    .dyn_into::<Blob>()
    .map_err(|_| "canvas.toBlob returned no data".to_string())
}
