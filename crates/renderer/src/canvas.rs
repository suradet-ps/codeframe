//! Canvas2D drawing. Every function takes the canvas/context from the
//! caller - this module never creates DOM elements itself and knows nothing
//! about Leptos.
//!
//! Drawing follows the layer order from AGENTS.md §5: background → card
//! (rounded rect, optional drop shadow) → traffic lights → tokens →
//! line numbers.

use codeframe_models::{Background, ExportOptions, FontStyle, ThemePalette, Token};
use thiserror::Error;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::layout::{
  compute_layout, split_tokens_into_lines, Layout, TRAFFIC_LIGHT_OFFSET_X, TRAFFIC_LIGHT_PITCH,
  TRAFFIC_LIGHT_RADIUS,
};

/// Conservative maximum canvas dimension in *device* pixels. Safari caps
/// canvases at 16384px per side; staying under it keeps exports portable.
pub const MAX_CANVAS_DIMENSION: f64 = 16384.0;

/// macOS traffic-light colors (close, minimize, zoom).
const TRAFFIC_LIGHT_COLORS: [&str; 3] = ["#ff5f57", "#febc2e", "#28c840"];

/// Convert a CSS angle (degrees) to canvas linear-gradient coordinates.
/// CSS convention: 0° = bottom→top, 90° = left→right, 180° = top→bottom,
/// 270° = right→left.
fn angle_to_coords(angle: f64, w: f64, h: f64) -> (f64, f64, f64, f64) {
  let rad = angle.to_radians();
  let dx = rad.sin();
  let dy = -rad.cos();
  let cx = w / 2.0;
  let cy = h / 2.0;
  (cx - dx * cx, cy - dy * cy, cx + dx * cx, cy + dy * cy)
}

/// Errors that can occur while measuring, sizing, or drawing the canvas.
#[derive(Debug, Error)]
pub enum RenderError {
  #[error("could not acquire a 2d rendering context from the canvas")]
  No2dContext,
  #[error(
    "image would be {width}x{height} device px, exceeding the {MAX_CANVAS_DIMENSION}px \
         browser limit - pick a lower export scale"
  )]
  CanvasTooLarge { width: u32, height: u32 },
  #[error("canvas API call failed: {0}")]
  Js(String),
}

impl From<wasm_bindgen::JsValue> for RenderError {
  fn from(value: wasm_bindgen::JsValue) -> Self {
    RenderError::Js(format!("{value:?}"))
  }
}

/// Build a Canvas2D `font` shorthand string for a token style.
///
/// Note: underline has no canvas equivalent and is intentionally ignored.
fn font_string(style: &FontStyle, family: &str, size_px: f64) -> String {
  let mut font = String::with_capacity(64);
  if style.italic {
    font.push_str("italic ");
  }
  font.push_str(if style.bold { "700 " } else { "400 " });
  font.push_str(&format!("{size_px}px \"{family}\", monospace"));
  font
}

fn get_2d_context(canvas: &HtmlCanvasElement) -> Result<CanvasRenderingContext2d, RenderError> {
  let context = canvas.get_context("2d")?.ok_or(RenderError::No2dContext)?;
  context
    .dyn_into::<CanvasRenderingContext2d>()
    .map_err(|_| RenderError::No2dContext)
}

/// Measure the width of one monospace character cell using a reference
/// string (averaging absorbs per-glyph rounding).
fn measure_char_width(ctx: &CanvasRenderingContext2d, family: &str, size_px: f64) -> f64 {
  ctx.set_font(&font_string(&FontStyle::default(), family, size_px));
  const REFERENCE: &str = "MMMMMMMMMMMMMMMM";
  match ctx.measure_text(REFERENCE) {
    Ok(metrics) => metrics.width() / REFERENCE.len() as f64,
    Err(_) => size_px * 0.6, // reasonable monospace fallback
  }
}

/// Advance width of one token. ASCII takes the fast path (`len * char_width`
/// - all bundled fonts are monospace); anything else is measured exactly.
///
/// Assumes the context font is already set (bold/italic do not change the
/// advance of a monospace font).
fn token_width(ctx: &CanvasRenderingContext2d, token: &Token, char_width: f64) -> f64 {
  if token.text.is_ascii() {
    token.text.len() as f64 * char_width
  } else {
    ctx
      .measure_text(&token.text)
      .map(|m| m.width())
      .unwrap_or_else(|_| token.text.chars().count() as f64 * char_width)
  }
}

