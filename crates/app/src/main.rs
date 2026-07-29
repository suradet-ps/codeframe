//! CodeFrame - Leptos CSR frontend.
//!
//! This is the only crate that knows about Leptos: components, signals, and
//! event handlers live here (AGENTS.md §3).
#![deny(unsafe_code)]

mod controls;
mod export;
mod fonts;
mod preview;
mod pwa;
mod state;
mod theme;

use leptos::mount::mount_to_body;
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::window;

use crate::controls::Controls;
use crate::preview::Preview;
use crate::pwa::PwaBanners;
use crate::state::Settings;
use crate::theme::ThemeToggle;

#[component]
fn App() -> impl IntoView {
  let settings = Settings::new();
  let (exporting, set_exporting) = signal(false);
  let (export_error, set_export_error) = signal(Option::<String>::None);
  let (copied, set_copied) = signal(false);

  // Sync the data-theme attribute on <html> whenever ui_theme changes.
  Effect::new(move |_| {
    let theme = settings.ui_theme.get();
    if let Some(html) = window()
      .and_then(|w| w.document())
      .and_then(|d| d.document_element())
    {
      let _ = html.set_attribute("data-theme", theme.as_str());
    }
  });

  // Keyboard shortcuts: Ctrl/Cmd+Enter to export.
  {
    let settings_kbd = settings;
    let set_export_error_kbd = set_export_error;
    let set_exporting_kbd = set_exporting;
    let listener = Closure::wrap(Box::new(move |ev: web_sys::KeyboardEvent| {
      let ctrl = ev.ctrl_key() || ev.meta_key();
      if ctrl && ev.key() == "Enter" {
        ev.prevent_default();
        set_export_error_kbd.set(None);
        export::export_png(settings_kbd, set_exporting_kbd, set_export_error_kbd);
      }
    }) as Box<dyn FnMut(_)>);
    if let Some(win) = window() {
      if let Some(doc) = win.document() {
        let _ = doc.add_event_listener_with_callback("keydown", listener.as_ref().unchecked_ref());
        listener.forget();
      }
    }
  }

  view! {
      <div class="app">
          <header class="topbar">
               <div class="brand">
                   <svg class="brand-logo" viewBox="0 0 512 512" width="32" height="32" aria-hidden="true">
                       <defs>
                           <linearGradient id="bgGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                               <stop offset="0%" stop-color="#09090b" />
                               <stop offset="100%" stop-color="#18181b" />
                           </linearGradient>
                       </defs>
                       <rect x="8" y="8" width="496" height="496" rx="116" fill="url(#bgGrad)" />
                       <rect x="88" y="64" width="336" height="384" rx="20" fill="#ffffff" />
                       <rect x="116" y="92" width="280" height="240" rx="12" fill="#0f172a" />
                       <circle cx="144" cy="118" r="6" fill="#ef4444" />
                       <circle cx="164" cy="118" r="6" fill="#f59e0b" />
                       <circle cx="184" cy="118" r="6" fill="#10b981" />
                       <rect x="144" y="152" width="125" height="11" rx="5.5" fill="#38bdf8" />
                       <rect x="144" y="182" width="200" height="11" rx="5.5" fill="#cbd5e1" />
                       <rect x="144" y="212" width="150" height="11" rx="5.5" fill="#c084fc" />
                       <rect x="144" y="242" width="105" height="11" rx="5.5" fill="#34d399" />
                       <circle cx="256" cy="392" r="18" fill="#71717a" />
                   </svg>
                  <span class="brand-name">"CodeFrame"</span>
                  <span class="brand-tag">"code → image"</span>
              </div>
              <div class="topbar-actions">
                  {move || {
                      export_error.get().map(|message| view! {
                          <span class="export-error" role="alert">{message}</span>
                      })
                  }}
                  <ThemeToggle settings />
                  <button
                      class="copy-btn"
                      aria-label="Copy to clipboard"
                      disabled=move || exporting.get()
                      on:click=move |_| {
                          set_export_error.set(None);
                          export::copy_to_clipboard(settings, set_copied, set_export_error);
                      }
                  >
                      <CopyIcon />
                      <span class="copy-label">{move || if copied.get() { "Copied!" } else { "Copy" }}</span>
                  </button>
                  <button
                      class="export-btn export-btn-secondary"
                      aria-label="Export as SVG"
                      disabled=move || exporting.get()
                      on:click=move |_| {
                          set_export_error.set(None);
                          export::export_svg(settings, set_exporting, set_export_error);
                      }
                  >
                      <SvgIcon />
                      <span class="btn-label">{move || if exporting.get() { "Exporting…" } else { "Export SVG" }}</span>
                  </button>
                  <button
                      class="export-btn"
                      aria-label="Export as PNG"
                      disabled=move || exporting.get()
                      on:click=move |_| {
                          set_export_error.set(None);
                          export::export_png(settings, set_exporting, set_export_error);
                      }
                  >
                      <DownloadIcon />
                      <span class="btn-label">{move || if exporting.get() { "Exporting…" } else { "Export PNG" }}</span>
                  </button>
              </div>
          </header>
          <Controls settings />
          <main class="main">
              <Preview settings />
          </main>
      </div>
      <PwaBanners />
  }
}

/// lucide `download` icon.
#[component]
fn DownloadIcon() -> impl IntoView {
  view! {
      <svg
          width="16"
          height="16"
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

/// lucide `clipboard` icon.
#[component]
fn CopyIcon() -> impl IntoView {
  view! {
      <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
      >
          <rect width="14" height="14" x="8" y="8" rx="2" ry="2" />
          <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
      </svg>
  }
}

/// lucide `file-code` icon for SVG export.
#[component]
fn SvgIcon() -> impl IntoView {
  view! {
      <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
      >
          <path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" />
          <path d="M14 2v4a2 2 0 0 0 2 2h4" />
          <path d="m10 13-2 2 2 2" />
          <path d="m14 17 2-2-2-2" />
      </svg>
  }
}

fn main() {
  console_error_panic_hook::set_once();
  mount_to_body(App);
}
