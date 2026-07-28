//! High-resolution export: build a fresh off-screen canvas at the full
//! export scale, draw once, then `toBlob("image/png")` → object URL →
//! download (AGENTS.md §4, rules 3 and 5). Also supports SVG export via
//! pure string generation.

use codeframe_highlighter::{highlight, theme_palette};
use codeframe_renderer::render_to_canvas;
use codeframe_renderer::svg::render_svg;
use js_sys::{Function, Promise};
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{Blob, ClipboardItem, HtmlAnchorElement, HtmlCanvasElement, Url};

use crate::fonts::ensure_fonts_ready;
use crate::state::Settings;

/// Kick off a PNG export with the current settings.
pub fn export_png(
  settings: Settings,
  set_exporting: WriteSignal<bool>,
  set_error: WriteSignal<Option<String>>,
) {
  set_exporting.set(true);
  spawn_local(async move {
    let result = do_export_png(settings).await;
    set_exporting.set(false);
    set_error.set(result.err());
  });
}

/// Kick off an SVG export with the current settings.
pub fn export_svg(
  settings: Settings,
  set_exporting: WriteSignal<bool>,
  set_error: WriteSignal<Option<String>>,
) {
  set_exporting.set(true);
  spawn_local(async move {
    let result = do_export_svg(settings).await;
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

async fn do_export_png(settings: Settings) -> Result<(), String> {
  let code = settings.code.get_untracked();
  let language = settings.language.get_untracked();
  let theme = settings.theme.get_untracked();
  let mut options = settings.export_options();

  ensure_fonts_ready(options.font_family.css_family()).await;

  let tokens = highlight(&code, language, theme).map_err(|e| e.to_string())?;
  let palette = theme_palette(theme).map_err(|e| e.to_string())?;

  // If target_width is set, compute scale to fit that width.
  if let Some(target) = settings.target_width.get_untracked() {
    options.scale = compute_scale_for_width(&tokens, &options, target);
  }

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
  let url = Url::create_object_url_with_blob(&blob).map_err(|e| format!("{e:?}"))?;
  let anchor: HtmlAnchorElement = document
    .create_element("a")
    .map_err(|e| format!("{e:?}"))?
    .unchecked_into();
  anchor.set_href(&url);
  anchor.set_download(&format!("{}.png", settings.expanded_filename()));
  anchor.click();
  let _ = Url::revoke_object_url(&url);
  Ok(())
}

async fn do_export_svg(settings: Settings) -> Result<(), String> {
  let code = settings.code.get_untracked();
  let language = settings.language.get_untracked();
  let theme = settings.theme.get_untracked();
  let mut options = settings.export_options();

  ensure_fonts_ready(options.font_family.css_family()).await;

  let tokens = highlight(&code, language, theme).map_err(|e| e.to_string())?;
  let palette = theme_palette(theme).map_err(|e| e.to_string())?;

  // If target_width is set, compute scale to fit that width.
  if let Some(target) = settings.target_width.get_untracked() {
    options.scale = compute_scale_for_width(&tokens, &options, target);
  }

  let (svg_string, _layout) = render_svg(&tokens, &palette, &options);

  let window = web_sys::window().ok_or_else(|| "no window object".to_string())?;
  let document = window
    .document()
    .ok_or_else(|| "no document object".to_string())?;

  let blob_parts = js_sys::Array::new();
  blob_parts.push(&JsValue::from_str(&svg_string));
  let blob_opts = web_sys::BlobPropertyBag::new();
  blob_opts.set_type("image/svg+xml");
  let blob = Blob::new_with_str_sequence_and_options(&blob_parts, &blob_opts)
    .map_err(|e| format!("{e:?}"))?;

  let url = Url::create_object_url_with_blob(&blob).map_err(|e| format!("{e:?}"))?;
  let anchor: HtmlAnchorElement = document
    .create_element("a")
    .map_err(|e| format!("{e:?}"))?
    .unchecked_into();
  anchor.set_href(&url);
  anchor.set_download(&format!("{}.svg", settings.expanded_filename()));
  anchor.click();
  let _ = Url::revoke_object_url(&url);
  Ok(())
}

/// Compute the export scale needed to make the output image `target_width`
/// pixels wide. Uses a 1x measurement pass to find the logical width.
fn compute_scale_for_width(
  tokens: &[codeframe_models::Token],
  options: &codeframe_models::ExportOptions,
  target_width: f64,
) -> f64 {
  use codeframe_renderer::layout::split_tokens_into_lines;

  // Approximate char width (monospace heuristic).
  let char_width = options.font_size * 0.602;
  let lines = split_tokens_into_lines(tokens, options.tab_width);
  let mut max_line_width = 0.0_f64;
  for line in &lines {
    let mut w = 0.0;
    for token in line {
      w += token.text.len() as f64 * char_width;
    }
    max_line_width = max_line_width.max(w);
  }

  let layout = codeframe_renderer::compute_layout(options, lines.len(), max_line_width, char_width);
  // Scale = target / logical. Clamp to sensible range.
  (target_width / layout.canvas_width).clamp(0.5, 12.0)
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
  js_sys::Reflect::set(&record, &"image/png".into(), &blob).map_err(|e| format!("{e:?}"))?;

  let item = ClipboardItem::new_with_record_from_str_to_blob_promise(&record)
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
