//! Pure SVG string generation — no browser APIs, no Leptos.
//!
//! Reuses the same [`Layout`] and token data from the canvas pipeline
//! to produce a standalone SVG string. Fonts are referenced by name
//! (system fonts or bundled via the app's `@font-face`).

use codeframe_models::{Background, ExportOptions, FontStyle, ThemePalette, Token};

use crate::layout::{
  compute_layout, split_tokens_into_lines, Layout, TRAFFIC_LIGHT_OFFSET_X, TRAFFIC_LIGHT_PITCH,
  TRAFFIC_LIGHT_RADIUS,
};

/// macOS traffic-light colors (close, minimize, zoom).
const TRAFFIC_LIGHT_COLORS: [&str; 3] = ["#ff5f57", "#febc2e", "#28c840"];

/// Convert an angle (degrees) to SVG linearGradient x1/y1/x2/y2 percentages.
/// 0° = left→right, 90° = top→bottom, etc.
fn angle_to_svg_coords(angle: f64) -> (f64, f64, f64, f64) {
  let rad = angle.to_radians();
  let cos = rad.cos();
  let sin = rad.sin();
  let x1 = 50.0 - cos * 50.0;
  let y1 = 50.0 - sin * 50.0;
  let x2 = 50.0 + cos * 50.0;
  let y2 = 50.0 + sin * 50.0;
  (x1, y1, x2, y2)
}

/// Escape special XML characters.
fn esc(s: &str) -> String {
  s.replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&#39;")
}

/// Build a CSS `font` property value for a token style.
fn font_css(style: &FontStyle, family: &str, size: f64) -> String {
  let weight = if style.bold { "700" } else { "400" };
  let size_str = format!("{size}px");
  let mut font = String::with_capacity(64);
  if style.italic {
    font.push_str("italic ");
  }
  font.push_str(weight);
  font.push(' ');
  font.push_str(&size_str);
  font.push_str(" \"");
  font.push_str(family);
  font.push_str("\", monospace");
  font
}

/// Approximate character width for a monospace font at a given size.
/// This matches the canvas renderer's heuristic (all ASCII in monospace).
fn estimate_char_width(font_size: f64) -> f64 {
  font_size * 0.602
}