/// A measured draw plan: everything [`draw_prepared`] needs, with no further
/// text measurement required.
#[derive(Debug)]
pub struct PreparedImage {
  /// Tokens split into lines (tabs expanded).
  pub lines: Vec<Vec<Token>>,
  /// Per-token advance widths, parallel to `lines`.
  widths: Vec<Vec<f64>>,
  /// Fully-resolved logical geometry.
  pub layout: Layout,
}

/// Trace a rounded-rectangle path (does not fill or stroke).
fn rounded_rect_path(
  ctx: &CanvasRenderingContext2d,
  x: f64,
  y: f64,
  width: f64,
  height: f64,
  radius: f64,
) -> Result<(), RenderError> {
  let radius = radius.clamp(0.0, width.min(height) / 2.0);
  ctx.begin_path();
  ctx.move_to(x + radius, y);
  ctx.arc_to(x + width, y, x + width, y + height, radius)?;
  ctx.arc_to(x + width, y + height, x, y + height, radius)?;
  ctx.arc_to(x, y + height, x, y, radius)?;
  ctx.arc_to(x, y, x + width, y, radius)?;
  ctx.close_path();
  Ok(())
}

/// Measure `tokens`, size `canvas` to `logical × scale`, and return a scaled
/// 2d context plus the prepared draw plan.
///
/// The canvas backing store is set to `logical_size * scale` and the context
/// is pre-scaled, so all drawing from here on uses logical coordinates
/// (AGENTS.md §4, rule 1). Note that resizing a canvas resets *all* context
/// state - [`draw_prepared`] re-applies every style it needs.
///
/// ```no_run
/// use codeframe_models::{ExportOptions, FontStyle, RgbColor, Token};
/// # fn f(canvas: web_sys::HtmlCanvasElement) -> Result<(), codeframe_renderer::RenderError> {
/// let tokens = vec![Token {
///     text: "fn main() {}".to_string(),
///     color: RgbColor::new(0xf8, 0xf8, 0xf2),
///     font_style: FontStyle::default(),
/// }];
/// let options = ExportOptions::default();
/// let (ctx, prepared) = codeframe_renderer::canvas::prepare(&canvas, &tokens, &options)?;
/// # Ok(())
/// # }
/// ```
pub fn prepare(
  canvas: &HtmlCanvasElement,
  tokens: &[Token],
  options: &ExportOptions,
) -> Result<(CanvasRenderingContext2d, PreparedImage), RenderError> {
  let ctx = get_2d_context(canvas)?;
  let family = options.font_family.css_family();
  let char_width = measure_char_width(&ctx, family, options.font_size);

  let lines = split_tokens_into_lines(tokens, options.tab_width);
  let mut widths = Vec::with_capacity(lines.len());
  let mut max_line_width = 0.0_f64;
  for line in &lines {
    let mut line_widths = Vec::with_capacity(line.len());
    let mut line_width = 0.0;
    for token in line {
      let width = token_width(&ctx, token, char_width);
      line_width += width;
      line_widths.push(width);
    }
    max_line_width = max_line_width.max(line_width);
    widths.push(line_widths);
  }

  let layout = compute_layout(options, lines.len(), max_line_width, char_width);

  // Size the backing store in device pixels (this resets all ctx state).
  let device_width = (layout.canvas_width * options.scale).round();
  let device_height = (layout.canvas_height * options.scale).round();
  if device_width > MAX_CANVAS_DIMENSION || device_height > MAX_CANVAS_DIMENSION {
    return Err(RenderError::CanvasTooLarge {
      width: device_width as u32,
      height: device_height as u32,
    });
  }
  canvas.set_width(device_width as u32);
  canvas.set_height(device_height as u32);
  ctx.scale(options.scale, options.scale)?;

  Ok((
    ctx,
    PreparedImage {
      lines,
      widths,
      layout,
    },
  ))
}

