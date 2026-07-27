//! UI theme toggle button - cycles through light / dark / sepia.

use leptos::prelude::*;

use crate::state::{Settings, UiTheme};

/// Theme toggle button placed in the topbar. Cycles light → dark → sepia →
/// light on each click and persists the choice to localStorage.
#[component]
pub fn ThemeToggle(settings: Settings) -> impl IntoView {
  view! {
      <button
          class="theme-toggle"
          title=move || format!("Theme: {} - click to cycle", settings.ui_theme.get().as_str())
          aria-label=move || format!("Current theme: {}. Click to cycle themes.", settings.ui_theme.get().as_str())
          on:click=move |_| {
              let next = settings.ui_theme.get().next();
              settings.ui_theme.set(next);
              settings.persist_ui_theme();
          }
      >
          {move || match settings.ui_theme.get() {
              UiTheme::Light => view! { <SunIcon /> }.into_any(),
              UiTheme::Dark => view! { <MoonIcon /> }.into_any(),
              UiTheme::Sepia => view! { <CoffeeIcon /> }.into_any(),
          }}
      </button>
  }
}

/// lucide `sun` icon (light theme).
#[component]
fn SunIcon() -> impl IntoView {
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
          <circle cx="12" cy="12" r="4" />
          <path d="M12 2v2" />
          <path d="M12 20v2" />
          <path d="m4.93 4.93 1.41 1.41" />
          <path d="m17.66 17.66 1.41 1.41" />
          <path d="M2 12h2" />
          <path d="M20 12h2" />
          <path d="m6.34 17.66-1.41 1.41" />
          <path d="m19.07 4.93-1.41 1.41" />
      </svg>
  }
}

/// lucide `moon` icon (dark theme).
#[component]
fn MoonIcon() -> impl IntoView {
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
          <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
      </svg>
  }
}

/// lucide `coffee` icon (sepia theme).
#[component]
fn CoffeeIcon() -> impl IntoView {
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
          <path d="M17 8h1a4 4 0 1 1 0 8h-1" />
          <path d="M3 8h14v9a4 4 0 0 1-4 4H7a4 4 0 0 1-4-4Z" />
          <line x1="6" x2="6" y1="2" y2="4" />
          <line x1="10" x2="10" y1="2" y2="4" />
          <line x1="14" x2="14" y1="2" y2="4" />
      </svg>
  }
}
