//! Sidebar with every user control: code input, language/theme/font pickers,
//! sliders, scale selector, toggles, and background swatches.

use codeshot_models::{Background, FontChoice, Language, ThemeChoice};
use leptos::prelude::*;

use crate::state::{Settings, SAMPLE_CODE};

const SCALE_PRESETS: [f64; 4] = [1.0, 2.0, 4.0, 8.0];

/// CSS value used to paint a background swatch button.
fn background_css(background: &Background) -> String {
  match background {
    Background::Solid(color) => color.to_css(),
    Background::Gradient(colors) => {
      let stops: Vec<String> = colors.iter().map(|c| c.to_css()).collect();
      format!("linear-gradient(135deg, {})", stops.join(", "))
    }
  }
}

/// A `<select>` bound to an `RwSignal` over a simple enum.
#[component]
fn EnumSelect<T>(
  value: RwSignal<T>,
  options: &'static [T],
  label: fn(T) -> &'static str,
) -> impl IntoView
where
  T: Copy + Eq + Send + Sync + 'static,
{
  view! {
      <select
          prop:value=move || label(value.get())
          on:change=move |ev| {
              let selected = event_target_value(&ev);
              if let Some(found) = options.iter().copied().find(|o| label(*o) == selected) {
                  value.set(found);
              }
          }
      >
          {options
              .iter()
              .map(|option| view! { <option value=label(*option)>{label(*option)}</option> })
              .collect_view()}
      </select>
  }
}

/// A range slider with a formatted value readout.
#[component]
fn Slider(
  value: RwSignal<f64>,
  min: f64,
  max: f64,
  step: f64,
  format: fn(f64) -> String,
) -> impl IntoView {
  view! {
      <div class="slider-row">
          <input
              type="range"
              min=min.to_string()
              max=max.to_string()
              step=step.to_string()
              prop:value=move || value.get().to_string()
              on:input=move |ev| {
                  if let Ok(parsed) = event_target_value(&ev).parse::<f64>() {
                      value.set(parsed);
                  }
              }
          />
          <span class="slider-value">{move || format(value.get())}</span>
      </div>
  }
}

/// lucide `triangle-alert` icon.
#[component]
fn AlertTriangleIcon() -> impl IntoView {
  view! {
      <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
      >
          <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" />
          <path d="M12 9v4" />
          <path d="M12 17h.01" />
      </svg>
  }
}

#[component]
pub fn Controls(settings: Settings) -> impl IntoView {
  view! {
      <aside class="controls">
          <section>
              <label class="control-label" for="code-input">"Code"</label>
              <textarea
                  id="code-input"
                  class="code-input"
                  rows="12"
                  spellcheck="false"
                  autocomplete="off"
                  on:input=move |ev| settings.code.set(event_target_value(&ev))
              >{SAMPLE_CODE}</textarea>
          </section>

          <div class="control-row">
              <section>
                  <label class="control-label">"Language"</label>
                  <EnumSelect
                      value=settings.language
                      options=&Language::ALL
                      label=Language::display_name
                  />
              </section>
              <section>
                  <label class="control-label">"Theme"</label>
                  <EnumSelect
                      value=settings.theme
                      options=&ThemeChoice::ALL
                      label=ThemeChoice::display_name
                  />
              </section>
          </div>

          <section>
              <label class="control-label">"Font"</label>
              <EnumSelect
                  value=settings.font
                  options=&FontChoice::ALL
                  label=FontChoice::display_name
              />
              {move || {
                  settings.font.get().has_ligatures().then(|| {
                      view! {
                          <p class="hint" title="Canvas2D fillText does not shape ligatures: sequences like != or => are exported as separate glyphs.">
                              <AlertTriangleIcon />
                              "This font has ligatures, but canvas export cannot render them."
                          </p>
                      }
                  })
              }}
          </section>

          <section>
              <label class="control-label">"Font size"</label>
              <Slider value=settings.font_size min=10.0 max=24.0 step=1.0 format=|v| format!("{v}px") />
          </section>

          <section>
              <label class="control-label">"Padding"</label>
              <Slider value=settings.padding min=16.0 max=128.0 step=8.0 format=|v| format!("{v}px") />
          </section>

          <section>
              <label class="control-label">"Corner radius"</label>
              <Slider value=settings.corner_radius min=0.0 max=24.0 step=1.0 format=|v| format!("{v}px") />
          </section>

          <section>
              <label class="control-label">"Export scale"</label>
              <div class="segmented">
                  {SCALE_PRESETS
                      .into_iter()
                      .map(|preset| {
                          view! {
                              <button
                                  class="seg"
                                  class:active=move || settings.scale.get() == preset
                                  on:click=move |_| settings.scale.set(preset)
                              >
                                  {format!("{preset}x")}
                              </button>
                          }
                      })
                      .collect_view()}
                  <input
                      class="scale-custom"
                      type="number"
                      min="1"
                      max="12"
                      step="0.5"
                      title="Custom scale (1–12)"
                      prop:value=move || settings.scale.get().to_string()
                      on:change=move |ev| {
                          if let Ok(parsed) = event_target_value(&ev).parse::<f64>() {
                              settings.scale.set(parsed.clamp(0.5, 12.0));
                          }
                      }
                  />
              </div>
          </section>

          <section class="toggles">
              <label class="toggle">
                  <input
                      type="checkbox"
                      prop:checked=move || settings.window_frame.get()
                      on:change=move |ev| settings.window_frame.set(event_target_checked(&ev))
                  />
                  "Window frame"
              </label>
              <label class="toggle">
                  <input
                      type="checkbox"
                      prop:checked=move || settings.line_numbers.get()
                      on:change=move |ev| settings.line_numbers.set(event_target_checked(&ev))
                  />
                  "Line numbers"
              </label>
          </section>

          <section>
              <label class="control-label">"Background"</label>
              <div class="swatches">
                  {Background::presets()
                      .into_iter()
                        .map(|(name, background)| {
                            let css = format!("background: {}", background_css(&background));
                            let bg_for_class = background.clone();
                          view! {
                              <button
                                  class="swatch"
                                  class:active=move || settings.background.get() == bg_for_class
                                  title=name
                                  style=css
                                  on:click=move |_| settings.background.set(background.clone())
                              ></button>
                          }
                      })
                      .collect_view()}
              </div>
          </section>
      </aside>
  }
}
