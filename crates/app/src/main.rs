//! CodeShot — Leptos CSR frontend.
//!
//! This is the only crate that knows about Leptos: components, signals, and
//! event handlers live here (AGENTS.md §3).
#![deny(unsafe_code)]

mod controls;
mod export;
mod fonts;
mod preview;
mod state;

use leptos::mount::mount_to_body;
use leptos::prelude::*;

use crate::controls::Controls;
use crate::preview::Preview;
use crate::state::Settings;

#[component]
fn App() -> impl IntoView {
  let settings = Settings::new();
  let (exporting, set_exporting) = signal(false);
  let (export_error, set_export_error) = signal(Option::<String>::None);

  view! {
      <div class="app">
          <header class="topbar">
              <div class="brand">
                  <span class="brand-name">"CodeShot"</span>
                  <span class="brand-tag">"code → png"</span>
              </div>
              <div class="topbar-actions">
                  {move || {
                      export_error.get().map(|message| view! {
                          <span class="export-error">{message}</span>
                      })
                  }}
                  <button
                      class="export-btn"
                      disabled=move || exporting.get()
                      on:click=move |_| {
                          set_export_error.set(None);
                          export::export_png(settings, set_exporting, set_export_error);
                      }
                  >
                      <DownloadIcon />
                      {move || if exporting.get() { "Exporting…" } else { "Export PNG" }}
                  </button>
              </div>
          </header>
          <Controls settings />
          <main class="main">
              <Preview settings />
          </main>
      </div>
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

fn main() {
  console_error_panic_hook::set_once();
  mount_to_body(App);
}