/// Draw a [`PreparedImage`] onto `ctx` in strict layer order.
///
/// `ctx` must be the context returned by [`prepare`] (already scaled); all
/// coordinates are logical pixels.
pub fn draw_prepared(
  ctx: &CanvasRenderingContext2d,
  prepared: &PreparedImage,
  palette: &ThemePalette,
  options: &ExportOptions,
) -> Result<(), RenderError> {
  let layout = &prepared.layout;
  let family = options.font_family.css_family();

  // 1. Background gradient / solid.
  match &options.background {
    Background::Solid(color) => {
      ctx.set_fill_style_str(&color.to_css());
      ctx.fill_rect(0.0, 0.0, layout.canvas_width, layout.canvas_height);
    }
    Background::LinearGradient { colors, angle } => {
      let (x1, y1, x2, y2) = angle_to_coords(*angle, layout.canvas_width, layout.canvas_height);
      let gradient = ctx.create_linear_gradient(x1, y1, x2, y2);
      let last = colors.len() - 1;
      for (index, color) in colors.iter().enumerate() {
        gradient.add_color_stop(index as f32 / last as f32, &color.to_css())?;
      }
      ctx.set_fill_style_canvas_gradient(&gradient);
      ctx.fill_rect(0.0, 0.0, layout.canvas_width, layout.canvas_height);
    }
    Background::RadialGradient { colors } => {
      let cx = layout.canvas_width / 2.0;
      let cy = layout.canvas_height / 2.0;
      let r = cx.max(cy);
      let gradient = ctx.create_radial_gradient(cx, cy, 0.0, cx, cy, r)?;
      let last = colors.len() - 1;
      for (index, color) in colors.iter().enumerate() {
        gradient.add_color_stop(index as f32 / last as f32, &color.to_css())?;
      }
      ctx.set_fill_style_canvas_gradient(&gradient);
      ctx.fill_rect(0.0, 0.0, layout.canvas_width, layout.canvas_height);
    }
  }

  // 2. Code card: rounded rect filled with the theme background, with a
  //    drop shadow when the window frame is enabled.
  ctx.save();
  rounded_rect_path(
    ctx,
    layout.card_x,
    layout.card_y,
    layout.card_width,
    layout.card_height,
    options.corner_radius,
  )?;
  if options.window_frame {
    ctx.set_shadow_color("rgba(0, 0, 0, 0.35)");
    ctx.set_shadow_blur(20.0);
    ctx.set_shadow_offset_y(10.0);
  }
  ctx.set_fill_style_str(&palette.background.to_css());
  ctx.fill();
  ctx.restore(); // resets shadow state

  // 3. Window-frame header band + traffic-light dots.
  if options.window_frame {
    ctx.save();
    rounded_rect_path(
      ctx,
      layout.card_x,
      layout.card_y,
      layout.card_width,
      layout.card_height,
      options.corner_radius,
    )?;
    ctx.clip();
    ctx.set_fill_style_str(if palette.is_dark() {
      "rgba(255, 255, 255, 0.06)"
    } else {
      "rgba(0, 0, 0, 0.05)"
    });
    ctx.fill_rect(
      layout.card_x,
      layout.card_y,
      layout.card_width,
      layout.header_height,
    );
    ctx.restore();

    let center_y = layout.card_y + layout.header_height / 2.0;
    for (index, color) in TRAFFIC_LIGHT_COLORS.iter().enumerate() {
      let center_x = layout.card_x + TRAFFIC_LIGHT_OFFSET_X + index as f64 * TRAFFIC_LIGHT_PITCH;
      ctx.begin_path();
      ctx.arc(
        center_x,
        center_y,
        TRAFFIC_LIGHT_RADIUS,
        0.0,
        std::f64::consts::TAU,
      )?;
      ctx.set_fill_style_str(color);
      ctx.fill();
    }
  }

  // 4. Token text, advancing the x cursor manually per token.
  ctx.set_text_baseline("top");
  let mut y = layout.code_origin_y;
  for (line, line_widths) in prepared.lines.iter().zip(&prepared.widths) {
    let mut x = layout.code_origin_x;
    for (token, width) in line.iter().zip(line_widths) {
      ctx.set_font(&font_string(&token.font_style, family, options.font_size));
      ctx.set_fill_style_str(&token.color.to_css());
      ctx.fill_text(&token.text, x, y)?;
      x += width;
    }
    y += layout.line_height_px;
  }

  // 5. Line numbers, right-aligned in the gutter, in a dimmed foreground.
  if options.line_numbers {
    ctx.set_font(&font_string(
      &FontStyle::default(),
      family,
      options.font_size,
    ));
    ctx.set_fill_style_str(&palette.foreground.to_css_with_alpha(0.45));
    ctx.set_text_align("right");
    let mut y = layout.code_origin_y;
    for number in 1..=layout.line_count {
      ctx.fill_text(&number.to_string(), layout.gutter_right_x, y)?;
      y += layout.line_height_px;
    }
    ctx.set_text_align("left");
  }

  Ok(())
}

