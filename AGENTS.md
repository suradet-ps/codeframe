# AGENTS.md - CodeShot (Code-to-PNG Web App)

> Architecture document for AI coding agents and humans, written before any implementation begins.
> Read this file in full before touching any code.

---

## 1. Project Overview

**Project name (working title):** CodeShot
**Goal:** A web app that converts source code into high-resolution PNG images, in the style of carbon.now.sh / ray.so
**Core selling point:** The highest possible export resolution achievable in-browser (not a DOM screenshot).
**Deployment target:** Static site (CSR/WASM) - can be hosted on Cloudflare Pages / Vercel / GitHub Pages, no backend required.

### Non-goals for v1
- No user accounts / no cloud save
- No server-side rendering or headless-browser export (ruled out entirely for the first version)
- No SVG export in v1 (design the token stream to be generic enough to add this later, but don't implement it now)

---

## 2. Tech Stack

| Layer | Technology | Rationale |
|---|---|---|
| UI Framework | Leptos v0.8 (CSR) | Team's preferred stack, fine-grained reactivity, compiles to WASM |
| Syntax highlighting | `syntect` (feature `default-fancy`) | `onig`'s C bindings don't compile to wasm32; must use `fancy-regex` backend instead |
| Canvas drawing | `web-sys` + `wasm-bindgen`, binding Canvas2D API directly | Full pixel-level control, no reliance on DOM screenshotting |
| Styling (for UI controls, not the output image) | ~~Tailwind CSS or UnoCSS~~ → **hand-written CSS** (decision changed during M0: avoids a Node/toolchain dependency; `style.css` is small and the canvas output is unaffected either way). If the file grows unwieldy, revisit Tailwind via Trunk's `rel="tailwind-css"` pipeline. |
| Icons | lucide (inline SVG or a Leptos-compatible crate) | Consistent with `lucide-vue-next` used in other projects |
| Build tool | Trunk | Standard tooling for Leptos CSR |
| Font loading | Web Font Loading API (`document.fonts.ready`) via `web-sys` | Prevents drawing with a fallback font before the real font has finished loading |

**Explicitly forbidden:** `html2canvas`, `dom-to-image`, or any DOM-screenshot-style library. This is a hard architectural constraint for this project - resolution would be capped by the browser's `devicePixelRatio`, and font rendering would not match what was intended.

---

## 3. Architecture: Cargo Workspace (lightweight hexagonal / "ports-lite")

```
codeshot/
├── Cargo.toml                 (workspace root, virtual manifest)
├── Trunk.toml                 # REQUIRED - see note below
├── index.html                 # Trunk entry; wires `rel="rust"` to crates/app/Cargo.toml
├── style.css                  # UI styles + @font-face for bundled fonts
├── crates/
│   ├── app/                   # Leptos CSR UI - components, state, event handlers
│   ├── highlighter/           # wraps syntect, exposes a framework-agnostic token stream
│   ├── renderer/               # pure canvas-drawing logic, knows nothing about Leptos
│   └── models/                # shared types: Theme, ExportOptions, Token, Language
├── themes/                    # bundled .tmTheme files (compile-time, include_bytes!)
├── syntaxes/                  # extra .sublime-syntax grammars missing from syntect defaults
├── fonts/                     # bundled monospace fonts (JetBrains Mono, Fira Code, Cascadia Code)
├── AGENTS.md
└── DESIGN.md                  # (later) UI/UX detail
```

**`Trunk.toml` is load-bearing:** if it is absent, Trunk auto-detects the
workspace-root `Cargo.toml` as its *config source* and dies with
`could not find the root package of the target crate` (a virtual manifest has
no root package). Do not delete it.

**`syntaxes/` note:** syntect's default dump lacks TypeScript and TOML.
`TOML.sublime-syntax` is vendored verbatim from `sublimehq/Packages`.
`TypeScript.sublime-syntax` is a hand-written *wrapper* grammar (TS keywords +
`include: scope:source.js`) because the official grammar uses `extends:` and
`version: 2` branching, which syntect cannot parse. If syntect ever supports
inheritance, replace the wrapper with the official grammar.

### Crate boundary rules
- **`models`** has no dependencies beyond serde - it's the shared vocabulary.
- **`highlighter`** depends only on `models` + `syntect` - takes `&str` + `Language`, returns `Vec<Token>` (knows nothing about canvas or Leptos).
- **`renderer`** depends on `models` + `web-sys` + `wasm-bindgen` - takes `Vec<Token>` + `ExportOptions`, draws into a `CanvasRenderingContext2d` passed in by the caller (doesn't create its own canvas, doesn't know about Leptos).
- **`app`** depends on all of the above - the only place with Leptos components, signals, and event handlers.

**Why split this way:** `renderer` and `highlighter` need to be unit-testable without spinning up a full WASM test runner (except for the small handful of functions that touch web-sys directly).

---

## 4. Core Data Flow

```
User types code (textarea, Leptos signal: RwSignal<String>)
        ↓
Selects Language + Theme + ExportOptions (scale, padding, background, window-frame on/off)
        ↓
highlighter::highlight(code, language, theme) -> Vec<Token>
        ↓
renderer::draw(ctx: &CanvasRenderingContext2d, tokens: &[Token], options: &ExportOptions)
        ↓
[preview] canvas rendered live, scaled down via CSS to fit the screen
[export]  canvas.toBlob("image/png") -> Blob -> ObjectURL -> <a download>
```

### Critical resolution rules (the project's core differentiator)

1. **Canvas pixel size = logical_size × export_scale**, not logical_size followed by later upscaling.
   ```rust
   canvas.set_width((logical_width * scale) as u32);
   canvas.set_height((logical_height * scale) as u32);
   ctx.scale(scale, scale)?; // draw using normal logical coordinates from here on
   ```
2. **export_scale is user-selectable**, not capped at 2x - default options: 1x / 2x / 4x / 8x, plus a custom numeric input.
3. **Preview canvas and export canvas are separate elements.** Preview renders at a screen-friendly scale (capped at, say, 2x for performance). Export creates a brand-new canvas at full scale only when the export button is pressed (prevents UI jank while typing).
4. **Always await `document.fonts.ready` before drawing** - both preview and export must check this every time, not just on mount.
5. **Use `toBlob("image/png")`, not `toDataURL`** - especially at high scale (8x): large files make `toDataURL` waste memory because it has to base64-encode the entire buffer in memory.

---

## 5. Rendering Layer Order (bottom to top)

Always draw in this order inside `renderer::draw`:

1. Background gradient/solid color (per `ExportOptions.background`)
2. Frame: rounded rect + drop shadow (if `window_frame: true`)
3. Traffic-light dots (macOS style, if `window_frame: true`)
4. Padding area (computed from `ExportOptions.padding`)
5. Token text, drawn token-by-token with theme colors (`fillText` per token, manually advancing the x-cursor - do not rely on the browser's text wrapping)
6. Line numbers (if enabled)

**Known limitation to plan around:** Canvas2D's `fillText` **does not support font ligatures** (e.g. `!=` → `≠` in Fira Code), because ligature shaping happens in the layout engine, not in canvas. This must be surfaced in the UI (e.g. a tooltip warning when a ligature font is selected). Do not attempt to "fix" this with manual character-pair substitution via regex - it breaks on too many edge cases.

---

## 6. Core Models (types in the `models` crate)

```rust
pub struct Token {
    pub text: String,
    pub color: RgbColor,        // derived from syntect scope -> theme
    pub font_style: FontStyle,  // bold / italic / underline (from theme)
}

pub struct ExportOptions {
    pub scale: f64,              // 1.0, 2.0, 4.0, 8.0, or custom
    pub padding: f64,             // px, at 1x scale
    pub background: Background,   // Solid(RgbColor) | Gradient(Vec<RgbColor>)
    pub window_frame: bool,
    pub line_numbers: bool,
    pub font_family: FontChoice,  // enum of bundled fonts
    pub font_size: f64,
    pub line_height: f64,
    pub corner_radius: f64,
}

pub enum Language { Rust, Python, JavaScript, TypeScript, /* ... */ }
```

**Rule:** Never introduce `web_sys::CanvasRenderingContext2d` or any Leptos type into `models` - this crate must compile even without the `wasm32` target.

---

## 7. Coding Standards (aligned with the team's AGENTS-RUST.md baseline)

- `#![deny(unsafe_code)]` in every crate, except where truly necessary - any exception must have a comment explaining why.
- Every public function in `renderer` and `highlighter` needs a doc comment with a usage example.
- Error handling: use `thiserror` for each crate's error type; no `unwrap()` on production code paths (except `main.rs` bootstrap).
- Clippy: run `cargo clippy --all-targets --all-features -- -D warnings` before every commit.
- Formatting: `cargo fmt --check` must always pass.
- Testing: `renderer` and `highlighter` must have unit tests that don't require a WASM runtime for any logic that doesn't directly call web-sys (e.g. token color mapping, layout calculations).

---

## 8. Milestones (recommended implementation order)

1. **M0** - Workspace scaffold: 4 empty crates compiling, `models` has all core types defined. ✅
2. **M1** - `highlighter`: takes a code string, returns correct `Vec<Token>` (unit test against a single theme first). ✅
3. **M2** - `renderer`: draws tokens onto a canvas at 1x scale (no frame/background yet). ✅
4. **M3** - `app`: minimal Leptos UI - textarea → live preview canvas. ✅
5. **M4** - Add background, window frame, padding, line numbers. ✅
6. **M5** - Export flow: export button, separate high-scale canvas, `toBlob` → download. ✅
7. **M6** - Font-loading guard (`document.fonts.ready`) + complete theme/font selector UI. ✅
8. **M7** - Polish: ligature warning, custom scale input, responsive preview. ✅

Status: **v1 complete (M0–M7).** Verified end-to-end in headless Chrome (WASM
boot, live re-render on settings change, line-number gutter, theme switching,
2x and 4x PNG export via `toBlob` → download).

---

## 9. Open Questions to Resolve Before M3 - RESOLVED

- [x] How many fonts to bundle in v1? **3: JetBrains Mono, Fira Code, Cascadia Code** (woff2/ttf in `fonts/`, see `fonts/README.md` for sources & licenses). All three have ligatures → the ligature warning shows for any selection.
- [x] How many themes to bundle? **4: Dracula, One Dark, GitHub Light, Nord** (hand-written `.tmTheme` files in `themes/` using the official palettes, embedded via `include_bytes!` and parsed lazily).
- [x] What scale cap should the preview canvas use to avoid jank while typing? **2x** (`PREVIEW_SCALE_CAP` in `crates/app/src/preview.rs`, clamped against `devicePixelRatio`).

---

## 10. Dev Commands

- `trunk serve` - dev server on <http://localhost:8080> (rebuilds on change)
- `trunk build --release` - optimized static site into `dist/` (wasm-opt `-Oz`)
- `cargo test --workspace` - unit tests + doctests (no WASM runtime needed)
- `cargo clippy --workspace --all-targets -- -D warnings` / `cargo fmt --all --check`
- `cargo check -p codeshot-app --target wasm32-unknown-unknown` - fast WASM compile check
