//! Pure layout math - no browser APIs, fully unit-testable on any host.

use codeshot_models::{ExportOptions, Token};

/// Tab stops are expanded to this many spaces before measuring and drawing
/// (Canvas2D text has no tab stops).
pub const TAB_WIDTH: usize = 4;

/// Inner padding between the code-card edge and the text (logical px).
pub const INNER_PADDING: f64 = 16.0;

/// Height of the macOS-style window header bar (logical px).
pub const FRAME_HEADER_HEIGHT: f64 = 40.0;

/// Radius of each traffic-light dot (logical px).
pub const TRAFFIC_LIGHT_RADIUS: f64 = 6.0;

/// Horizontal pitch between traffic-light dot centers (logical px).
pub const TRAFFIC_LIGHT_PITCH: f64 = 22.0;

/// X offset of the first traffic-light dot center from the card edge.
pub const TRAFFIC_LIGHT_OFFSET_X: f64 = 22.0;

/// Gap between the line-number gutter and the code, in character cells.
pub const GUTTER_GAP_CELLS: f64 = 1.5;

/// Expand `'\t'` to spaces.
///
/// # Example
/// ```
/// assert_eq!(codeshot_renderer::layout::expand_tabs("a\tb", 4), "a    b");
/// ```
pub fn expand_tabs(text: &str, tab_width: usize) -> String {
  text.replace('\t', &" ".repeat(tab_width))
}

/// Split a flat token stream on `'\n'`, preserving empty lines and expanding
/// tabs. Always returns at least one (possibly empty) line.
///
/// # Example
/// ```
/// use codeshot_models::{FontStyle, RgbColor, Token};
/// let color = RgbColor::new(1, 2, 3);
/// let tokens = vec![
///     Token { text: "a\nb".into(), color, font_style: FontStyle::default() },
///     Token { text: "\nc".into(), color, font_style: FontStyle::default() },
/// ];
/// let lines = codeshot_renderer::layout::split_tokens_into_lines(&tokens);
/// assert_eq!(lines.len(), 3);
/// assert_eq!(lines[0][0].text, "a");
/// assert_eq!(lines[1][0].text, "b");
/// assert_eq!(lines[2][0].text, "c");
/// ```
pub fn split_tokens_into_lines(tokens: &[Token]) -> Vec<Vec<Token>> {
  let mut lines: Vec<Vec<Token>> = vec![Vec::new()];
  for token in tokens {
    for (part_index, part) in token.text.split('\n').enumerate() {
      if part_index > 0 {
        lines.push(Vec::new());
      }
      if !part.is_empty() {
        // `lines` is never empty: it starts with one line and only grows.
        if let Some(line) = lines.last_mut() {
          line.push(Token {
            text: expand_tabs(part, TAB_WIDTH),
            color: token.color,
            font_style: token.font_style,
          });
        }
      }
    }
  }
  lines
}

/// Number of character cells needed for the widest line number.
pub fn gutter_cells(line_count: usize) -> usize {
  line_count.max(1).to_string().len()
}

/// Pixel width of the line-number gutter; `0.0` when line numbers are off.
pub fn gutter_width_px(options: &ExportOptions, line_count: usize, char_width: f64) -> f64 {
  if options.line_numbers {
    (gutter_cells(line_count) as f64 + GUTTER_GAP_CELLS) * char_width
  } else {
    0.0
  }
}

/// Fully-resolved geometry for one render pass, in logical (1x) pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
  /// Logical canvas size; multiply by `ExportOptions::scale` for device px.
  pub canvas_width: f64,
  pub canvas_height: f64,
  /// Code-card rectangle (the rounded rect holding the code).
  pub card_x: f64,
  pub card_y: f64,
  pub card_width: f64,
  pub card_height: f64,
  /// Window-frame header height; `0.0` when the frame is disabled.
  pub header_height: f64,
  /// Top-left of the first character cell of line 0 (drawn with
  /// `textBaseline = "top"`).
  pub code_origin_x: f64,
  pub code_origin_y: f64,
  /// Right edge x for right-aligned line numbers.
  pub gutter_right_x: f64,
  /// `font_size * line_height`.
  pub line_height_px: f64,
  pub line_count: usize,
}

