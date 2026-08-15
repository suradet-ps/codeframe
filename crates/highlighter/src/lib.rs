//! Syntax highlighting via `syntect`, exposed as a framework-agnostic token
//! stream. Depends only on `codeframe-models` + `syntect` - it knows nothing
//! about canvas or Leptos (AGENTS.md §3).
//!
//! The bundled `.tmTheme` files from the workspace `themes/` directory are
//! embedded at compile time and parsed lazily on first use.
#![deny(unsafe_code)]

use std::io::Cursor;
use std::sync::OnceLock;

use codeframe_models::{FontStyle, Language, RgbColor, ThemeChoice, ThemePalette, Token};
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle as SyntectFontStyle, Theme, ThemeSet};
use syntect::parsing::{SyntaxDefinition, SyntaxSet};
use thiserror::Error;

/// Errors produced while highlighting code or loading a theme.
#[derive(Debug, Error)]
pub enum HighlightError {
  #[error("no syntax definition found for {0:?}")]
  NoSyntax(Language),
  #[error("failed to load syntax definitions: {0}")]
  SyntaxLoad(String),
  #[error("failed to load bundled theme {0:?}: {1}")]
  ThemeLoad(ThemeChoice, String),
  #[error("syntect failed while highlighting: {0}")]
  Syntect(String),
}

/// Extra grammars missing from syntect's default set (see `syntaxes/`).
const TYPESCRIPT_SYNTAX: &str = include_str!("../../../syntaxes/TypeScript.sublime-syntax");
const TOML_SYNTAX: &str = include_str!("../../../syntaxes/TOML.sublime-syntax");

fn build_syntax_set() -> Result<SyntaxSet, HighlightError> {
  let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
  for (name, source) in [("TypeScript", TYPESCRIPT_SYNTAX), ("TOML", TOML_SYNTAX)] {
    let definition = SyntaxDefinition::load_from_str(source, true, Some(name))
      .map_err(|e| HighlightError::SyntaxLoad(format!("{name}: {e}")))?;
    builder.add(definition);
  }
  Ok(builder.build())
}

/// Default syntax set (binary dump shipped inside syntect) plus the bundled
/// extra grammars, built once.
fn syntax_set() -> Result<&'static SyntaxSet, HighlightError> {
  static SYNTAX_SET: OnceLock<Result<SyntaxSet, HighlightError>> = OnceLock::new();
  SYNTAX_SET
    .get_or_init(build_syntax_set)
    .as_ref()
    .map_err(|e| HighlightError::SyntaxLoad(e.to_string()))
}

const DRACULA: &[u8] = include_bytes!("../../../themes/dracula.tmTheme");
const ONE_DARK: &[u8] = include_bytes!("../../../themes/one-dark.tmTheme");
const NORD: &[u8] = include_bytes!("../../../themes/nord.tmTheme");
const GITHUB_LIGHT: &[u8] = include_bytes!("../../../themes/github-light.tmTheme");
const TOKYO_NIGHT: &[u8] = include_bytes!("../../../themes/tokyo-night.tmTheme");
const CATPPUCCIN_MOCHA: &[u8] = include_bytes!("../../../themes/catppuccin-mocha.tmTheme");
const MONOKAI: &[u8] = include_bytes!("../../../themes/monokai.tmTheme");

fn theme_bytes(choice: ThemeChoice) -> &'static [u8] {
  match choice {
    ThemeChoice::Dracula => DRACULA,
    ThemeChoice::OneDark => ONE_DARK,
    ThemeChoice::Nord => NORD,
    ThemeChoice::GithubLight => GITHUB_LIGHT,
    ThemeChoice::TokyoNight => TOKYO_NIGHT,
    ThemeChoice::CatppuccinMocha => CATPPUCCIN_MOCHA,
    ThemeChoice::Monokai => MONOKAI,
  }
}

fn theme_index(choice: ThemeChoice) -> usize {
  match choice {
    ThemeChoice::Dracula => 0,
    ThemeChoice::OneDark => 1,
    ThemeChoice::Nord => 2,
    ThemeChoice::GithubLight => 3,
    ThemeChoice::TokyoNight => 4,
    ThemeChoice::CatppuccinMocha => 5,
    ThemeChoice::Monokai => 6,
  }
}

fn parse_theme(choice: ThemeChoice) -> Result<Theme, HighlightError> {
  ThemeSet::load_from_reader(&mut Cursor::new(theme_bytes(choice)))
    .map_err(|e| HighlightError::ThemeLoad(choice, e.to_string()))
}

