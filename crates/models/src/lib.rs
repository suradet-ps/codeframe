//! Shared vocabulary types for CodeFrame.
//!
//! This crate intentionally has no dependencies beyond `serde` - it must
//! compile on any target (including non-wasm hosts) and knows nothing about
//! Leptos, syntect, or the Canvas2D API (see AGENTS.md §6).
#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

/// An opaque 8-bit RGB color.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RgbColor {
  pub r: u8,
  pub g: u8,
  pub b: u8,
}

impl RgbColor {
  pub const fn new(r: u8, g: u8, b: u8) -> Self {
    Self { r, g, b }
  }

  /// `#rrggbb` CSS color string.
  pub fn to_css(&self) -> String {
    format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
  }

  /// `rgba(r, g, b, a)` CSS color string with `alpha` in `0.0..=1.0`.
  pub fn to_css_with_alpha(&self, alpha: f64) -> String {
    format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, alpha)
  }
}

/// Text styling attached to a [`Token`], derived from theme rules.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FontStyle {
  pub bold: bool,
  pub italic: bool,
  pub underline: bool,
}

/// A run of highlighted text sharing one color and style.
///
/// Token text may contain `'\n'` - splitting into lines is the renderer's job
/// (`layout::split_tokens_into_lines`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Token {
  pub text: String,
  pub color: RgbColor,
  pub font_style: FontStyle,
}

/// Direction for a linear gradient.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GradientDir {
  ToBottom,
  ToTop,
  ToRight,
  ToLeft,
}

impl GradientDir {
  /// CSS angle in degrees for this direction.
  pub fn css_angle(self) -> f64 {
    match self {
      GradientDir::ToBottom => 180.0,
      GradientDir::ToTop => 0.0,
      GradientDir::ToRight => 90.0,
      GradientDir::ToLeft => 270.0,
    }
  }
}

/// Canvas backdrop painted behind the code card.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Background {
  Solid(RgbColor),
  LinearGradient {
    colors: Vec<RgbColor>,
    dir: GradientDir,
  },
  RadialGradient {
    colors: Vec<RgbColor>,
  },
}

impl Background {
  /// Built-in B&W presets shown in the UI: `(display name, background)`.
  pub fn presets() -> Vec<(&'static str, Background)> {
    let black = RgbColor::new(0x00, 0x00, 0x00);
    let white = RgbColor::new(0xff, 0xff, 0xff);
    let dark_gray = RgbColor::new(0x1a, 0x1a, 0x1a);
    vec![
      ("Snow", Background::Solid(white)),
      (
        "Top Glow",
        Background::LinearGradient {
          colors: vec![white, black],
          dir: GradientDir::ToBottom,
        },
      ),
      (
        "Bottom Glow",
        Background::LinearGradient {
          colors: vec![black, white],
          dir: GradientDir::ToTop,
        },
      ),
      (
        "Left Beam",
        Background::LinearGradient {
          colors: vec![white, black],
          dir: GradientDir::ToRight,
        },
      ),
      (
        "Right Beam",
        Background::LinearGradient {
          colors: vec![black, white],
          dir: GradientDir::ToLeft,
        },
      ),
      (
        "Center Radial",
        Background::RadialGradient {
          colors: vec![white, black],
        },
      ),
      (
        "Dark Vignette",
        Background::RadialGradient {
          colors: vec![black, dark_gray, white],
        },
      ),
    ]
  }
}

impl Default for Background {
  fn default() -> Self {
    Background::Solid(RgbColor::new(0xff, 0xff, 0xff))
  }
}

/// One of the monospace fonts bundled with the app (`fonts/` directory).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FontChoice {
  #[default]
  JetBrainsMono,
  FiraCode,
  CascadiaCode,
}

impl FontChoice {
  pub const ALL: [FontChoice; 3] = [
    FontChoice::JetBrainsMono,
    FontChoice::FiraCode,
    FontChoice::CascadiaCode,
  ];

  pub fn display_name(self) -> &'static str {
    match self {
      FontChoice::JetBrainsMono => "JetBrains Mono",
      FontChoice::FiraCode => "Fira Code",
      FontChoice::CascadiaCode => "Cascadia Code",
    }
  }

  /// CSS `font-family` name, matching the `@font-face` rules in `style.css`.
  pub fn css_family(self) -> &'static str {
    self.display_name()
  }

  /// Whether the font ships ligature glyphs. Canvas2D `fillText` does not
  /// shape ligatures, so the UI warns when such a font is selected
  /// (AGENTS.md §5). All currently bundled fonts have ligatures.
  pub fn has_ligatures(self) -> bool {
    true
  }
}

/// Languages selectable in the UI.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
  #[default]
  Rust,
  Python,
  JavaScript,
  TypeScript,
  Go,
  Java,
  C,
  Cpp,
  Html,
  Css,
  Json,
  Yaml,
  Toml,
  Bash,
  Sql,
}

impl Language {
  pub const ALL: [Language; 15] = [
    Language::Rust,
    Language::Python,
    Language::JavaScript,
    Language::TypeScript,
    Language::Go,
    Language::Java,
    Language::C,
    Language::Cpp,
    Language::Html,
    Language::Css,
    Language::Json,
    Language::Yaml,
    Language::Toml,
    Language::Bash,
    Language::Sql,
  ];

