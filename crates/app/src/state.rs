//! Shared application state: one struct of signals passed to all components.

use codeshot_models::{Background, ExportOptions, FontChoice, Language, RgbColor, ThemeChoice};
use leptos::prelude::*;

/// Sample code shown on first load.
pub const SAMPLE_CODE: &str = r#"fn main() {
    // CodeShot — turn code into beautiful images
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
}

impl Settings {
  pub fn new() -> Self {
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
    }
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
      line_height: 1.5,
      corner_radius: self.corner_radius.get(),
    }
  }
}