/// Full pipeline: measure → size the canvas → draw. Returns the logical
/// layout (useful for CSS-sizing a preview canvas).
///
/// ```no_run
/// use codeframe_models::{ExportOptions, FontStyle, RgbColor, ThemePalette, Token};
/// # fn f(canvas: web_sys::HtmlCanvasElement) -> Result<(), codeframe_renderer::RenderError> {
/// let tokens = vec![Token {
///     text: "fn main() {}".to_string(),
///     color: RgbColor::new(0xf8, 0xf8, 0xf2),
///     font_style: FontStyle::default(),
/// }];
/// let palette = ThemePalette {
///     background: RgbColor::new(0x28, 0x2a, 0x36),
///     foreground: RgbColor::new(0xf8, 0xf8, 0xf2),
/// };
/// let layout = codeframe_renderer::canvas::render_to_canvas(
///     &canvas, &tokens, &palette, &ExportOptions::default(),
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn render_to_canvas(
  canvas: &HtmlCanvasElement,
  tokens: &[Token],
  palette: &ThemePalette,
  options: &ExportOptions,
) -> Result<Layout, RenderError> {
  let (ctx, prepared) = prepare(canvas, tokens, options)?;
  draw_prepared(&ctx, &prepared, palette, options)?;
  Ok(prepared.layout)
}

/// Gap between the two panels in split-screen mode (logical px).
const SPLIT_GAP: f64 = 6.0;

/// Fully-resolved geometry for a split-screen render.
#[derive(Clone, Debug)]
pub struct SplitLayout {
  pub left: Layout,
  pub right: Layout,
  /// Total canvas width (left.width + gap + right.width).
  pub canvas_width: f64,
  /// Total canvas height (max of both panels).
  pub canvas_height: f64,
}

/// Prepare a split-screen render: measure both panels and compute combined
/// geometry. The canvas is not touched — call [`draw_split_prepared`] after.
pub fn prepare_split(
  canvas: &HtmlCanvasElement,
  tokens_left: &[Token],
  tokens_right: &[Token],
  options: &ExportOptions,
) -> Result<(SplitLayout, PreparedImage, PreparedImage), RenderError> {
  let ctx = get_2d_context(canvas)?;
  let family = options.font_family.css_family();
  let cw = measure_char_width(&ctx, family, options.font_size);

  // Measure left panel.
  let lines_left = split_tokens_into_lines(tokens_left, options.tab_width);
  let mut widths_left = Vec::with_capacity(lines_left.len());
  let mut max_left = 0.0_f64;
  for line in &lines_left {
    let mut lw = Vec::with_capacity(line.len());
    let mut line_w = 0.0;
    for token in line {
      let w = token_width(&ctx, token, cw);
      line_w += w;
      lw.push(w);
    }
    max_left = max_left.max(line_w);
    widths_left.push(lw);
  }
  let layout_left = compute_layout(options, lines_left.len(), max_left, cw);

  // Measure right panel.
  let lines_right = split_tokens_into_lines(tokens_right, options.tab_width);
  let mut widths_right = Vec::with_capacity(lines_right.len());
  let mut max_right = 0.0_f64;
  for line in &lines_right {
    let mut lw = Vec::with_capacity(line.len());
    let mut line_w = 0.0;
    for token in line {
      let w = token_width(&ctx, token, cw);
      line_w += w;
      lw.push(w);
    }
    max_right = max_right.max(line_w);
    widths_right.push(lw);
  }
  let layout_right = compute_layout(options, lines_right.len(), max_right, cw);

  let canvas_width = layout_left.canvas_width + SPLIT_GAP + layout_right.canvas_width;
  let canvas_height = layout_left.canvas_height.max(layout_right.canvas_height);

  let split = SplitLayout {
    left: layout_left,
    right: layout_right,
    canvas_width,
    canvas_height,
  };

  // Size the backing store.
  let device_width = (canvas_width * options.scale).round();
  let device_height = (canvas_height * options.scale).round();
  if device_width > MAX_CANVAS_DIMENSION || device_height > MAX_CANVAS_DIMENSION {
    return Err(RenderError::CanvasTooLarge {
      width: device_width as u32,
      height: device_height as u32,
    });
  }
  canvas.set_width(device_width as u32);
  canvas.set_height(device_height as u32);
  ctx.scale(options.scale, options.scale)?;

  let prepared_left = PreparedImage {
    lines: lines_left,
    widths: widths_left,
    layout: layout_left,
  };
  let prepared_right = PreparedImage {
    lines: lines_right,
    widths: widths_right,
    layout: layout_right,
  };

  Ok((split, prepared_left, prepared_right))
}