/// Compute geometry from measured text dimensions.
///
/// `max_line_width_px` and `char_width` come from Canvas2D measurement (see
/// the [`crate::canvas`] module); everything else derives from `options`.
pub fn compute_layout(
  options: &ExportOptions,
  line_count: usize,
  max_line_width_px: f64,
  char_width: f64,
) -> Layout {
  let line_count = line_count.max(1);
  let line_height_px = options.font_size * options.line_height;
  let code_height = line_count as f64 * line_height_px;
  let gutter = gutter_width_px(options, line_count, char_width);
  let header_height = if options.window_frame {
    FRAME_HEADER_HEIGHT
  } else {
    0.0
  };

  let card_width = gutter + max_line_width_px + 2.0 * INNER_PADDING;
  let card_height = header_height + code_height + 2.0 * INNER_PADDING;

  Layout {
    canvas_width: options.padding + card_width + options.padding,
    canvas_height: options.padding + card_height + options.padding,
    card_x: options.padding,
    card_y: options.padding,
    card_width,
    card_height,
    header_height,
    code_origin_x: options.padding + INNER_PADDING + gutter,
    code_origin_y: options.padding + header_height + INNER_PADDING,
    gutter_right_x: options.padding + INNER_PADDING + gutter - 0.5 * char_width,
    line_height_px,
    line_count,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn expand_tabs_replaces_with_spaces() {
    assert_eq!(expand_tabs("\tlet", 4), "    let");
    assert_eq!(expand_tabs("no tabs", 4), "no tabs");
  }

  #[test]
  fn split_handles_empty_input() {
    let lines = split_tokens_into_lines(&[]);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].is_empty());
  }

  #[test]
  fn split_preserves_empty_lines_and_expands_tabs() {
    use codeshot_models::{FontStyle, RgbColor};
    let color = RgbColor::new(0, 0, 0);
    let tokens = vec![Token {
      text: "a\n\n\tb".to_string(),
      color,
      font_style: FontStyle::default(),
    }];
    let lines = split_tokens_into_lines(&tokens);
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0][0].text, "a");
    assert!(lines[1].is_empty());
    assert_eq!(lines[2][0].text, "    b");
  }

  #[test]
  fn gutter_cells_counts_digits() {
    assert_eq!(gutter_cells(1), 1);
    assert_eq!(gutter_cells(9), 1);
    assert_eq!(gutter_cells(10), 2);
    assert_eq!(gutter_cells(999), 3);
    assert_eq!(gutter_cells(0), 1);
  }

  #[test]
  fn gutter_width_respects_line_numbers_toggle() {
    let off = ExportOptions {
      line_numbers: false,
      ..Default::default()
    };
    assert_eq!(gutter_width_px(&off, 100, 8.0), 0.0);
    let on = ExportOptions {
      line_numbers: true,
      ..Default::default()
    };
    // (3 digits + 1.5 gap) * 8px
    assert_eq!(gutter_width_px(&on, 100, 8.0), 4.5 * 8.0);
  }

  #[test]
  fn layout_with_frame_and_padding() {
    let options = ExportOptions {
      padding: 48.0,
      font_size: 14.0,
      line_height: 1.5,
      window_frame: true,
      line_numbers: false,
      ..Default::default()
    };

    let layout = compute_layout(&options, 3, 100.0, 8.0);

    assert_eq!(layout.line_height_px, 21.0);
    assert_eq!(layout.header_height, FRAME_HEADER_HEIGHT);
    // card: 100 wide + 2*16 inner; 3*21 tall + 40 header + 2*16 inner
    assert_eq!(layout.card_width, 100.0 + 32.0);
    assert_eq!(layout.card_height, 40.0 + 63.0 + 32.0);
    assert_eq!(layout.canvas_width, 48.0 * 2.0 + 132.0);
    assert_eq!(layout.canvas_height, 48.0 * 2.0 + 135.0);
    assert_eq!(layout.code_origin_x, 48.0 + 16.0);
    assert_eq!(layout.code_origin_y, 48.0 + 40.0 + 16.0);
  }

  #[test]
  fn layout_without_frame_has_no_header() {
    let options = ExportOptions {
      window_frame: false,
      ..Default::default()
    };
    let layout = compute_layout(&options, 1, 50.0, 8.0);
    assert_eq!(layout.header_height, 0.0);
    assert_eq!(layout.code_origin_y, options.padding + INNER_PADDING);
  }

  #[test]
  fn layout_with_line_numbers_shifts_code_origin() {
    let options = ExportOptions {
      window_frame: false,
      line_numbers: true,
      ..Default::default()
    };
    let char_width = 8.0;
    let layout = compute_layout(&options, 12, 100.0, char_width);
    let gutter = (2.0 + GUTTER_GAP_CELLS) * char_width;
    assert_eq!(
      layout.code_origin_x,
      options.padding + INNER_PADDING + gutter
    );
    assert_eq!(
      layout.gutter_right_x,
      options.padding + INNER_PADDING + gutter - 0.5 * char_width
    );
  }
}
