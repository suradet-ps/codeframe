# CONTRIBUTING.md - CodeFrame

> How to set up, develop, and ship changes to CodeFrame.

---

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Trunk](https://trunkrs.dev/) (`cargo install trunk`)
- wasm32 target: `rustup target add wasm32-unknown-unknown`

---

## Development

```sh
trunk serve
```

Opens at `http://localhost:8080` with hot-reload on file changes.

---

## CI Checks

Every PR must pass these 5 jobs (defined in `.github/workflows/ci.yml`):

| Job | Command | What it catches |
|-----|---------|-----------------|
| Check | `cargo check -p codeframe-app --target wasm32-unknown-unknown` | Compile errors in the WASM target |
| Clippy | `cargo clippy --workspace --all-targets -- -D clippy::correctness -D clippy::suspicious` | Correctness and suspicious lints |
| Format | `cargo fmt --all --check` | Formatting drift |
| Test | `cargo test --lib` | Unit test failures |
| Build | `trunk build --release` | Full production build (gated on the 4 above) |

Additionally, `cargo audit` and `cargo deny` run as separate jobs to catch
advisories and license issues.

Run all checks locally before pushing:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D clippy::correctness -D clippy::suspicious
cargo test --lib
cargo check -p codeframe-app --target wasm32-unknown-unknown
trunk build --release
```

---

## Crate Boundaries

CodeFrame is a Cargo workspace with 4 crates. Each has a strict role:

```
models ──► highlighter ──► renderer
   │            │              │
   └────────────┴──────────────┘
                │
                ▼
               app
```

### `models` (zero dependencies beyond `serde`)

Shared types: `Token`, `ExportOptions`, `Language`, `ThemeChoice`,
`FontChoice`, `Background`, `RgbColor`, `FontStyle`, `ThemePalette`.

**Rule:** No `web-sys`, no `wasm-bindgen`, no Leptos. This crate must
compile on any target, including native.

### `highlighter` (depends on `models` + `syntect`)

Takes `&str` + `Language` + `ThemeChoice`, returns `Vec<Token>`.

**Rule:** No canvas APIs, no Leptos. Pure Rust logic.

### `renderer` (depends on `models` + `web-sys`)

Takes `Vec<Token>` + `ThemePalette` + `ExportOptions`, draws onto a
`CanvasRenderingContext2d` passed in by the caller.

**Rule:** Never creates its own canvas element. Never knows about Leptos.
Unit-testable for layout math (the `layout` module has no browser APIs).

### `app` (depends on all of the above + Leptos)

The only crate that knows about Leptos. Components, signals, event handlers.

**Rule:** Business logic goes in the other crates. `app` wires things together.

---

## Safety Policy

Every crate starts with:

```rust
#![deny(unsafe_code)]
```

No exceptions in production code. If you truly need `unsafe`, it must:

1. Have a doc comment explaining *why* it is necessary.
2. Be isolated to the smallest possible scope.
3. Have a test that exercises the unsafe path.
4. Be noted in the crate's `lib.rs` doc comment.

---

## Code Style

- **Formatting:** `cargo fmt --all --check` (2-space indent, 100 char max).
  See `rustfmt.toml`.
- **Lints:** `#![deny(unsafe_code)]` + `#![deny(unused_must_use)]` at crate
  root. Clippy: correctness + suspicious.
- **Error handling:** Use `thiserror` for crate error types. No `unwrap()`
  in production code paths.
- **Doc comments:** Every public function in `renderer` and `highlighter`
  needs a doc comment with a usage example.

---

## Adding a Language

1. Add a variant to `Language` in `crates/models/src/lib.rs`.
2. Add its `display_name()` and `syntax_token()` mappings.
3. Add it to `Language::ALL`.
4. If syntect has a grammar for it, it works automatically. If not (like
   TypeScript), add a `.sublime-syntax` file to `syntaxes/` and register
   it in `crates/highlighter/src/lib.rs`.

---

## Adding a Theme

1. Create a `.tmTheme` file in `themes/`.
2. Add a variant to `ThemeChoice` in `crates/models/src/lib.rs`.
3. Add its bytes to `theme_bytes()` in `crates/highlighter/src/lib.rs`.
4. Add it to `ThemeChoice::ALL`.

---

## Adding a Font

1. Add the woff2/ttf file to `fonts/`.
2. Add `@font-face` declarations to `style.css`.
3. Add a variant to `FontChoice` in `crates/models/src/lib.rs`.
4. Add it to `FontChoice::ALL`.

---

## PR Workflow

1. Create a branch from `main`.
2. Make your changes, ensuring all CI checks pass locally.
3. Open a PR against `main`.
4. CI runs automatically. All 7 jobs must pass.
5. Squash-merge (or regular merge - team preference).

---

## Project Structure

```
CodeFrame/
├── crates/
│   ├── app/            # Leptos CSR UI
│   ├── highlighter/    # syntect wrapper
│   ├── renderer/       # Canvas2D drawing
│   └── models/         # Shared types
├── fonts/              # Bundled monospace fonts
├── themes/             # Bundled .tmTheme files
├── syntaxes/           # Extra .sublime-syntax grammars
├── style.css           # UI design tokens + component styles
├── index.html          # Trunk entry point
├── Trunk.toml          # Trunk configuration
├── DESIGN.md           # Design system spec
├── SECURITY.md         # Security posture
├── CONTRIBUTING.md     # This file
└── ROADMAP.md          # Project direction
```