/// Returns the parsed `syntect` theme for `choice`, parsed and cached on
/// first use.
pub fn theme(choice: ThemeChoice) -> Result<&'static Theme, HighlightError> {
  static THEMES: [OnceLock<Theme>; 7] = [const { OnceLock::new() }; 7];
  if let Some(theme) = THEMES[theme_index(choice)].get() {
    return Ok(theme);
  }
  let parsed = parse_theme(choice)?;
  Ok(THEMES[theme_index(choice)].get_or_init(|| parsed))
}

/// Base colors of a theme (code-card background + default foreground).
///
/// # Example
/// ```
/// use codeframe_models::{RgbColor, ThemeChoice};
/// let palette = codeframe_highlighter::theme_palette(ThemeChoice::Dracula)?;
/// assert_eq!(palette.background, RgbColor::new(0x28, 0x2a, 0x36));
/// assert_eq!(palette.foreground, RgbColor::new(0xf8, 0xf8, 0xf2));
/// # Ok::<(), codeframe_highlighter::HighlightError>(())
/// ```
pub fn theme_palette(choice: ThemeChoice) -> Result<ThemePalette, HighlightError> {
  let theme = theme(choice)?;
  let settings = &theme.settings;
  let to_rgb = |c: syntect::highlighting::Color| RgbColor::new(c.r, c.g, c.b);
  Ok(ThemePalette {
    background: settings
      .background
      .map(to_rgb)
      .unwrap_or(RgbColor::new(0x28, 0x2a, 0x36)),
    foreground: settings
      .foreground
      .map(to_rgb)
      .unwrap_or(RgbColor::new(0xf8, 0xf8, 0xf2)),
  })
}

/// Highlight `code` as `language` under `theme_choice`, returning a flat
/// token stream.
///
/// * Newlines are preserved inside token text (`'\n'`); splitting into lines
///   is the renderer's responsibility.
/// * Adjacent ranges with identical color and style are merged into a single
///   token to keep the stream (and the eventual `fillText` calls) small.
///
/// # Example
/// ```
/// use codeframe_models::{Language, RgbColor, ThemeChoice};
/// let tokens = codeframe_highlighter::highlight("fn main() {}", Language::Rust, ThemeChoice::Dracula)?;
/// // Dracula colors keywords pink (#ff79c6).
/// let pink = RgbColor::new(0xff, 0x79, 0xc6);
/// assert!(tokens.iter().any(|t| t.text.contains("fn") && t.color == pink));
/// # Ok::<(), codeframe_highlighter::HighlightError>(())
/// ```
pub fn highlight(
  code: &str,
  language: Language,
  theme_choice: ThemeChoice,
) -> Result<Vec<Token>, HighlightError> {
  let syntax_set = syntax_set()?;
  let syntax = syntax_set
    .find_syntax_by_token(language.syntax_token())
    .ok_or(HighlightError::NoSyntax(language))?;
  let theme = theme(theme_choice)?;
  let mut highlighter = HighlightLines::new(syntax, theme);

  let mut tokens: Vec<Token> = Vec::new();
  for line in syntect::util::LinesWithEndings::from(code) {
    let ranges = highlighter
      .highlight_line(line, syntax_set)
      .map_err(|e| HighlightError::Syntect(e.to_string()))?;
    for (style, text) in ranges {
      let color = RgbColor::new(style.foreground.r, style.foreground.g, style.foreground.b);
      let font_style = FontStyle {
        bold: style.font_style.contains(SyntectFontStyle::BOLD),
        italic: style.font_style.contains(SyntectFontStyle::ITALIC),
        underline: style.font_style.contains(SyntectFontStyle::UNDERLINE),
      };
      match tokens.last_mut() {
        Some(last) if last.color == color && last.font_style == font_style => {
          last.text.push_str(text);
        }
        _ => tokens.push(Token {
          text: text.to_owned(),
          color,
          font_style,
        }),
      }
    }
  }
  Ok(tokens)
}

/// Escape `&`, `<`, and `>` for safe embedding of source text in HTML
/// (quotes are safe in element text content).
///
/// # Example
/// ```
/// assert_eq!(codeframe_highlighter::escape_html("a < b && c > d"), "a &lt; b &amp;&amp; c &gt; d");
/// ```
pub fn escape_html(text: &str) -> String {
  let mut out = String::with_capacity(text.len());
  for c in text.chars() {
    match c {
      '&' => out.push_str("&amp;"),
      '<' => out.push_str("&lt;"),
      '>' => out.push_str("&gt;"),
      _ => out.push(c),
    }
  }
  out
}

