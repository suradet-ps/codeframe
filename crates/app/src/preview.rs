//! Live preview canvas. Renders on every state change at a screen-friendly
//! scale (capped - AGENTS.md §4, rule 3); export uses a separate canvas.

use codeframe_highlighter::{highlight, theme_palette};
use codeframe_renderer::render_to_canvas;
use leptos::html::Canvas;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::fonts::ensure_fonts_ready;
use crate::state::Settings;

/// Preview backing-store scale cap (typing must stay jank-free).
const PREVIEW_SCALE_CAP: f64 = 2.0;

#[component]
pub fn Preview(settings: Settings) -> impl IntoView {
  let canvas_ref: NodeRef<Canvas> = NodeRef::new();
  let error: RwSignal<Option<String>> = RwSignal::new(None);
  // Incremented per draw request; a draw that finishes stale is dropped.
  let generation: RwSignal<u64> = RwSignal::new(0);

  Effect::new(move || {
    // Track everything that affects the image.
    let code = settings.code.get();
    let language = settings.language.get();
    let theme = settings.theme.get();
    let mut options = settings.export_options();

    let device_pixel_ratio = web_sys::window()
      .map(|w| w.device_pixel_ratio())
      .unwrap_or(1.0);
    options.scale = device_pixel_ratio.clamp(1.0, PREVIEW_SCALE_CAP);

    generation.update(|g| *g += 1);
    let my_generation = generation.get_untracked();

    spawn_local(async move {
      ensure_fonts_ready(options.font_family.css_family()).await;
      if generation.get_untracked() != my_generation {
        return; // superseded by a newer draw
      }
      let Some(canvas) = canvas_ref.get() else {
        return;
      };
      let result: Result<codeframe_renderer::Layout, String> = (|| {
        let tokens = highlight(&code, language, theme).map_err(|e| e.to_string())?;
        let palette = theme_palette(theme).map_err(|e| e.to_string())?;
        render_to_canvas(&canvas, &tokens, &palette, &options).map_err(|e| e.to_string())
      })();
      match result {
        Ok(layout) => {
          error.set(None);
          // CSS-size the canvas to logical px; `max-width` + `height:
          // auto` in the stylesheet keep the aspect ratio when the
          // panel is narrower than the image.
          let _ = canvas.set_attribute("style", &format!("width:{}px", layout.canvas_width));
        }
        Err(message) => error.set(Some(message)),
      }
    });
  });

  view! {
      <div class="preview-area">
          <canvas node_ref=canvas_ref class="preview-canvas"></canvas>
          {move || {
              error.get().map(|message| view! {
                  <div class="error-banner">{message}</div>
              })
          }}
      </div>
  }
}
