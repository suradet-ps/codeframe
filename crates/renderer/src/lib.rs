//! Canvas-drawing logic for CodeFrame.
//!
//! Split into two layers (AGENTS.md §3):
//! * [`layout`] - pure geometry math, no browser APIs, unit-testable anywhere.
//! * [`canvas`] - Canvas2D drawing; takes the canvas from the caller, never
//!   creates DOM elements itself, and knows nothing about Leptos.
#![deny(unsafe_code)]

pub mod canvas;
pub mod layout;

pub use canvas::{draw_prepared, prepare, render_to_canvas, PreparedImage, RenderError};
pub use layout::{compute_layout, split_tokens_into_lines, Layout};