/// Render a single [`Token`] as an inline-HTML `<span>`.
///
/// # Example
/// ```
/// use codeframe_models::{FontStyle, RgbColor, Token};
/// let token = Token {
///     text: "let".to_string(),
///     color: RgbColor::new(0x98, 0xc3, 0x79),
///     font_style: FontStyle { bold: false, italic: true, underline: false },
/// };
/// let html = codeframe_highlighter::token_to_html(&token);
/// assert_eq!(html, "<span style=\"color:#98c379;font-style:italic;\">let</span>");
/// ```
pub fn token_to_html(token: &Token) -> String {
  let mut out = String::with_capacity(token.text.len() * 2 + 48);
  out.push_str("<span style=\"");
  out.push_str("color:");
  out.push_str(&token.color.to_css());
  out.push(';');
  if token.font_style.bold {
    out.push_str("font-weight:700;");
  }
  if token.font_style.italic {
    out.push_str("font-style:italic;");
  }
  if token.font_style.underline {
    out.push_str("text-decoration:underline;");
  }
  out.push('"');
  out.push('>');
  out.push_str(&escape_html(&token.text));
  out.push_str("</span>");
  out
}

/// Highlight `code` as `language` under `theme_choice` and render the token
/// stream as inline-HTML spans, suitable for the `innerHTML` of an overlay
/// `<pre>` behind a transparent-text textarea.
///
/// * Every token becomes a `<span>` with `color:` plus `font-weight` /
///   `font-style` / `text-decoration` when the theme marks them (see
///   [`token_to_html`]).
/// * Text is HTML-escaped; newlines are preserved verbatim (the overlay uses
///   `white-space: pre-wrap` so the browser lays them out like the textarea).
/// * Requires no web-sys or Leptos types - pure string building, unit-testable
///   on any host.
///
/// # Example
/// ```
/// use codeframe_models::{Language, ThemeChoice};
/// let html = codeframe_highlighter::highlight_to_html(
///     "fn main() {}",
///     Language::Rust,
///     ThemeChoice::Dracula,
/// )?;
/// assert!(html.contains("color:#ff79c6"), "expected a pink `fn` span: {html}");
/// assert!(html.starts_with("<span"));
/// # Ok::<(), codeframe_highlighter::HighlightError>(())
/// ```
pub fn highlight_to_html(
  code: &str,
  language: Language,
  theme_choice: ThemeChoice,
) -> Result<String, HighlightError> {
  let tokens = highlight(code, language, theme_choice)?;
  let mut out = String::with_capacity(code.len() * 2);
  for token in &tokens {
    out.push_str(&token_to_html(token));
  }
  Ok(out)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dracula_keywords_are_pink() {
    let tokens = highlight("fn main() {}", Language::Rust, ThemeChoice::Dracula).unwrap();
    let pink = RgbColor::new(0xff, 0x79, 0xc6);
    assert!(
      tokens
        .iter()
        .any(|t| t.text.contains("fn") && t.color == pink),
      "expected a pink `fn` keyword token, got: {tokens:?}"
    );
  }

  #[test]
  fn adjacent_tokens_have_distinct_styles() {
    let tokens = highlight("let answer = 40 + 2;", Language::Rust, ThemeChoice::OneDark).unwrap();
    assert!(tokens.len() >= 3);
    for pair in tokens.windows(2) {
      assert!(
        pair[0].color != pair[1].color || pair[0].font_style != pair[1].font_style,
        "adjacent tokens with identical style should have been merged: {pair:?}"
      );
    }
  }

  #[test]
  fn round_trips_source_text_exactly() {
    let code = "fn main() {\n\tlet s = \"hi\";\n}\n";
    let tokens = highlight(code, Language::Rust, ThemeChoice::Nord).unwrap();
    let joined: String = tokens.iter().map(|t| t.text.as_str()).collect();
    assert_eq!(joined, code);
  }

  #[test]
  fn all_languages_have_syntax_definitions() {
    for language in Language::ALL {
      highlight("test", language, ThemeChoice::Dracula)
        .unwrap_or_else(|e| panic!("{language:?}: {e}"));
    }
  }

  #[test]
  fn all_themes_parse_with_expected_backgrounds() {
    let cases = [
      (ThemeChoice::Dracula, RgbColor::new(0x28, 0x2a, 0x36)),
      (ThemeChoice::OneDark, RgbColor::new(0x28, 0x2c, 0x34)),
      (ThemeChoice::Nord, RgbColor::new(0x2e, 0x34, 0x40)),
      (ThemeChoice::GithubLight, RgbColor::new(0xff, 0xff, 0xff)),
      (ThemeChoice::TokyoNight, RgbColor::new(0x1a, 0x1b, 0x26)),
      (
        ThemeChoice::CatppuccinMocha,
        RgbColor::new(0x1e, 0x1e, 0x2e),
      ),
      (ThemeChoice::Monokai, RgbColor::new(0x24, 0x24, 0x24)),
    ];
    for (choice, expected_bg) in cases {
      let palette = theme_palette(choice).unwrap();
      assert_eq!(palette.background, expected_bg, "{choice:?}");
    }
  }

  #[test]
  fn empty_input_produces_no_tokens() {
    let tokens = highlight("", Language::Rust, ThemeChoice::Dracula).unwrap();
    assert!(tokens.is_empty());
  }

  #[test]
  fn typescript_wrapper_grammar_highlights_ts_keywords() {
    let tokens = highlight(
      "interface User { name: string }",
      Language::TypeScript,
      ThemeChoice::Dracula,
    )
    .unwrap();
    let pink = RgbColor::new(0xff, 0x79, 0xc6);
    assert!(
      tokens
        .iter()
        .any(|t| t.text.contains("interface") && t.color == pink),
      "expected `interface` as a keyword token, got: {tokens:?}"
    );
  }

  #[test]
  fn escape_html_escapes_ampersands_and_angle_brackets() {
    assert_eq!(
      escape_html("a < b && c > d"),
      "a &lt; b &amp;&amp; c &gt; d"
    );
    assert_eq!(escape_html("plain text"), "plain text");
    assert_eq!(escape_html(""), "");
  }

  #[test]
  fn highlight_to_html_escapes_source_and_styles_keywords() {
    let html = highlight_to_html(
      "fn main() { let x = \"<b>&\"; }",
      Language::Rust,
      ThemeChoice::Dracula,
    )
    .unwrap();
    assert!(
      html.contains("&lt;b&gt;&amp;"),
      "source must be escaped: {html}"
    );
    assert!(
      html.contains("color:#ff79c6"),
      "expected pink `fn` span: {html}"
    );
    assert!(html.starts_with("<span") && html.ends_with("</span>"));
  }

  #[test]
  fn token_to_html_emits_theme_styles() {
    let bold = Token {
      text: "B".into(),
      color: RgbColor::new(0xff, 0x00, 0x00),
      font_style: FontStyle {
        bold: true,
        italic: false,
        underline: false,
      },
    };
    assert_eq!(
      token_to_html(&bold),
      "<span style=\"color:#ff0000;font-weight:700;\">B</span>"
    );
    let italic = Token {
      text: "i".into(),
      color: RgbColor::new(0x00, 0xff, 0x00),
      font_style: FontStyle {
        bold: false,
        italic: true,
        underline: false,
      },
    };
    assert_eq!(
      token_to_html(&italic),
      "<span style=\"color:#00ff00;font-style:italic;\">i</span>"
    );
    let underlined = Token {
      text: "u".into(),
      color: RgbColor::new(0x00, 0x00, 0xff),
      font_style: FontStyle {
        bold: false,
        italic: false,
        underline: true,
      },
    };
    assert_eq!(
      token_to_html(&underlined),
      "<span style=\"color:#0000ff;text-decoration:underline;\">u</span>"
    );
    let plain = Token {
      text: "p".into(),
      color: RgbColor::new(0xaa, 0xbb, 0xcc),
      font_style: FontStyle::default(),
    };
    assert_eq!(
      token_to_html(&plain),
      "<span style=\"color:#aabbcc;\">p</span>"
    );
  }

  #[test]
  fn token_to_html_escapes_angle_brackets_in_text() {
    let token = Token {
      text: "<tag>&".into(),
      color: RgbColor::new(0xaa, 0xbb, 0xcc),
      font_style: FontStyle::default(),
    };
    assert_eq!(
      token_to_html(&token),
      "<span style=\"color:#aabbcc;\">&lt;tag&gt;&amp;</span>"
    );
  }

  #[test]
  fn highlight_to_html_preserves_newlines_and_round_trips_source() {
    let code = "fn main() {\n\tlet s = \"hi\";\n}\n";
    let html = highlight_to_html(code, Language::Rust, ThemeChoice::Nord).unwrap();
    assert!(html.contains('\n'));
    assert_eq!(
      html.matches("<span").count(),
      html.matches("</span>").count()
    );
    let mut reconstructed = String::new();
    for part in html.split("</span>") {
      let Some(inner) = part.split_once('>') else {
        continue;
      };
      reconstructed.push_str(inner.1);
    }
    assert_eq!(reconstructed, code, "html must round-trip source text");
  }

  #[test]
  fn highlight_to_html_empty_input_is_empty() {
    assert_eq!(
      highlight_to_html("", Language::Rust, ThemeChoice::Dracula).unwrap(),
      ""
    );
  }
}