/// Render code tokens into a standalone SVG string.
///
/// The returned SVG uses `width`/`height` in pixels and renders at 1x scale.
/// For higher resolution, multiply the dimensions externally.
pub fn render_svg(
  tokens: &[Token],
  palette: &ThemePalette,
  options: &ExportOptions,
) -> (String, Layout) {
  let family = options.font_family.css_family();
  let cw = estimate_char_width(options.font_size);

  let lines = split_tokens_into_lines(tokens, options.tab_width);
  // Measure max line width (ASCII fast path).
  let mut max_line_width = 0.0_f64;
  for line in &lines {
    let mut w = 0.0;
    for token in line {
      w += token.text.len() as f64 * cw;
    }
    max_line_width = max_line_width.max(w);
  }

  let layout = compute_layout(options, lines.len(), max_line_width, cw);

  let mut svg = String::with_capacity(4096);
  let w = layout.canvas_width;
  let h = layout.canvas_height;

  // --- Header ---
  svg.push_str(&format!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#,
  ));

  // --- Defs (gradient, shadow filter) ---
  svg.push_str("<defs>");
  // Drop-shadow filter for the card.
  if options.window_frame {
    svg.push_str(
      r#"<filter id="shadow" x="-10%" y="-5%" width="130%" height="130%">
        <feDropShadow dx="0" dy="10" stdDeviation="10" flood-color="rgba(0,0,0,0.35)"/>
      </filter>"#,
    );
  }
  // Background gradient defs.
  match &options.background {
    Background::LinearGradient { colors, angle } if colors.len() >= 2 => {
      let (x1, y1, x2, y2) = angle_to_svg_coords(*angle);
      svg.push_str(&format!(
        "<linearGradient id=\"bg\" x1=\"{x1:.1}%\" y1=\"{y1:.1}%\" x2=\"{x2:.1}%\" y2=\"{y2:.1}%\">"
      ));
      let last = colors.len() - 1;
      for (i, c) in colors.iter().enumerate() {
        let offset = i as f32 / last as f32 * 100.0;
        svg.push_str(&format!(
          "<stop offset=\"{offset:.0}%\" stop-color=\"{}\"/>",
          c.to_css()
        ));
      }
      svg.push_str("</linearGradient>");
    }
    Background::RadialGradient { colors } if colors.len() >= 2 => {
      svg.push_str("<radialGradient id=\"bg\" cx=\"50%\" cy=\"50%\" r=\"70%\">");
      let last = colors.len() - 1;
      for (i, c) in colors.iter().enumerate() {
        let offset = i as f32 / last as f32 * 100.0;
        svg.push_str(&format!(
          "<stop offset=\"{offset:.0}%\" stop-color=\"{}\"/>",
          c.to_css()
        ));
      }
      svg.push_str("</radialGradient>");
    }
    _ => {}
  }
  svg.push_str("</defs>");

  // --- 1. Background ---
  match &options.background {
    Background::Solid(color) => {
      svg.push_str(&format!(
        "<rect width=\"{w}\" height=\"{h}\" fill=\"{}\"/>",
        color.to_css()
      ));
    }
    Background::LinearGradient { colors, .. } => {
      if colors.len() == 1 {
        svg.push_str(&format!(
          "<rect width=\"{w}\" height=\"{h}\" fill=\"{}\"/>",
          colors[0].to_css()
        ));
      } else {
        svg.push_str(&format!(
          "<rect width=\"{w}\" height=\"{h}\" fill=\"url(#bg)\"/>"
        ));
      }
    }
    Background::RadialGradient { colors } => {
      if colors.len() == 1 {
        svg.push_str(&format!(
          "<rect width=\"{w}\" height=\"{h}\" fill=\"{}\"/>",
          colors[0].to_css()
        ));
      } else {
        svg.push_str(&format!(
          "<rect width=\"{w}\" height=\"{h}\" fill=\"url(#bg)\"/>"
        ));
      }
    }
  }

  // --- 2. Code card ---
  let rx = options.corner_radius;
  let filter = if options.window_frame {
    " filter=\"url(#shadow)\""
  } else {
    ""
  };
  svg.push_str(&format!(
    "<rect x=\"{x}\" y=\"{y}\" width=\"{cw}\" height=\"{ch}\" rx=\"{rx}\" fill=\"{fill}\"{filter}/>",
    x = layout.card_x,
    y = layout.card_y,
    cw = layout.card_width,
    ch = layout.card_height,
    fill = palette.background.to_css(),
  ));

  // --- 3. Window-frame header + traffic lights ---
  if options.window_frame {
    let header_fill = if is_dark(palette) {
      "rgba(255,255,255,0.06)"
    } else {
      "rgba(0,0,0,0.05)"
    };
    // Clip header to card shape via a simple rect (no clipping path needed —
    // the header band sits entirely within the card).
    svg.push_str(&format!(
      "<rect x=\"{x}\" y=\"{y}\" width=\"{cw}\" height=\"{hh}\" rx=\"{rx}\" fill=\"{header_fill}\"/>",
      x = layout.card_x,
      y = layout.card_y,
      cw = layout.card_width,
      hh = layout.header_height,
    ));

    let center_y = layout.card_y + layout.header_height / 2.0;
    for (i, color) in TRAFFIC_LIGHT_COLORS.iter().enumerate() {
      let cx = layout.card_x + TRAFFIC_LIGHT_OFFSET_X + i as f64 * TRAFFIC_LIGHT_PITCH;
      svg.push_str(&format!(
        "<circle cx=\"{cx}\" cy=\"{center_y}\" r=\"{r}\" fill=\"{color}\"/>",
        r = TRAFFIC_LIGHT_RADIUS,
      ));
    }
  }

  // --- 4. Token text ---
  let mut y = layout.code_origin_y;
  for line in &lines {
    let mut x = layout.code_origin_x;
    for token in line {
      let font = font_css(&token.font_style, family, options.font_size);
      let fill = token.color.to_css();
      let text = esc(&token.text);
      svg.push_str(&format!(
        "<text x=\"{x}\" y=\"{y}\" font=\"{font}\" fill=\"{fill}\" dominant-baseline=\"hanging\">{text}</text>",
      ));
      x += token.text.len() as f64 * cw;
    }
    y += layout.line_height_px;
  }

  // --- 5. Line numbers ---
  if options.line_numbers {
    let fill = palette.foreground.to_css_with_alpha(0.45);
    let font = font_css(&FontStyle::default(), family, options.font_size);
    let mut y = layout.code_origin_y;
    for number in 1..=layout.line_count {
      svg.push_str(&format!(
        "<text x=\"{x}\" y=\"{y}\" font=\"{font}\" fill=\"{fill}\" text-anchor=\"end\" dominant-baseline=\"hopping\">{number}</text>",
        x = layout.gutter_right_x,
      ));
      y += layout.line_height_px;
    }
  }

  svg.push_str("</svg>");
  (svg, layout)
}

