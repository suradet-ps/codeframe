//! Syntax-highlighted code input: a transparent-text textarea overlaid on a
//! `<pre>` rendered from the token stream, so the input mirrors the export.

use codeframe_models::{Language, ThemeChoice};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// Palette-derived editor background + caret color, painted on the wrapper so
/// the input matches the export canvas for the same theme.
fn editor_colors(theme: ThemeChoice) -> String {
  match codeframe_highlighter::theme_palette(theme) {
    Ok(palette) => format!(
      "background: {}; caret-color: {};",
      palette.background.to_css(),
      palette.foreground.to_css()
    ),
    Err(_) => "caret-color: var(--ink);".to_string(),
  }
}

/// A code textarea with live syntax highlighting behind the text.
///
/// The `<pre>` layer carries the token colors (HTML from
/// `codeframe_highlighter::highlight_to_html`); the textarea above it uses
/// transparent text so only the caret and selection are visible. Both layers
/// share identical font metrics and wrapping, and the `<pre>` scrolls in
/// lockstep with the textarea.
#[component]
pub fn CodeEditor(
  id: &'static str,
  code: RwSignal<String>,
  language: RwSignal<Language>,
  theme: RwSignal<ThemeChoice>,
) -> impl IntoView {
  let pre_ref: NodeRef<leptos::html::Pre> = NodeRef::new();
  let textarea_ref: NodeRef<leptos::html::Textarea> = NodeRef::new();

  let html = Memo::new(move |_| {
    let code = code.get();
    let language = language.get();
    let theme = theme.get();
    match codeframe_highlighter::highlight_to_html(&code, language, theme) {
      Ok(html) => html,
      Err(e) => format!(
        "<span style=\"color:var(--warning)\">{}</span>",
        codeframe_highlighter::escape_html(&e.to_string())
      ),
    }
  });

  view! {
      <div class="code-editor" style=move || editor_colors(theme.get())>
          <pre
              class="code-editor-highlight"
              aria-hidden="true"
              node_ref=pre_ref
              inner_html=move || html.get()
          ></pre>
          <textarea
              id=id
              class="code-input"
              rows="12"
              spellcheck="false"
              autocomplete="off"
              node_ref=textarea_ref
              prop:value=move || code.get()
              on:input=move |ev| code.set(event_target_value(&ev))
              on:keydown=move |ev| {
                  if ev.key() == "Tab" {
                      ev.prevent_default();
                      let target = ev.target().unwrap();
                      let textarea: web_sys::HtmlTextAreaElement = target.unchecked_into();
                      let start = textarea.selection_start().unwrap_or_default().unwrap_or(0) as usize;
                      let end = textarea.selection_end().unwrap_or_default().unwrap_or(0) as usize;
                      let value = textarea.value();
                      let new_value = format!("{}    {}", &value[..start], &value[end..]);
                      code.set(new_value.clone());
                      textarea.set_value(&new_value);
                      let pos = (start + 4) as u32;
                      let _ = textarea.set_selection_range(pos, pos);
                  }
              }
              on:scroll=move |_| {
                  if let (Some(pre), Some(textarea)) = (pre_ref.get(), textarea_ref.get()) {
                      pre.set_scroll_top(textarea.scroll_top());
                      pre.set_scroll_left(textarea.scroll_left());
                  }
              }
          ></textarea>
      </div>
  }
}
