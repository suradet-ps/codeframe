//! Sidebar with every user control, grouped into labeled sections: code,
//! appearance, canvas geometry, frame toggles, and export settings.

use codeframe_models::{Background, FontChoice, Language, ThemeChoice};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::state::{Settings, SAMPLE_CODE};

const SCALE_PRESETS: [f64; 4] = [1.0, 2.0, 4.0, 8.0];

/// CSS value used to paint a background swatch button.
fn background_css(background: &Background) -> String {
  match background {
    Background::Solid(color) => color.to_css(),
    Background::LinearGradient { colors, dir } => {
      let stops: Vec<String> = colors.iter().map(|c| c.to_css()).collect();
      let kw = match dir {
        codeframe_models::GradientDir::ToBottom => "to bottom",
        codeframe_models::GradientDir::ToTop => "to top",
        codeframe_models::GradientDir::ToRight => "to right",
        codeframe_models::GradientDir::ToLeft => "to left",
      };
      format!("linear-gradient({kw}, {})", stops.join(", "))
    }
    Background::RadialGradient { colors } => {
      let stops: Vec<String> = colors.iter().map(|c| c.to_css()).collect();
      format!("radial-gradient(circle, {})", stops.join(", "))
    }
  }
}

/// Sidebar section header: small lucide icon + uppercase title.
#[component]
fn SectionHeader(title: &'static str, children: Children) -> impl IntoView {
  view! {
      <div class="section-header">
          <span class="section-icon" aria-hidden="true">{children()}</span>
          <h2 class="section-title">{title}</h2>
      </div>
  }
}

/// A `<select>` bound to an `RwSignal` over a simple enum.
#[component]
fn EnumSelect<T>(
  value: RwSignal<T>,
  options: &'static [T],
  label: fn(T) -> &'static str,
  #[prop(optional)] id: Option<&'static str>,
) -> impl IntoView
where
  T: Copy + Eq + Send + Sync + 'static,
{
  view! {
      <select
          id=id.unwrap_or_default()
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
  #[prop(optional)] id: Option<&'static str>,
) -> impl IntoView {
  view! {
      <div class="slider-row">
          <input
              type="range"
              id=id.unwrap_or_default()
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

/// A labeled toggle switch (checkbox + styled track/thumb).
#[component]
fn Toggle(checked: RwSignal<bool>, label: &'static str, id: &'static str) -> impl IntoView {
  view! {
      <label class="switch" for=id>
          <input
              id=id
              type="checkbox"
              role="switch"
              prop:checked=move || checked.get()
              on:change=move |ev| checked.set(event_target_checked(&ev))
          />
          <span class="switch-track" aria-hidden="true">
              <span class="switch-thumb" />
          </span>
          <span class="switch-label">{label}</span>
      </label>
  }
}

/// lucide `code` icon.
#[component]
fn CodeIcon() -> impl IntoView {
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
          <polyline points="16 18 22 12 16 6" />
          <polyline points="8 6 2 12 8 18" />
      </svg>
  }
}

/// lucide `palette` icon.
#[component]
fn PaletteIcon() -> impl IntoView {
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
          <circle cx="13.5" cy="6.5" r=".5" fill="currentColor" />
          <circle cx="17.5" cy="10.5" r=".5" fill="currentColor" />
          <circle cx="8.5" cy="7.5" r=".5" fill="currentColor" />
          <circle cx="6.5" cy="12.5" r=".5" fill="currentColor" />
          <path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z" />
      </svg>
  }
}

/// lucide `square` icon (canvas geometry).
#[component]
fn SquareIcon() -> impl IntoView {
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
          <rect width="18" height="18" x="3" y="3" rx="2" />
      </svg>
  }
}

/// lucide `rectangle-horizontal` icon (window frame).
#[component]
fn WindowIcon() -> impl IntoView {
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
          <rect width="20" height="16" x="2" y="4" rx="2" />
      </svg>
  }
}