/// Gap between panels in split-screen SVG (logical px).
const SPLIT_GAP: f64 = 6.0;

/// Render two code panels side by side as a single SVG.
pub fn render_split_svg(
  tokens_left: &[Token],
  palette_left: &ThemePalette,
  tokens_right: &[Token],
  palette_right: &ThemePalette,
  options: &ExportOptions,
) -> (String, f64, f64) {
  let family = options.font_family.css_family();
  let cw = estimate_char_width(options.font_size);

  // Measure left panel.
  let lines_left = split_tokens_into_lines(tokens_left, options.tab_width);
  let mut max_left = 0.0_f64;
  for line in &lines_left {
    let mut w = 0.0;
    for token in line {
      w += token.text.len() as f64 * cw;
    }
    max_left = max_left.max(w);
  }
  let layout_left = compute_layout(options, lines_left.len(), max_left, cw);

  // Measure right panel.
  let lines_right = split_tokens_into_lines(tokens_right, options.tab_width);
  let mut max_right = 0.0_f64;
  for line in &lines_right {
    let mut w = 0.0;
    for token in line {
      w += token.text.len() as f64 * cw;
    }
    max_right = max_right.max(w);
  }
  let layout_right = compute_layout(options, lines_right.len(), max_right, cw);

  let total_w = layout_left.canvas_width + SPLIT_GAP + layout_right.canvas_width;
  let total_h = layout_left.canvas_height.max(layout_right.canvas_height);

  let mut svg = String::with_capacity(8192);
  svg.push_str(&format!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="{total_w}" height="{total_h}" viewBox="0 0 {total_w} {total_h}">"#,
  ));

  // Defs.
  svg.push_str("<defs>");
  if options.window_frame {
    svg.push_str(
      r#"<filter id="shadow" x="-10%" y="-5%" width="130%" height="130%">
        <feDropShadow dx="0" dy="10" stdDeviation="10" flood-color="rgba(0,0,0,0.35)"/>
      </filter>"#,
    );
  }
  match &options.background {
    Background::LinearGradient { colors, angle } if colors.len() >= 2 => {
      let (x1, y1, x2, y2) = angle_to_svg_coords(*angle);
      svg.push_str(&format!(
        "<linearGradient id=\"bg\" x1=\"{x1:.1}%\" y1=\"{y1:.1}%\" x2=\"{x2:.1}%\" y2=\"{y2:.1}%\">"
      ));
      let last = colors.len() - 1;
      for (i, c) in colors.iter().enumerate() {
        let offset = i as f32 / last as f32 * 100.0;
        svg.push_str(&format!(
          "<stop offset=\"{offset:.0}%\" stop-color=\"{}\"/>",
          c.to_css()
        ));
      }
      svg.push_str("</linearGradient>");
    }
    Background::RadialGradient { colors } if colors.len() >= 2 => {
      svg.push_str("<radialGradient id=\"bg\" cx=\"50%\" cy=\"50%\" r=\"70%\">");
      let last = colors.len() - 1;
      for (i, c) in colors.iter().enumerate() {
        let offset = i as f32 / last as f32 * 100.0;
        svg.push_str(&format!(
          "<stop offset=\"{offset:.0}%\" stop-color=\"{}\"/>",
          c.to_css()
        ));
      }
      svg.push_str("</radialGradient>");
    }
    _ => {}
  }
  svg.push_str("</defs>");

  // Background.
  match &options.background {
    Background::Solid(color) => {
      svg.push_str(&format!(
        "<rect width=\"{total_w}\" height=\"{total_h}\" fill=\"{}\"/>",
        color.to_css()
      ));
    }
    Background::LinearGradient { colors, .. } => {
      if colors.len() == 1 {
        svg.push_str(&format!(
          "<rect width=\"{total_w}\" height=\"{total_h}\" fill=\"{}\"/>",
          colors[0].to_css()
        ));
      } else {
        svg.push_str("<rect width=\"{total_w}\" height=\"{total_h}\" fill=\"url(#bg)\"/>");
      }
    }
    Background::RadialGradient { colors } => {
      if colors.len() == 1 {
        svg.push_str(&format!(
          "<rect width=\"{total_w}\" height=\"{total_h}\" fill=\"{}\"/>",
          colors[0].to_css()
        ));
      } else {
        svg.push_str("<rect width=\"{total_w}\" height=\"{total_h}\" fill=\"url(#bg)\"/>");
      }
    }
  }

  // Left panel.
  render_panel_svg(
    &mut svg,
    &lines_left,
    palette_left,
    options,
    &layout_left,
    family,
    cw,
    0.0,
  );

  // Divider.
  let divider_x = layout_left.canvas_width + SPLIT_GAP / 2.0;
  svg.push_str(&format!(
    "<rect x=\"{divider_x}\" y=\"{y}\" width=\"1\" height=\"{h}\" fill=\"rgba(128,128,128,0.3)\"/>",
    y = options.padding,
    h = total_h - 2.0 * options.padding,
  ));

  // Right panel.
  let offset_x = layout_left.canvas_width + SPLIT_GAP;
  render_panel_svg(
    &mut svg,
    &lines_right,
    palette_right,
    options,
    &layout_right,
    family,
    cw,
    offset_x,
  );

  svg.push_str("</svg>");
  (svg, total_w, total_h)
}

