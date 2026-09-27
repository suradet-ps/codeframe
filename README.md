# CodeFrame

[![CI](https://github.com/suradet-ps/codeframe/actions/workflows/ci.yml/badge.svg)](https://github.com/suradet-ps/codeframe/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust: stable](https://img.shields.io/badge/rust-stable-orange.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Leptos v0.8](https://img.shields.io/badge/Leptos-v0.8-blue.svg)](https://leptos.dev)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/suradet-ps/codeframe/issues)

---

## ◆ PULSE

The pixel is yours. CodeFrame turns source code into print-grade PNG,
entirely inside your browser - no server, no tracking, no watermarks.
Export resolution is not capped by your screen's `devicePixelRatio`:
12x is real 12x, drawn fresh on an off-screen canvas, never stretched
after the fact.

| 15 langs ▣ | 7 themes ▣ | 3 fonts ▣ | 12x max ▣ | PNG + SVG ▣ |
|---|---|---|---|---|

*v0.7.0 is forged. v1.0 is being tempered: reproducible builds, branch
protection, and a privacy statement that is already true.*

> Built with Leptos v0.8 (CSR/WASM), highlighted by `syntect`, drawn by
> Canvas2D via `web-sys` - nothing leaves the browser, no server in the path.
>
> **suradet-ps**, artifact keeper

---

## ◆ IGNITION

One sequence. Nothing more.

```
⟫ rustup target add wasm32-unknown-unknown
⟫ cargo install trunk
⟫ trunk serve
```

Open [http://localhost:8080](http://localhost:8080). Hot-reload is on.

```
⟫ trunk build --release
```

The artifact lands in `dist/` - deployable to Vercel, Cloudflare Pages,
GitHub Pages, or a thumb drive.

<details>
<summary>Prerequisites</summary>

- [Rust](https://rustup.rs/) (stable toolchain, pinned in `rust-toolchain.toml`)
- [Trunk](https://trunkrs.dev/) - the bundler, installed above

</details>

---

## ◆ ANATOMY

Four crates, one boundary that never bends: `app` knows Leptos; the rest
do not.

- **Highlights** - `highlighter` wraps `syntect` and emits a
  framework-agnostic token stream. 15 languages out of the box; extra
  grammars are vendored under `syntaxes/`.
- **Draws** - `renderer` turns tokens into pixels on a Canvas2D context,
  or token-accurate SVG through the same layout engine. No Leptos knows
  its name. No DOM screenshots, ever - that is the architectural line.
- **Wires** - `app` is the only crate that knows Leptos: signals,
  components, event handlers, and the `Ctrl/Cmd+Enter` export ritual.
- **Speaks** - `models` is the shared vocabulary - `Theme`, `Token`,
  `Language`, `ExportOptions` - with no web-sys dependency and no opinion
  about pixels.
- **Guards** - fonts wait for `document.fonts.ready` before the first
  draw; a generation counter retires stale preview renders; every crate
  carries `#![deny(unsafe_code)]`.

The separation is enforced so `highlighter` and `renderer` can be
unit-tested without a WASM runtime - the tests run anywhere, the same way
the export does.

---

## ◆ RITUALS

**The core ceremony** - open, paste, tweak, export:

1. Paste or type source code. Tab inserts spaces, never traps focus.
2. Choose language, theme, font, and scale. Every knob is a signal; the
   preview answers instantly.
3. Press `Ctrl/Cmd+Enter`. A fresh off-screen canvas renders the export at
   your chosen scale - 1x for drafts, 12x for the print shop.
4. The image is yours. Download PNG, copy to clipboard, or export SVG.

**The ceremony of scale** - the preview is capped at
`min(devicePixelRatio, 2.0)` to stay fluid; the export is uncapped.
Canvas pixel size is logical size times scale, computed up front - an
image is never upscaled after the fact and never lied to about its
resolution.

**The ceremony of honesty** - ligature fonts (Fira Code, JetBrains Mono,
Cascadia Code) cannot shape ligatures through Canvas2D `fillText()`. The
UI says so out loud instead of shipping a silent surprise.

---

## ◆ ECHOES

**Where this artifact is heading**

```
v0.2-v0.9 ▸ foundation, identity, export, a11y, PWA, budgets, security ▸ sealed
v1.0     ▸ reproducible build, branch protection, privacy statement ▸ being tempered
after    ▸ theme editor, snippet library, batch export, custom fonts ▸ beyond the ridge
```

**Raising the artifact** - the path is written in `docs/ROADMAP.md`, the
boundary rules in `docs/AGENTS.md`, the checks in `docs/CONTRIBUTING.md`.
The security posture lives in `docs/SECURITY.md`; the visual tokens in
`docs/DESIGN.md`. CI runs 10 jobs per commit: fmt, clippy, tests, audit,
deny, hex-audit, CSP verification, and the performance budgets - WASM
gzipped stays under 1200 KB, total bundle under 1350 KB.

**Status** - every commit is judged by [the same 10 gates](.github/workflows).

---

```
  ─────────────────────────────────────────
   No server between you and the pixel.
   No screen between the pixel and the print.
  ─────────────────────────────────────────
```

CodeFrame is released under the [MIT License](LICENSE).