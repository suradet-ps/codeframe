# ROADMAP.md — CodeShot

> Architecture document for the project's direction, written from reading its
> own code — not from assumptions. Read this file in full before planning work.

---

## North Star

**CodeShot will be the fastest, most private code-to-image tool that works
everywhere — offline, on any device, with zero data leaving the browser.**

Not a code editor. Not a snippet manager. Not a collaboration platform. A
single-purpose tool that does one thing with surgical precision: turn source
code into a beautiful, high-resolution image in under a second. The image
is yours. The code is yours. Nothing is tracked. Nothing is stored. Nothing
leaves your machine.

If you can describe CodeShot in one sentence, it should be:
> *Open. Paste. Screenshot. Done.*

---

## What CodeShot is

A **quiet, focused tool** that turns source code into beautiful PNG images,
entirely in the browser. No server. No tracking. No watermarks. You type (or
paste) code, tweak a few knobs, and get a high-resolution image ready for
slides, docs, or sharing.

**What CodeShot is not.** Not a code editor. Not a snippet manager. Not a
collaboration tool. It does one thing — code to image — and does it with
precision. The single-function, zero-server, privacy-first shape is the
product, not a stepping stone to something larger. Features that break that
shape are listed under "Out of Scope" so the line is drawn on purpose.

---

## Current State (verified against the repo, not assumed)

- **Stack**: Rust 2021 + Leptos 0.8 (CSR) + Trunk, `wasm32-unknown-unknown`,
  deployed to Vercel as static assets behind security headers. Version `0.1.0`
  in `Cargo.toml`. No server — the browser does everything.
- **Workspace**: 4 crates — `models` (shared types, zero deps beyond serde),
  `highlighter` (syntect wrapper, framework-agnostic), `renderer` (Canvas2D
  drawing, no Leptos), `app` (the only Leptos-aware crate).
- **CI** (`.github/workflows/ci.yml`): 7 jobs — `check` (WASM), `clippy`,
  `fmt --check`, `test` (`cargo test --lib`), `cargo audit`, `cargo deny`,
  gated `trunk build --release`. SHA-pinned actions, `persist-credentials:
  false`, `permissions: contents: read`. `RUSTFLAGS: "-Dwarnings"` enforced
  globally.
- **Syntax highlighting**: `syntect` with `default-fancy` (fancy-regex backend,
  wasm32-compatible). 15 languages. Extra grammars: TypeScript (wrapper),
  TOML (vendored).
- **Themes**: 4 bundled `.tmTheme` files — Dracula, One Dark, Nord, GitHub
  Light. Embedded via `include_bytes!`, parsed lazily with `OnceLock`.
- **Fonts**: 3 bundled monospace fonts — JetBrains Mono, Fira Code, Cascadia
  Code (woff2/ttf). `@font-face` declarations in `style.css`.
- **Rendering**: Canvas2D pixel-level control via `web-sys`. Preview canvas
  (capped at `min(devicePixelRatio, 2.0)`) and export canvas (fresh off-screen
  element, user-selectable 1–12x scale). `toBlob("image/png")` for export.
- **Design system**: Editorial monochrome — five tiers of grey, hairline
  dividers, tonal surface shifts, black pill buttons, 8px spacing grid. All
  colors route through CSS custom properties in `:root`.
- **Controls**: Code textarea, language/theme/font selects, font-size/padding/
  corner-radius sliders, segmented scale selector (1x/2x/4x/8x + custom),
  window-frame and line-number toggles, 6 background presets (solid + gradient).
- **Tests**: 19 unit tests across `models` (4), `highlighter` (7), `renderer`
  (8 in `layout.rs`). All pure Rust, no WASM runtime needed.
- **Deployment**: Vercel with SPA rewrite, security headers (X-Content-Type-Options,
  X-Frame-Options, Referrer-Policy, Permissions-Policy), immutable caching for
  static assets.