/// Render a single panel's SVG elements at the given x offset.
#[allow(clippy::too_many_arguments)]
fn render_panel_svg(
  svg: &mut String,
  lines: &[Vec<Token>],
  palette: &ThemePalette,
  options: &ExportOptions,
  layout: &Layout,
  family: &str,
  cw: f64,
  offset_x: f64,
) {
  let rx = options.corner_radius;
  let cx = layout.card_x + offset_x;
  let filter = if options.window_frame {
    " filter=\"url(#shadow)\""
  } else {
    ""
  };

  // Card rect.
  svg.push_str(&format!(
    "<rect x=\"{cx}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" rx=\"{rx}\" fill=\"{fill}\"{filter}/>",
    y = layout.card_y,
    w = layout.card_width,
    h = layout.card_height,
    fill = palette.background.to_css(),
  ));

  // Header.
  if options.window_frame {
    let header_fill = if is_dark(palette) {
      "rgba(255,255,255,0.06)"
    } else {
      "rgba(0,0,0,0.05)"
    };
    svg.push_str(&format!(
      "<rect x=\"{cx}\" y=\"{y}\" width=\"{w}\" height=\"{hh}\" rx=\"{rx}\" fill=\"{header_fill}\"/>",
      y = layout.card_y,
      w = layout.card_width,
      hh = layout.header_height,
    ));

    let center_y = layout.card_y + layout.header_height / 2.0;
    for (i, color) in TRAFFIC_LIGHT_COLORS.iter().enumerate() {
      let tcx = cx + TRAFFIC_LIGHT_OFFSET_X + i as f64 * TRAFFIC_LIGHT_PITCH;
      svg.push_str(&format!(
        "<circle cx=\"{tcx}\" cy=\"{center_y}\" r=\"{r}\" fill=\"{color}\"/>",
        r = TRAFFIC_LIGHT_RADIUS,
      ));
    }
  }

  // Tokens.
  let mut y = layout.code_origin_y;
  let code_x = layout.code_origin_x + offset_x;
  for line in lines {
    let mut x = code_x;
    for token in line {
      let font = font_css(&token.font_style, family, options.font_size);
      let fill = token.color.to_css();
      let text = esc(&token.text);
      svg.push_str(&format!(
        "<text x=\"{x}\" y=\"{y}\" font=\"{font}\" fill=\"{fill}\" dominant-baseline=\"hanging\">{text}</text>",
      ));
      x += token.text.len() as f64 * cw;
    }
    y += layout.line_height_px;
  }

  // Line numbers.
  if options.line_numbers {
    let fill = palette.foreground.to_css_with_alpha(0.45);
    let font = font_css(&FontStyle::default(), family, options.font_size);
    let mut y = layout.code_origin_y;
    let gutter_x = layout.gutter_right_x + offset_x;
    for number in 1..=layout.line_count {
      svg.push_str(&format!(
        "<text x=\"{gutter_x}\" y=\"{y}\" font=\"{font}\" fill=\"{fill}\" text-anchor=\"end\" dominant-baseline=\"hopping\">{number}</text>",
      ));
      y += layout.line_height_px;
    }
  }
}