  pub fn display_name(self) -> &'static str {
    match self {
      Language::Rust => "Rust",
      Language::Python => "Python",
      Language::JavaScript => "JavaScript",
      Language::TypeScript => "TypeScript",
      Language::Go => "Go",
      Language::Java => "Java",
      Language::C => "C",
      Language::Cpp => "C++",
      Language::Html => "HTML",
      Language::Css => "CSS",
      Language::Json => "JSON",
      Language::Yaml => "YAML",
      Language::Toml => "TOML",
      Language::Bash => "Bash",
      Language::Sql => "SQL",
    }
  }

  /// Token used to look up this language in the syntect syntax set
  /// (matches a file extension of the corresponding syntax definition).
  pub fn syntax_token(self) -> &'static str {
    match self {
      Language::Rust => "rs",
      Language::Python => "py",
      Language::JavaScript => "js",
      Language::TypeScript => "ts",
      Language::Go => "go",
      Language::Java => "java",
      Language::C => "c",
      Language::Cpp => "cpp",
      Language::Html => "html",
      Language::Css => "css",
      Language::Json => "json",
      Language::Yaml => "yaml",
      Language::Toml => "toml",
      Language::Bash => "sh",
      Language::Sql => "sql",
    }
  }
}

/// Syntax color themes bundled as `.tmTheme` files in `themes/` and embedded
/// at compile time by the `highlighter` crate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemeChoice {
  #[default]
  Dracula,
  OneDark,
  Nord,
  GithubLight,
  TokyoNight,
  CatppuccinMocha,
  Monokai,
}

impl ThemeChoice {
  pub const ALL: [ThemeChoice; 7] = [
    ThemeChoice::Dracula,
    ThemeChoice::OneDark,
    ThemeChoice::Nord,
    ThemeChoice::GithubLight,
    ThemeChoice::TokyoNight,
    ThemeChoice::CatppuccinMocha,
    ThemeChoice::Monokai,
  ];

  pub fn display_name(self) -> &'static str {
    match self {
      ThemeChoice::Dracula => "Dracula",
      ThemeChoice::OneDark => "One Dark",
      ThemeChoice::Nord => "Nord",
      ThemeChoice::GithubLight => "GitHub Light",
      ThemeChoice::TokyoNight => "Tokyo Night",
      ThemeChoice::CatppuccinMocha => "Catppuccin Mocha",
      ThemeChoice::Monokai => "Monokai",
    }
  }

  pub fn is_dark(self) -> bool {
    !matches!(self, ThemeChoice::GithubLight)
  }
}

/// Base colors of a theme, derived from a [`ThemeChoice`] by the highlighter
/// crate. The renderer needs these for the code-card fill and line numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemePalette {
  pub background: RgbColor,
  pub foreground: RgbColor,
}

/// Everything needed to lay out and draw a code image.
///
/// All lengths are logical (1x) pixels; the canvas backing store is
/// `logical * scale` (AGENTS.md §4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExportOptions {
  /// Device-pixel multiplier: 1.0, 2.0, 4.0, 8.0, or a custom value.
  pub scale: f64,
  /// Space between canvas edge and the code card (logical px).
  pub padding: f64,
  pub background: Background,
  /// macOS-style window chrome (header bar + traffic lights + shadow).
  pub window_frame: bool,
  pub line_numbers: bool,
  pub font_family: FontChoice,
  pub font_size: f64,
  /// Multiple of `font_size`.
  pub line_height: f64,
  pub corner_radius: f64,
  /// Number of spaces a tab character expands to.
  pub tab_width: usize,
}

impl Default for ExportOptions {
  fn default() -> Self {
    Self {
      scale: 2.0,
      padding: 48.0,
      background: Background::default(),
      window_frame: true,
      line_numbers: false,
      font_family: FontChoice::default(),
      font_size: 14.0,
      line_height: 1.5,
      corner_radius: 12.0,
      tab_width: 4,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rgb_color_to_css() {
    assert_eq!(RgbColor::new(0x28, 0x2a, 0x36).to_css(), "#282a36");
    assert_eq!(
      RgbColor::new(255, 121, 198).to_css_with_alpha(0.5),
      "rgba(255, 121, 198, 0.5)"
    );
  }

  #[test]
  fn background_presets_are_non_empty_and_valid() {
    let presets = Background::presets();
    assert!(presets.len() >= 2);
    for (_, bg) in &presets {
      match bg {
        Background::Solid(_) => {}
        Background::LinearGradient { colors, .. } => assert!(colors.len() >= 2),
        Background::RadialGradient { colors } => assert!(colors.len() >= 2),
      }
    }
  }

  #[test]
  fn language_tokens_are_unique() {
    let mut tokens: Vec<_> = Language::ALL.iter().map(|l| l.syntax_token()).collect();
    tokens.sort_unstable();
    let len = tokens.len();
    tokens.dedup();
    assert_eq!(tokens.len(), len);
  }

  #[test]
  fn font_css_families_match_style_css() {
    // Keep in sync with the @font-face declarations in style.css.
    assert_eq!(FontChoice::JetBrainsMono.css_family(), "JetBrains Mono");
    assert_eq!(FontChoice::FiraCode.css_family(), "Fira Code");
    assert_eq!(FontChoice::CascadiaCode.css_family(), "Cascadia Code");
  }
}
