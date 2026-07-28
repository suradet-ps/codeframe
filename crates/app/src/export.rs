//! High-resolution PNG export: build a fresh off-screen canvas at the full
//! export scale, draw once, then `toBlob("image/png")` → object URL →
//! download (AGENTS.md §4, rules 3 and 5).

use codeframe_highlighter::{highlight, theme_palette};
use codeframe_renderer::render_to_canvas;
use js_sys::{Function, Promise};
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{Blob, ClipboardItem, HtmlAnchorElement, HtmlCanvasElement, Url};

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

/// Kick off a clipboard copy with the current settings.
pub fn copy_to_clipboard(
  settings: Settings,
  set_copied: WriteSignal<bool>,
  set_error: WriteSignal<Option<String>>,
) {
  set_copied.set(false);
  spawn_local(async move {
    let result = do_copy(settings).await;
    if result.is_ok() {
      set_copied.set(true);
      // Reset after 2 seconds.
      let handle = gloo_timers::future::TimeoutFuture::new(2000);
      handle.await;
      set_copied.set(false);
    } else {
      set_error.set(result.err());
    }
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
  anchor.set_download(&format!("codeframe-{}x.png", options.scale));
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

/// Render to an off-screen canvas and copy the PNG blob to the clipboard.
async fn do_copy(settings: Settings) -> Result<(), String> {
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
  let canvas: HtmlCanvasElement = document
    .create_element("canvas")
    .map_err(|e| format!("{e:?}"))?
    .unchecked_into();

  render_to_canvas(&canvas, &tokens, &palette, &options).map_err(|e| e.to_string())?;

  let blob = canvas_to_blob(&canvas).await?;

  // Build a Record<string, Blob> for ClipboardItem.
  let record = js_sys::Object::new();
  js_sys::Reflect::set(&record, &"image/png".into(), &blob)
    .map_err(|e| format!("{e:?}"))?;

  let item = ClipboardItem::new_with_record_from_str_to_blob_promise(&record.into())
    .map_err(|e| format!("{e:?}"))?;
  let item_array = js_sys::Array::new();
  item_array.push(&item);

  let navigator = window.navigator();
  let clipboard = navigator.clipboard();
  let promise = clipboard.write(&item_array.into());
  JsFuture::from(promise)
    .await
    .map_err(|e| format!("{e:?}"))?;
  Ok(())
}
