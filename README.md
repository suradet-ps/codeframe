# CodeShot

**Code → PNG, entirely in your browser.**

No server. No tracking. No watermarks.

[![rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust)](https://www.rust-lang.org/)
[![leptos](https://img.shields.io/badge/Leptos-0.8-000000?style=flat)](https://leptos.dev/)
[![wasm](https://img.shields.io/badge/WebAssembly-654FF0?style=flat&logo=webassembly)](https://webassembly.org/)
[![license](https://img.shields.io/badge/License-MIT-000000?style=flat)](LICENSE)

---

## What is CodeShot?

CodeShot converts source code into high-resolution PNG images, running entirely in the browser via WebAssembly. There is no backend — every highlight, every pixel, every export happens locally on your machine.

Built as a lightweight alternative to carbon.now.sh and ray.so, with one differentiator: **export resolution is not capped by your screen's devicePixelRatio**. You can export at 4x, 8x, or even 12x scale for print-quality output.

## Features

- **Browser-native rendering** — syntax highlighting via `syntect`, canvas drawing via `web-sys`. No DOM screenshots, no html2canvas.
- **High-resolution export** — user-selectable scale up to 12x. Canvas pixel size = logical size × scale, not upscaled after the fact.
- **15 languages** — Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, HTML, CSS, JSON, YAML, TOML, Bash, SQL.
- **4 themes** — Dracula, One Dark, Nord, GitHub Light.
- **3 monospace fonts** — JetBrains Mono, Fira Code, Cascadia Code (all bundled as woff2/ttf).
- **Live preview** — Reactivity-driven canvas updates as you type, with a capped preview scale for performance.
- **Zero dependencies at runtime** — static WASM, no server required.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Trunk](https://trunkrs.dev/) (`cargo install trunk`)
- wasm32 target: `rustup target add wasm32-unknown-unknown`

### Development

```sh
trunk serve
```

Opens at `http://localhost:8080` with hot-reload on file changes.

### Production Build

```sh
trunk build --release
```

Outputs a static site to `dist/` — deployable to Cloudflare Pages, Vercel, GitHub Pages, or any static host.

## Project Structure

```
codeshot/
├── crates/
│   ├── app/          # Leptos CSR UI — components, signals, event handlers
│   ├── highlighter/  # syntect wrapper, framework-agnostic token stream
│   ├── renderer/     # Canvas2D drawing logic, no Leptos dependency
│   └── models/       # Shared types: Theme, ExportOptions, Token, Language
├── fonts/            # Bundled monospace fonts (woff2/ttf)
├── themes/           # Bundled .tmTheme files (Dracula, One Dark, Nord, GitHub Light)
├── syntaxes/         # Extra .sublime-syntax grammars (TypeScript, TOML)
├── style.css         # UI design tokens and component styles
├── index.html        # Trunk entry point
└── Trunk.toml        # Trunk configuration
```

## Available Scripts

| Command | Description |
|---|---|
| `trunk serve` | Dev server with hot-reload |
| `trunk build --release` | Optimized static build to `dist/` |
| `cargo test --workspace` | Unit tests + doctests |
| `cargo clippy --workspace --all-targets -- -D warnings` | Lint check |
| `cargo fmt --all --check` | Format check |

## Architecture

CodeShot follows a workspace-based architecture with clear crate boundaries:

- **`models`** — Shared vocabulary (Token, ExportOptions, Language, Theme). No web-sys or Leptos dependency.
- **`highlighter`** — Takes source code + language, returns a colored token stream. Framework-agnostic.
- **`renderer`** — Takes tokens + export options, draws onto a `CanvasRenderingContext2d`. No Leptos dependency.
- **`app`** — The only crate that knows about Leptos. Wires signals, components, and event handlers.

This separation allows `renderer` and `highlighter` to be unit-tested without a WASM runtime.

## Design System

CodeShot uses an editorial monochrome design system inspired by austere, restraint-first aesthetics:

- Monochrome palette with five tiers of grey for the entire UI
- Hairline dividers instead of thick borders or shadows
- Tonal surface shifts for depth (no drop shadows on chrome)
- Black pill buttons for primary actions (`border-radius: 9999px`)
- 8px spacing grid with consistent token scale

See [DESIGN.md](DESIGN.md) for the full specification.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for dev setup, CI checks, and crate boundary rules.

## Security

See [SECURITY.md](SECURITY.md) for the security posture, data flow, and vulnerability reporting.

## Known Limitations

- **Font ligatures** — Canvas2D `fillText()` does not support ligature shaping. Sequences like `!=` or `=>` in ligature fonts (Fira Code, JetBrains Mono, Cascadia Code) render as separate glyphs. A warning is shown in the UI when a ligature font is selected.
- **No SVG export** in v1 — the token stream is designed to support this in the future.
- **No server-side rendering** — export is purely client-side via `canvas.toBlob()`.

## License

This project is licensed under the [MIT License](LICENSE).

You are free to use, modify, and distribute this software for personal or commercial purposes, provided that the original copyright notice and this permission notice are included in all copies or substantial portions of the software. See the full license text for details.