- **Safety**: `#![deny(unsafe_code)]` in every crate. No `unwrap()` in
  production paths.

### Current status

Phase 1 (Foundation + CI Hardening) is **complete**. PR #1 merged into `main`
with all 7 CI jobs passing. The project now has:

- Complete documentation: `DESIGN.md`, `CONTRIBUTING.md`, `SECURITY.md`
- Supply-chain security: `cargo audit` + `cargo deny` enforced in CI
- Visual identity: SVG favicon linked in `index.html`
- CI hardened: SHA-pinned actions, restricted permissions, 7-job pipeline

| Feature | Status |
|---------|--------|
| Syntax highlighting (15 languages) | Working |
| 4 themes (dark + light) | Working |
| 3 bundled fonts | Working |
| Live preview (reactive, capped scale) | Working |
| High-res PNG export (1–12x) | Working |
| Window frame (macOS traffic lights) | Working |
| Line numbers | Working |
| Background presets (6) | Working |
| Font-size / padding / corner-radius controls | Working |
| Responsive layout (controls move below on small screens) | Working |
| Ligature warning | Working |
| Font loading guard (`document.fonts.ready`) | Working |
| Staleness guard in preview (generation counter) | Working |
| Separate preview/export canvases | Working |
| `#![deny(unsafe_code)]` | Enforced |
| CI (7 jobs) | Enforced |
| DESIGN.md | Exists |
| CONTRIBUTING.md | Exists |
| SECURITY.md | Exists |
| Favicon (SVG) | Exists |
| `cargo audit` | Passes |
| `cargo deny` | Passes |

### Gaps found while reading the repo

1. **No dark/light theme toggle for the UI itself.** The canvas output has
   4 themes, but the controls and chrome are always light. A user working in
   Dracula at 2 AM stares at a white sidebar.

2. **Inline hex colors leak past the token system.** The renderer's
   `TRAFFIC_LIGHT_COLORS` array in `canvas.rs` uses `["#ff5f57", "#febc2e",
   "#28c840"]` — hardcoded, not routable through the design system. The
   preview canvas shadow in `style.css` uses `rgba(0, 0, 0, 0.08)` and
   `rgba(0, 0, 0, 0.12)` inline.

3. **No SVG export.** The token stream is generic enough to support it, but
   no implementation exists. The README explicitly marks this as a known
   limitation.

4. **No clipboard integration.** Users must use the file-download flow
   even when they just want to paste the image into Slack or a slide.

5. **No undo/redo for the code editor.** The `<textarea>` has native undo,
   but there is no history for the full settings state (theme, scale, etc.).

6. **No URL sharing of settings.** Each page load starts from the same
   defaults. There is no way to bookmark a specific configuration.

7. **`line_height` is hardcoded to 1.5** in `Settings::export_options()`.
   Not exposed as a user control, not documented as intentional.

