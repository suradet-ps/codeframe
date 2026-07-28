//! Shared application state: one struct of signals passed to all components.

use codeframe_models::{Background, ExportOptions, FontChoice, Language, RgbColor, ThemeChoice};
use leptos::prelude::*;
use web_sys::window;

/// UI chrome theme - controls the sidebar/topbar appearance, independent of
/// the canvas syntax theme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum UiTheme {
  #[default]
  Light,
  Dark,
  Sepia,
}

impl UiTheme {
  #[allow(dead_code)]
  pub const ALL: [UiTheme; 3] = [UiTheme::Light, UiTheme::Dark, UiTheme::Sepia];

  pub fn as_str(self) -> &'static str {
    match self {
      UiTheme::Light => "light",
      UiTheme::Dark => "dark",
      UiTheme::Sepia => "sepia",
    }
  }

  pub fn from_str(s: &str) -> Self {
    match s {
      "dark" => UiTheme::Dark,
      "sepia" => UiTheme::Sepia,
      _ => UiTheme::Light,
    }
  }

  /// Cycle to the next theme in the sequence: light → dark → sepia → light.
  pub fn next(self) -> Self {
    match self {
      UiTheme::Light => UiTheme::Dark,
      UiTheme::Dark => UiTheme::Sepia,
      UiTheme::Sepia => UiTheme::Light,
    }
  }
}

const STORAGE_KEY: &str = "codeframe-ui-theme";

/// Read the persisted theme from `localStorage`. Falls back to `Light`.
fn load_theme_from_storage() -> UiTheme {
  let Some(win) = window() else {
    return UiTheme::Light;
  };
  let Some(storage) = win.local_storage().ok().flatten() else {
    return UiTheme::Light;
  };
  let Some(value) = storage.get_item(STORAGE_KEY).ok().flatten() else {
    return UiTheme::Light;
  };
  UiTheme::from_str(&value)
}

/// Persist the theme to `localStorage`.
fn save_theme_to_storage(theme: UiTheme) {
  let Some(win) = window() else {
    return;
  };
  let Some(storage) = win.local_storage().ok().flatten() else {
    return;
  };
  let _ = storage.set_item(STORAGE_KEY, theme.as_str());
}

/// Sample code shown on first load.
pub const SAMPLE_CODE: &str = r#"fn main() {
    // CodeFrame - turn code into beautiful images
    let message = "Hello, world!";
    println!("{message}");
}
"#;

/// Every user-tweakable input, as fine-grained signals (cheap to copy and
/// hand to child components).
#[derive(Clone, Copy)]
pub struct Settings {
  pub code: RwSignal<String>,
  pub language: RwSignal<Language>,
  pub theme: RwSignal<ThemeChoice>,
  pub font: RwSignal<FontChoice>,
  pub font_size: RwSignal<f64>,
  pub padding: RwSignal<f64>,
  pub corner_radius: RwSignal<f64>,
  pub scale: RwSignal<f64>,
  pub window_frame: RwSignal<bool>,
  pub line_numbers: RwSignal<bool>,
  pub background: RwSignal<Background>,
  pub ui_theme: RwSignal<UiTheme>,
  pub line_height: RwSignal<f64>,
  pub tab_width: RwSignal<usize>,
}

impl Settings {
  pub fn new() -> Self {
    let initial_ui_theme = load_theme_from_storage();
    Self {
      code: RwSignal::new(SAMPLE_CODE.to_string()),
      language: RwSignal::new(Language::Rust),
      theme: RwSignal::new(ThemeChoice::Dracula),
      font: RwSignal::new(FontChoice::JetBrainsMono),
      font_size: RwSignal::new(14.0),
      padding: RwSignal::new(48.0),
      corner_radius: RwSignal::new(12.0),
      scale: RwSignal::new(2.0),
      window_frame: RwSignal::new(true),
      line_numbers: RwSignal::new(false),
      background: RwSignal::new(Background::Gradient(vec![
        RgbColor::new(0x63, 0x66, 0xf1),
        RgbColor::new(0xa8, 0x55, 0xf7),
        RgbColor::new(0xec, 0x48, 0x99),
      ])),
      ui_theme: RwSignal::new(initial_ui_theme),
      line_height: RwSignal::new(1.5),
      tab_width: RwSignal::new(4),
    }
  }

  /// Persist the current UI theme to localStorage.
  pub fn persist_ui_theme(&self) {
    save_theme_to_storage(self.ui_theme.get());
  }

  /// Snapshot of the current settings as renderer-ready options.
  pub fn export_options(&self) -> ExportOptions {
    ExportOptions {
      scale: self.scale.get(),
      padding: self.padding.get(),
      background: self.background.get(),
      window_frame: self.window_frame.get(),
      line_numbers: self.line_numbers.get(),
      font_family: self.font.get(),
      font_size: self.font_size.get(),
      line_height: self.line_height.get(),
      corner_radius: self.corner_radius.get(),
      tab_width: self.tab_width.get(),
    }
  }
}