/// Draw a split-screen render: background → left panel → divider → right panel.
///
/// Both `prepared_left` and `prepared_right` share the same `options` but
/// have independent token data and palettes.
pub fn draw_split_prepared(
  ctx: &CanvasRenderingContext2d,
  split: &SplitLayout,
  prepared_left: &PreparedImage,
  palette_left: &ThemePalette,
  prepared_right: &PreparedImage,
  palette_right: &ThemePalette,
  options: &ExportOptions,
) -> Result<(), RenderError> {
  // 1. Background (full canvas).
  match &options.background {
    Background::Solid(color) => {
      ctx.set_fill_style_str(&color.to_css());
      ctx.fill_rect(0.0, 0.0, split.canvas_width, split.canvas_height);
    }
    Background::LinearGradient { colors, angle } => {
      let (x1, y1, x2, y2) = angle_to_coords(*angle, split.canvas_width, split.canvas_height);
      let gradient = ctx.create_linear_gradient(x1, y1, x2, y2);
      let last = colors.len() - 1;
      for (index, color) in colors.iter().enumerate() {
        gradient.add_color_stop(index as f32 / last as f32, &color.to_css())?;
      }
      ctx.set_fill_style_canvas_gradient(&gradient);
      ctx.fill_rect(0.0, 0.0, split.canvas_width, split.canvas_height);
    }
    Background::RadialGradient { colors } => {
      let cx = split.canvas_width / 2.0;
      let cy = split.canvas_height / 2.0;
      let r = cx.max(cy);
      let gradient = ctx.create_radial_gradient(cx, cy, 0.0, cx, cy, r)?;
      let last = colors.len() - 1;
      for (index, color) in colors.iter().enumerate() {
        gradient.add_color_stop(index as f32 / last as f32, &color.to_css())?;
      }
      ctx.set_fill_style_canvas_gradient(&gradient);
      ctx.fill_rect(0.0, 0.0, split.canvas_width, split.canvas_height);
    }
  }

  // 2. Left panel (at origin).
  draw_panel(ctx, prepared_left, palette_left, options, 0.0, 0.0)?;

  // 3. Divider line (subtle vertical line between panels).
  let divider_x = split.left.canvas_width + SPLIT_GAP / 2.0;
  ctx.set_fill_style_str("rgba(128, 128, 128, 0.3)");
  ctx.fill_rect(
    divider_x,
    options.padding,
    1.0,
    split.canvas_height - 2.0 * options.padding,
  );

  // 4. Right panel (offset by left width + gap).
  let offset_x = split.left.canvas_width + SPLIT_GAP;
  draw_panel(ctx, prepared_right, palette_right, options, offset_x, 0.0)?;

  Ok(())
}