8. **No offline story.** The app is a static site, but there is no service
   worker, no manifest, no PWA support. It could work offline trivially
   (it's already CSR + static), but doesn't yet.

---

## Milestones

| Milestone | Theme | What ships |
|-----------|-------|------------|
| **v0.2** | Foundation | `DESIGN.md`, `CONTRIBUTING.md`, `SECURITY.md`, favicon, `cargo audit` + `cargo deny` in CI ✅ |
| **v0.4** | Visual Identity | Dark / sepia / light UI theme toggle, favicon, inline hex audit, perf baseline measured |
| **v0.6** | Export & UX | SVG export, copy to clipboard, new controls (line-height, tab-width, custom bg), keyboard shortcuts |
| **v0.8** | Accessible + Offline | Full a11y pass, WCAG AA contrast, PWA with offline support, service worker |
| **v1.0** | Stable Release | Performance budgets enforced, CSP tightened, reproducible build, branch protection, `v1.0.0` tag |

---

## Phase 1: Foundation + CI Hardening ✅

The project already references `DESIGN.md`, `CONTRIBUTING.md`, and
`SECURITY.md` — none of which exist. At the same time, supply-chain
security should be enforced from day one, not bolted on later. This
phase does both: writes the missing docs *and* tightens CI.

- [x] **Write `DESIGN.md`** — an authoritative spec for CodeShot's visual
  identity: the editorial monochrome system, palette tokens, type scale,
  spacing grid, elevation model, and the *why* behind each choice. Reference
  the existing `:root` tokens in `style.css` as the source of truth. This
  file is the single reference for any future UI work.

- [x] **Write `CONTRIBUTING.md`** — how to set up the dev environment, run
  the CI checks locally, the crate boundary rules (AGENTS.md §3), the
  `#![deny(unsafe_code)]` policy, and the PR workflow. Should be short and
  actionable.

- [x] **Write `SECURITY.md`** — the security posture: static-site-only (no
  server, no secrets in client code), no user data collected, no cookies,
  no tracking, no analytics. What the CSP and Vercel headers protect
  against. How to report a vulnerability.

- [x] **`cargo audit` in CI** — fail the build on any advisory for a direct
  dependency. Yanked crate detection. Add as a new job in
  `.github/workflows/ci.yml`.

- [x] **`cargo deny` in CI** — license audit (only MIT/Apache-2.0 allowed),
  duplicate crate detection, yanked crate detection. Add as a new job in
  `.github/workflows/ci.yml`.

- [x] **Favicon** — generate a simple SVG favicon (a code bracket icon or
  the letters "CS"). Add `<link rel="icon">` to `index.html`. Minimal
  visual identity so the tab is recognizable.

**Acceptance:** all three docs exist, are accurate to the code (not aspirational),
and are referenced from the README; `cargo audit` + `cargo deny` jobs pass
in CI; favicon loads in all browsers. ✅ **All met.**

---

## Phase 2: Visual Identity — Dark Mode + Design Tokens

The canvas has themes. The UI chrome does not. A user working at night
should not be blinded by a white sidebar.

- [ ] **Write `DESIGN.md`'s dark mode extension** — define how the
  monochrome system adapts under `[data-theme="dark"]` and
  `[data-theme="sepia"]`. Same token names, different values. No new hex —
  all three modes derive from the same CSS custom properties.

- [ ] **Add `data-theme` attribute toggle** to `<html>`, cycling through
  `light`, `dark`, and `sepia`. Persist the choice in `localStorage`.
  Re-derive every CSS token under each theme — no new hex colors.

- [ ] **Extend `style.css` with dark and sepia token sets** under
  `[data-theme="dark"]` and `[data-theme="sepia"]`, using the same
  `--variable` names. Dark: dark surfaces, light text. Sepia: warm paper
  background, muted ink.

- [ ] **Add a theme-toggle button** in the topbar (lucide `sun` / `moon`
  / `coffee` icons), next to the export button.

- [ ] **Inline hex audit** — move the renderer's `TRAFFIC_LIGHT_COLORS`
  into the theme palette (each theme defines its own traffic-light colors,
  or a fixed set is exposed as a CSS custom property). Move the canvas
  shadow `rgba` values in `style.css` into tokens. Add a CI grep step
  that fails on raw `#rrggbb` in `.css` / `.rs` view code.

- [ ] **Performance baseline** — measure WASM `.wasm` gzip size, cold
  first-paint, preview render time, export time at 4x on a mid-tier
  device with throttled network. Record in `docs/perf-baseline.md`.
  This number becomes the reference for Phase 7 budgets.

**Acceptance:** three-theme toggle works; zero inline hex in CSS (CI enforced);
perf baseline doc exists; favicon in `index.html`.

---

## Phase 3: Export & UX Depth

The current controls are functional but minimal. This phase adds the
adjustments that make the output *yours*, plus export formats beyond PNG.

- [ ] **SVG export** — walk the same `Token` + `Layout` data, emit SVG
  `<text>` elements with `<tspan>` per token (color, font-style). The
  layout math in `layout.rs` is already pure and testable — reuse it
  directly. Offer SVG as a second download option (button or dropdown).

- [ ] **Copy to clipboard** — `navigator.clipboard.write()` with a
  `ClipboardItem` holding the PNG blob. A "Copy" button next to the
  "Export" button. Show a brief "Copied!" confirmation.

- [ ] **Line-height control** — expose `line_height` as a slider (range
  1.0–2.5, step 0.1, default 1.5). Currently hardcoded in
  `state.rs:63`.

- [ ] **Tab-width control** — the renderer hardcodes `TAB_WIDTH = 4` in
  `layout.rs`. Expose as a select (2 / 4 / 8) so users can match their
  editor's settings.

- [ ] **Custom background color picker** — beyond the 6 presets, allow
  the user to pick a solid color or define a two-stop gradient via a
  simple color input. The `Background` enum in `models` already supports
  `Solid(RgbColor)` and `Gradient(Vec<RgbColor>)`.

- [ ] **Code input improvements** — tab key inserts spaces (not focus-
  trap), line numbers in the textarea gutter (CSS counter), and a
  "paste from clipboard" button for quick import.

- [ ] **Export filename template** — allow the user to set a pattern
  (default: `codeshot-{scale}x.png`). Simple string interpolation:
  `{language}`, `{theme}`, `{timestamp}`.

- [ ] **Keyboard shortcuts** — `Ctrl/Cmd+Enter` to export, `Ctrl/Cmd+Z`
  undo (native textarea), `Ctrl/Cmd+Shift+Z` redo. Document in the UI
  with a subtle hint or a `?` help overlay.

- [ ] **Custom export dimensions** — let the user set a target width
  (e.g. 1200px for Twitter, 1920 for a slide) and compute the scale
  from that, instead of always starting from logical size × scale.

**Acceptance:** SVG export produces valid, renderable SVG; clipboard copy works
in Chrome/Firefox/Safari; every new control is reactive (Leptos signal), has
a sensible default, and the export reflects it immediately in the preview.

---

## Phase 4: Accessibility & Keyboard-First Use

A tool that generates images for sharing should itself be shareable —
via keyboard, screen reader, and assistive technology.

- [ ] **Keyboard-only pass** — every control reachable via Tab, every
  action triggerable via Enter/Space. The code textarea already handles
  typing; the selects, sliders, toggles, and buttons must follow.

- [ ] **ARIA labels** — `aria-label` on the canvas ("Preview of exported
  code image"), `aria-live="polite"` on the error banner, proper
  `<label>` associations on every input (some `for=`/`id=` links are
  missing today).

- [ ] **Focus-visible styling** — a clear focus ring on every interactive
  element, using the existing `--primary` token. No `outline: none`
  without a replacement.

- [ ] **Reduced-motion respect** — `@media (prefers-reduced-motion:
  reduce)` disables the export-button opacity transition and any future
  animations.

- [ ] **Contrast audit** — verify all text/background combinations in
  light, dark, and sepia themes against WCAG AA (4.5:1 for body text,
  3:1 for large text and UI components).

**Acceptance:** full keyboard navigation; ARIA labels on all interactive
elements; contrast ratios pass AA in all three themes.

---

## Phase 5: Offline-First (PWA)

CodeShot is already a static site with no server dependency. Making it
a PWA is a natural fit — and it means the tool works on planes, trains,
and bad café wifi.

- [ ] **Web app manifest** (`manifest.json`) — name, icons, theme-color,
  display: standalone. Link from `index.html`.

- [ ] **Service worker** — cache the app shell (HTML, CSS, WASM, fonts,
  themes) on install. Serve from cache first, fall back to network.
  The app has no dynamic API calls, so this is straightforward.

- [ ] **Install prompt** — detect `beforeinstallprompt`, show an
  "Install CodeShot" button in the topbar. Dismiss after install.

- [ ] **Offline indicator** — a calm banner when the network is offline
  ("You're offline — CodeShot still works"). Dismiss when back online.

**Acceptance:** `trunk build --release` + deploy to any static host; the app
loads and fully functions with network disabled; installable on Chrome/Edge/
Safari.

---

## Phase 6: Performance Budgets (verified, not claimed)

The baseline was measured in Phase 2. Now enforce it.

- [ ] **Set CI-enforced budgets** — bundle size ceiling that fails the
  build; first-paint target. Calibrated to the Phase 2 baseline, not
  guesses. Write the budget rules in `ci.yml`.

- [ ] **Over-render audit** — confirm the reactive graph doesn't
  recompute the canvas on unrelated signal changes. The `generation`
  counter in `preview.rs` already handles staleness, but the effect
  could still fire unnecessarily.

- [ ] **Re-enable `wasm-opt`** — `index.html` currently has
  `data-wasm-opt` on the Trunk rust link, but verify it's actually
  running in the CI build. If not, add `--release` to the trunk build
  command in `ci.yml` and confirm the size reduction.

- [ ] **PNG optimization** — explore `oxipng` compiled to WASM for
  post-processing the exported blob (lossless compression). Measure
  the size reduction vs. the compile-time and runtime cost before
  committing.

**Acceptance:** budgets enforced in CI; baseline doc updated with before/after;
no regression merges without a noted exception.

---

## Phase 7: Supply-Chain & Security Hardening

`cargo audit` and `cargo deny` were added in Phase 1. This phase
tightens the remaining security surface.

- [ ] **CSP audit** — review `vercel.json` headers. The current config
  has no `Content-Security-Policy` header. Add one that allows only
  `script-src 'self'`, `style-src 'self' 'unsafe-inline'` (Leptos
  needs inline styles), `connect-src 'self'` (no external APIs), and
  `font-src 'self'`. No `unsafe-eval`.

- [ ] **Dependency pinning** — `Cargo.lock` is already committed (good).
  Verify `Cargo.toml` uses version ranges, not exact pins, for direct
  deps; the lock file handles reproducibility.

- [ ] **`#![deny(unsafe_code)]` stays** in every crate. Any future
  exception must be justified, isolated, tested, and noted in the crate's
  `lib.rs` doc comment.

- [ ] **CSP header verification** — add a CI step that fetches the
  deployed site and asserts the `Content-Security-Policy` header is
  present and contains no `unsafe-inline` or `unsafe-eval` (except
  the Leptos inline-style exception).

**Acceptance:** CSP header present and correct; `cargo audit` + `cargo deny`
green in CI; no `unsafe` in any crate.

---

## Phase 8: Documentation & First Stable Release (v1.0.0)

- [ ] **Reproducible build documented** — exact Rust toolchain (via
  `rust-toolchain.toml`), Trunk version, env inputs → the same `dist/`
  from a given commit. Write in `docs/build.md`.

- [ ] **Vercel preview on every PR** — `vercel.json` is already
  configured; ensure the GitHub integration creates preview deployments
  so header/rewrite regressions are caught before `main`.

- [ ] **Branch protection on `main`** — strict required status checks
  (the 7 CI jobs), no force-push, no deletion.

- [ ] **User-facing getting-started** — extend the README with a
  screenshot walkthrough: open → paste code → tweak settings → export.
  Include the keyboard shortcuts from Phase 3.

- [ ] **Privacy statement** — explicit in the README and in-app: no
  cookies, no analytics, no tracking, no server. Your code never leaves
  your browser.

- [ ] **`v1.0.0` tag** — once Phases 1–7 acceptance checks pass, cut a
  release with `git-cliff` changelog.

**Acceptance:** a tagged, reproducible release; branch protection live; docs
match the app.

---

## How the phases relate

```
Phase 1 (docs + CI hardening)     ─┐
Phase 2 (visual identity + perf)   ─┤ foundation — do these first
Phase 3 (export + UX depth)        ─┘
        │
        ▼
Phase 4 (a11y)  ─┬─► Phase 5 (PWA)
                  └─► Phase 6 (perf budgets — enforced against Phase 2 baseline)
        │
        ▼
Phase 7 (security hardening)
        │
        ▼
Phase 8 (v1.0.0)
```

Phase 1 comes first on purpose: CodeShot cannot ship without the documents
it already references, and `cargo audit`/`cargo deny` should catch
vulnerabilities from day one — not after they've accumulated. Phase 2 follows
with the visual identity (dark mode) because users notice a white sidebar at
2 AM immediately. Everything after deepens the one thing CodeShot does:
turning code into beautiful images.

---

## Out of Scope (drawn on purpose, to stay a focused tool)

Each of these is valuable *for a different product*. CodeShot stays small
and single-purpose on purpose:

- **Cloud storage / user accounts** — CodeShot is stateless by design.
  There is no user data to store server-side. The moment you add accounts,
  you add a server, a privacy surface, and a maintenance burden. Out of
  scope.

- **Collaboration / sharing** — CodeShot generates images for sharing;
  it is not a platform for sharing code. No team features, no shared
  workspaces, no comments.

- **Code editing features** — autocomplete, linting, multi-file support,
  git integration. CodeShot has a textarea, not an editor. That is
  intentional.

- **Server-side rendering / headless browser export** — defeats the
  entire purpose. All rendering is client-side via Canvas2D. No exception.

- **Telemetry / analytics** — explicitly never. The privacy statement
  in Phase 8 commits to zero data collection.

- **Native mobile apps** — the PWA (Phase 5) is the mobile story. A
  separate native app is post-1.0 at the earliest, if ever.

- **AI features** — code explanation, auto-formatting, smart suggestions.
  Adds a network dependency, a cost surface, and a privacy concern that
  a quiet offline tool should not carry.

## Future / Ecosystem (post-1.0, if they keep CodeShot focused)

- **Theme editor** — a visual tool to create custom `.tmTheme` files
  within the app, with live preview on the current code. Still single-
  user, still offline.

- **Code snippet library** — save snippets to `localStorage` / IndexedDB,
  organize by language, re-use across exports. Still no server.

- **Batch export** — paste multiple snippets, export all as PNG/SVG in a
  zip. Useful for documentation or slide decks.

- **Custom font upload** — let the user load their own monospace font
  (woff2) for the canvas export, beyond the 3 bundled.

- **Additional languages** — community-contributed grammars via
  `syntaxes/` directory. The highlighter already supports custom
  grammars.

- **Embed / iframe mode** — a query-parameter-driven mode that
  pre-populates the settings (code, theme, scale) for embedding in
  other tools or documentation sites.

---

## Vision After v1.0.0

CodeShot's North Star does not change after v1.0. It gets sharper.

**The goal is to be the default code-screenshot tool** — the one people reach
for instinctively, the way they reach for `curl` or `jq`. Not because it has
the most features, but because it has *exactly the right ones*, and they work
everywhere, offline, with zero friction.

Post-1.0, the priorities are:

1. **Speed** — export in under 200ms for any reasonable snippet. The
   performance baseline from Phase 2 becomes a regression gate, not a
   nice-to-have.

2. **Fidelity** — the exported image should look exactly like what the user
   sees in the preview. No rendering surprises at any scale, any font, any
   theme.

3. **Portability** — work on any device with a browser, from a Chromebook to
   an iPad to a train with no wifi. The PWA from Phase 5 is the foundation;
   post-1.0 adds refinements, not a new platform.

4. **Silence** — no telemetry, no accounts, no cloud, no prompts. The tool
   appears, does its job, and gets out of the way. That quietness is the
   feature, and it is non-negotiable.

If CodeShot ships v1.0 with these four properties intact, it will have
earned its place — not by being the biggest tool, but by being the one
that respects the user's time, attention, and privacy more than any
alternative.