/// lucide `download` icon.
#[component]
fn DownloadIcon() -> impl IntoView {
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
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <polyline points="7 10 12 15 17 10" />
          <line x1="12" x2="12" y1="15" y2="3" />
      </svg>
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
      <aside class="controls" aria-label="Settings">
          <section class="control-group">
              <SectionHeader title="Code"><CodeIcon /></SectionHeader>
              <div class="section-body">
                  <div>
                      <label class="control-label" for="code-input">{move || if settings.split_enabled.get() { "Left panel code" } else { "Code" }}</label>
                      <textarea
                          id="code-input"
                          class="code-input"
                          rows="12"
                          spellcheck="false"
                          autocomplete="off"
                          on:input=move |ev| settings.code.set(event_target_value(&ev))
                          on:keydown=move |ev| {
                              if ev.key() == "Tab" {
                                  ev.prevent_default();
                                  let target = ev.target().unwrap();
                                  let textarea: web_sys::HtmlTextAreaElement = target.unchecked_into();
                                  let start = textarea.selection_start().unwrap_or_default().unwrap_or(0) as usize;
                                  let end = textarea.selection_end().unwrap_or_default().unwrap_or(0) as usize;
                                  let value = textarea.value();
                                  let new_value = format!("{}    {}", &value[..start], &value[end..]);
                                  settings.code.set(new_value.clone());
                                  textarea.set_value(&new_value);
                                  let pos = (start + 4) as u32;
                                  let _ = textarea.set_selection_range(pos, pos);
                              }
                          }
                      >{SAMPLE_CODE}</textarea>
                  </div>

                  <Toggle id="split-toggle" checked=settings.split_enabled label="Split-screen comparison" />

                  {move || {
                      settings.split_enabled.get().then(|| {
                          let code_signal = settings.split_code;
                          view! {
                              <div class="split-controls">
                                  <div>
                                      <label class="control-label" for="split-code-input">"Right panel code"</label>
                                      <textarea
                                          id="split-code-input"
                                          class="code-input"
                                          rows="12"
                                          spellcheck="false"
                                          autocomplete="off"
                                          prop:value=move || code_signal.get()
                                          on:input=move |ev| code_signal.set(event_target_value(&ev))
                                          on:keydown=move |ev| {
                                              if ev.key() == "Tab" {
                                                  ev.prevent_default();
                                                  let target = ev.target().unwrap();
                                                  let textarea: web_sys::HtmlTextAreaElement = target.unchecked_into();
                                                  let start = textarea.selection_start().unwrap_or_default().unwrap_or(0) as usize;
                                                  let end = textarea.selection_end().unwrap_or_default().unwrap_or(0) as usize;
                                                  let value = textarea.value();
                                                  let new_value = format!("{}    {}", &value[..start], &value[end..]);
                                                  code_signal.set(new_value.clone());
                                                  textarea.set_value(&new_value);
                                                  let pos = (start + 4) as u32;
                                                  let _ = textarea.set_selection_range(pos, pos);
                                              }
                                          }
                                      ></textarea>
                                  </div>
                                  <div class="control-row">
                                      <div>
                                          <label class="control-label" for="split-theme-select">"Right panel theme"</label>
                                          <EnumSelect
                                              id="split-theme-select"
                                              value=settings.split_theme
                                              options=&ThemeChoice::ALL
                                              label=ThemeChoice::display_name
                                          />
                                      </div>
                                      <div>
                                          <label class="control-label" for="split-language-select">"Right panel language"</label>
                                          <EnumSelect
                                              id="split-language-select"
                                              value=settings.split_language
                                              options=&Language::ALL
                                              label=Language::display_name
                                          />
                                      </div>
                                  </div>
                                  <p class="hint">"Left panel uses the main theme/language above."</p>
                              </div>
                          }
                      })
                  }}
              </div>
          </section>

          <section class="control-group">
              <SectionHeader title="Appearance"><PaletteIcon /></SectionHeader>
              <div class="section-body">
                  <div class="control-row">
                      <div>
                          <label class="control-label" for="language-select">"Language"</label>
                          <EnumSelect
                              id="language-select"
                              value=settings.language
                              options=&Language::ALL
                              label=Language::display_name
                          />
                      </div>
                      <div>
                          <label class="control-label" for="theme-select">"Theme"</label>
                          <EnumSelect
                              id="theme-select"
                              value=settings.theme
                              options=&ThemeChoice::ALL
                              label=ThemeChoice::display_name
                          />
                      </div>
                  </div>

                  <div>
                      <label class="control-label" for="font-select">"Font"</label>
                      <EnumSelect
                          id="font-select"
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
                  </div>

                  <div>
                      <label class="control-label">"Background"</label>
                      <div class="bw-swatches" role="radiogroup" aria-label="Background preset">
                          {Background::presets()
                              .into_iter()
                              .map(|(name, background)| {
                                  let css = format!("background: {}", background_css(&background));
                                  let bg_for_cmp = background.clone();
                                  let bg_for_active = background.clone();
                                  view! {
                                      <button
                                          class="swatch bw-swatch"
                                          class:active=move || settings.background.get() == bg_for_cmp
                                          role="radio"
                                          aria-checked=move || settings.background.get() == bg_for_active
                                          aria-label=name
                                          title=name
                                          style=css
                                          on:click=move |_| {
                                              settings.background.set(background.clone());
                                          }
                                      ></button>
                                  }
                              })
                              .collect_view()}
                      </div>
                  </div>
              </div>
          </section>

          <section class="control-group">
              <SectionHeader title="Canvas"><SquareIcon /></SectionHeader>
              <div class="section-body slider-grid">
                  <div class="slider-cell">
                      <label class="control-label" for="font-size-slider">"Font size"</label>
                      <Slider id="font-size-slider" value=settings.font_size min=10.0 max=24.0 step=1.0 format=|v| format!("{v}px") />
                  </div>
                  <div class="slider-cell">
                      <label class="control-label" for="line-height-slider">"Line height"</label>
                      <Slider id="line-height-slider" value=settings.line_height min=1.0 max=2.5 step=0.1 format=|v| format!("{v:.1}") />
                  </div>
                  <div class="slider-cell">
                      <label class="control-label" for="padding-slider">"Padding"</label>
                      <Slider id="padding-slider" value=settings.padding min=16.0 max=128.0 step=8.0 format=|v| format!("{v}px") />
                  </div>
                  <div class="slider-cell">
                      <label class="control-label" for="corner-radius-slider">"Corner radius"</label>
                      <Slider id="corner-radius-slider" value=settings.corner_radius min=0.0 max=24.0 step=1.0 format=|v| format!("{v}px") />
                  </div>
              </div>
          </section>

          <section class="control-group">
              <SectionHeader title="Frame"><WindowIcon /></SectionHeader>
              <div class="section-body">
                  <div class="toggles">
                      <Toggle id="window-frame-toggle" checked=settings.window_frame label="Window frame" />
                      <Toggle id="line-numbers-toggle" checked=settings.line_numbers label="Line numbers" />
                  </div>
              </div>
          </section>

          <section class="control-group">
              <SectionHeader title="Export"><DownloadIcon /></SectionHeader>
              <div class="section-body">
                  <div>
                      <label class="control-label" id="export-scale-label">"Export scale"</label>
                      <div class="segmented" role="radiogroup" aria-labelledby="export-scale-label">
                          {SCALE_PRESETS
                              .into_iter()
                              .map(|preset| {
                                  view! {
                                      <button
                                          class="seg"
                                          class:active=move || settings.scale.get() == preset
                                          role="radio"
                                          aria-checked=move || settings.scale.get() == preset
                                          on:click=move |_| settings.scale.set(preset)
                                      >
                                          {format!("{preset}x")}
                                      </button>
                                  }
                              })
                              .collect_view()}
                          <input
                              class="scale-custom"
                              id="scale-custom"
                              type="number"
                              min="1"
                              max="12"
                              step="0.5"
                              title="Custom scale (1โ€“12)"
                              prop:value=move || settings.scale.get().to_string()
                              on:change=move |ev| {
                                  if let Ok(parsed) = event_target_value(&ev).parse::<f64>() {
                                      settings.scale.set(parsed.clamp(0.5, 12.0));
                                  }
                              }
                          />
                      </div>
                  </div>

                  <div>
                      <label class="control-label" for="target-width-input">"Target width"</label>
                      <div class="slider-row">
                          <input
                              id="target-width-input"
                              type="number"
                              min="320"
                              max="3840"
                              step="10"
                              placeholder="auto"
                              title="Lock export width in pixels (scale computed automatically)"
                              prop:value=move || {
                                  settings.target_width
                                      .get()
                                      .map(|w| format!("{w:.0}"))
                                      .unwrap_or_default()
                              }
                              on:input=move |ev| {
                                  let val = event_target_value(&ev);
                                  if val.is_empty() {
                                      settings.target_width.set(None);
                                  } else if let Ok(px) = val.parse::<f64>() {
                                      settings.target_width.set(Some(px.clamp(320.0, 3840.0)));
                                  }
                              }
                          />
                          <span class="slider-value">"px"</span>
                      </div>
                      <p class="hint">"Overrides scale. Leave empty for manual scale."</p>
                  </div>

                  <div>
                      <label class="control-label" for="filename-input">"Filename template"</label>
                      <input
                          id="filename-input"
                          class="filename-input"
                          type="text"
                          prop:value=move || settings.filename_template.get()
                          on:input=move |ev| settings.filename_template.set(event_target_value(&ev))
                          title="Placeholders: {scale}, {language}, {theme}, {timestamp}"
                      />
                      <p class="hint">"Preview: " {move || format!("{}.png", settings.expanded_filename())}</p>
                  </div>
              </div>
          </section>
      </aside>
  }
}