/// Draw a single code panel at the given offset. This is the core drawing
/// logic shared between single and split-screen modes.
fn draw_panel(
  ctx: &CanvasRenderingContext2d,
  prepared: &PreparedImage,
  palette: &ThemePalette,
  options: &ExportOptions,
  offset_x: f64,
  _offset_y: f64,
) -> Result<(), RenderError> {
  let layout = &prepared.layout;
  let family = options.font_family.css_family();

  // Card rounded rect with optional shadow.
  let cx = layout.card_x + offset_x;
  ctx.save();
  rounded_rect_path(
    ctx,
    cx,
    layout.card_y,
    layout.card_width,
    layout.card_height,
    options.corner_radius,
  )?;
  if options.window_frame {
    ctx.set_shadow_color("rgba(0, 0, 0, 0.35)");
    ctx.set_shadow_blur(20.0);
    ctx.set_shadow_offset_y(10.0);
  }
  ctx.set_fill_style_str(&palette.background.to_css());
  ctx.fill();
  ctx.restore();

  // Window-frame header band + traffic lights.
  if options.window_frame {
    ctx.save();
    rounded_rect_path(
      ctx,
      cx,
      layout.card_y,
      layout.card_width,
      layout.card_height,
      options.corner_radius,
    )?;
    ctx.clip();
    ctx.set_fill_style_str(if palette.is_dark() {
      "rgba(255, 255, 255, 0.06)"
    } else {
      "rgba(0, 0, 0, 0.05)"
    });
    ctx.fill_rect(cx, layout.card_y, layout.card_width, layout.header_height);
    ctx.restore();

    let center_y = layout.card_y + layout.header_height / 2.0;
    for (index, color) in TRAFFIC_LIGHT_COLORS.iter().enumerate() {
      let center_x = cx + TRAFFIC_LIGHT_OFFSET_X + index as f64 * TRAFFIC_LIGHT_PITCH;
      ctx.begin_path();
      ctx.arc(
        center_x,
        center_y,
        TRAFFIC_LIGHT_RADIUS,
        0.0,
        std::f64::consts::TAU,
      )?;
      ctx.set_fill_style_str(color);
      ctx.fill();
    }
  }

  // Token text.
  ctx.set_text_baseline("top");
  let mut y = layout.code_origin_y;
  let code_x = layout.code_origin_x + offset_x;
  for (line, line_widths) in prepared.lines.iter().zip(&prepared.widths) {
    let mut x = code_x;
    for (token, width) in line.iter().zip(line_widths) {
      ctx.set_font(&font_string(&token.font_style, family, options.font_size));
      ctx.set_fill_style_str(&token.color.to_css());
      ctx.fill_text(&token.text, x, y)?;
      x += width;
    }
    y += layout.line_height_px;
  }

  // Line numbers.
  if options.line_numbers {
    ctx.set_font(&font_string(
      &FontStyle::default(),
      family,
      options.font_size,
    ));
    ctx.set_fill_style_str(&palette.foreground.to_css_with_alpha(0.45));
    ctx.set_text_align("right");
    let mut y = layout.code_origin_y;
    let gutter_x = layout.gutter_right_x + offset_x;
    for number in 1..=layout.line_count {
      ctx.fill_text(&number.to_string(), gutter_x, y)?;
      y += layout.line_height_px;
    }
    ctx.set_text_align("left");
  }

  Ok(())
}

/// Full pipeline for split-screen: measure → size the canvas → draw both
/// panels. Returns the combined layout.
pub fn render_split_to_canvas(
  canvas: &HtmlCanvasElement,
  tokens_left: &[Token],
  palette_left: &ThemePalette,
  tokens_right: &[Token],
  palette_right: &ThemePalette,
  options: &ExportOptions,
) -> Result<SplitLayout, RenderError> {
  let (split, prepared_left, prepared_right) =
    prepare_split(canvas, tokens_left, tokens_right, options)?;
  let ctx = get_2d_context(canvas)?;
  draw_split_prepared(
    &ctx,
    &split,
    &prepared_left,
    palette_left,
    &prepared_right,
    palette_right,
    options,
  )?;
  Ok(split)
}

/// Helper for the header-band tint: dark themes get a light overlay, light
/// themes a dark one.
trait PaletteDarkness {
  fn is_dark(&self) -> bool;
}

impl PaletteDarkness for ThemePalette {
  fn is_dark(&self) -> bool {
    let ThemePalette { background, .. } = self;
    (background.r as u32 + background.g as u32 + background.b as u32) < 384
  }
}