fn is_dark(palette: &ThemePalette) -> bool {
  (palette.background.r as u32 + palette.background.g as u32 + palette.background.b as u32) < 384
}

#[cfg(test)]
mod tests {
  use super::*;
  use codeframe_models::{FontStyle, RgbColor};

  fn sample_tokens() -> Vec<Token> {
    vec![
      Token {
        text: "fn ".to_string(),
        color: RgbColor::new(0xff, 0x79, 0xc6),
        font_style: FontStyle::default(),
      },
      Token {
        text: "main".to_string(),
        color: RgbColor::new(0xf8, 0xf8, 0xf2),
        font_style: FontStyle::default(),
      },
      Token {
        text: "() {}".to_string(),
        color: RgbColor::new(0xf8, 0xf8, 0xf2),
        font_style: FontStyle::default(),
      },
    ]
  }

  #[test]
  fn svg_starts_with_svg_tag() {
    let tokens = sample_tokens();
    let palette = ThemePalette {
      background: RgbColor::new(0x28, 0x2a, 0x36),
      foreground: RgbColor::new(0xf8, 0xf8, 0xf2),
    };
    let options = ExportOptions::default();
    let (svg, _) = render_svg(&tokens, &palette, &options);
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
  }

  #[test]
  fn svg_contains_token_text() {
    let tokens = sample_tokens();
    let palette = ThemePalette {
      background: RgbColor::new(0x28, 0x2a, 0x36),
      foreground: RgbColor::new(0xf8, 0xf8, 0xf2),
    };
    let options = ExportOptions::default();
    let (svg, _) = render_svg(&tokens, &palette, &options);
    assert!(svg.contains("fn"));
    assert!(svg.contains("main"));
  }

  #[test]
  fn svg_escapes_special_chars() {
    let tokens = vec![Token {
      text: "<script>alert(\"xss\")</script>".to_string(),
      color: RgbColor::new(0xff, 0xff, 0xff),
      font_style: FontStyle::default(),
    }];
    let palette = ThemePalette {
      background: RgbColor::new(0x00, 0x00, 0x00),
      foreground: RgbColor::new(0xff, 0xff, 0xff),
    };
    let options = ExportOptions::default();
    let (svg, _) = render_svg(&tokens, &palette, &options);
    assert!(!svg.contains("<script>"));
    assert!(svg.contains("&lt;script&gt;"));
  }

  #[test]
  fn line_numbers_appear_when_enabled() {
    let tokens = sample_tokens();
    let palette = ThemePalette {
      background: RgbColor::new(0x28, 0x2a, 0x36),
      foreground: RgbColor::new(0xf8, 0xf8, 0xf2),
    };
    let options = ExportOptions {
      line_numbers: true,
      ..Default::default()
    };
    let (svg, _) = render_svg(&tokens, &palette, &options);
    assert!(svg.contains("1"));
  }

  #[test]
  fn no_line_numbers_when_disabled() {
    let tokens = sample_tokens();
    let palette = ThemePalette {
      background: RgbColor::new(0x28, 0x2a, 0x36),
      foreground: RgbColor::new(0xf8, 0xf8, 0xf2),
    };
    let options = ExportOptions {
      line_numbers: false,
      ..Default::default()
    };
    let (svg, _) = render_svg(&tokens, &palette, &options);
    // The number "1" might appear in coordinates, but text-anchor="end" (line numbers)
    // won't be present.
    assert!(!svg.contains("text-anchor"));
  }
}
