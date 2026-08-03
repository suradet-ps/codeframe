# Performance Baseline

> Measured on 2026-07-27 from `trunk build --release` output.
> This document is the reference for Phase 6 performance budgets.

---

## Test Environment

- **OS**: Windows 11
- **Browser**: Chrome 151 (headless)
- **Machine**: Developer workstation (not a throttled mobile device)
- **Rust**: nightly via `rust-toolchain.toml`
- **Trunk**: 0.21.14
- **Build profile**: `release` (optimized, `wasm-opt -Oz`)

---

## Bundle Sizes

| Asset | Raw | Gzipped |
|-------|-----|---------|
| WASM (`_bg.wasm`) | 2,488 KB (2.37 MB) | 1,057 KB |
| JS glue (`_bg.js`) | 41 KB | ~12 KB |
| CSS (`style.css`) | 11 KB | ~3 KB |
| **Total (excl. fonts)** | **2,540 KB** | **~1,072 KB** |

| Font | Size |
|------|------|
| Cascadia Code (TTF) | 719 KB |
| JetBrains Mono (woff2, 4 variants) | 88 KB |
| Fira Code (woff2, 2 variants) | 45 KB |

Fonts are loaded on demand via `@font-face` and cached by the browser.

---

## Navigation Timing (Chrome headless, local server)

| Metric | Time |
|--------|------|
| DOM interactive | 53 ms |
| DOM content loaded | 54 ms |
| Load complete | 56 ms |

> Note: Measured on a local server with no network latency. Cold load over
> the network (e.g. Vercel) will be higher due to asset download time.

---

## Memory

| Metric | Value |
|--------|-------|
| JS Heap used | 2.7 MB |
| JS Heap total | 3.7 MB |

---

## How to Reproduce

1. `trunk build --release`
2. Serve `dist/` with any static server
3. Open Chrome DevTools → Performance tab → reload page
4. Check Network tab for transfer sizes
5. Check Performance tab for timing breakdown

---

## Reference for Phase 6 Budgets

These numbers calibrate the CI-enforced budgets in Phase 6:

- **WASM gzipped budget**: ~1,200 KB ceiling (10% headroom over 1,057 KB)
- **First paint target**: < 200 ms (local), < 1,000 ms (3G throttled)
- **Preview render target**: < 50 ms per frame (typical snippet, < 200 lines)
- **Export at 4x target**: < 500 ms per frame (typical snippet, < 200 lines)

> These targets are intentionally conservative for v1. Phase 6 will tighten
> them after measuring on a throttled mid-tier device.

---

## PNG Optimization Exploration (Phase 6)

### Option: oxipng via WASM

**Package**: `@jsquash/oxipng` (npm) - oxipng compiled to WebAssembly
**Alternative**: Use `oxipng` as a Rust library compiled to WASM

#### Potential Benefits
- Lossless PNG compression (10-30% size reduction typical)
- No server required - runs entirely in browser
- Could be offered as optional post-processing step

#### Integration Challenges
1. **WASM-in-WASM**: CodeFrame's main app is already WASM; loading a second
   WASM module (oxipng) adds complexity and bundle size
2. **Async boundary**: oxipng WASM would need to be loaded separately,
   adding to initial load time
3. **Performance trade-off**: Compression takes 100-500ms depending on
   image size and optimization level

#### Recommendation
**Deferred to post-v1.** The current PNG output from Canvas2D is already
well-compressed by the browser. The marginal gains from oxipng don't
justify the added complexity and bundle size for v1.

If pursued later, the recommended approach is:
- Use `oxipng` as a Rust library (not the npm wrapper)
- Compile to WASM via `wasm-pack`
- Offer as optional "Optimize PNG" toggle in export settings
- Show compression ratio and savings in UI

### Current State
Canvas2D's `toBlob("image/png")` uses the browser's built-in PNG encoder,
which provides reasonable compression. The exported images are typically
200-800 KB at 2x scale for typical code snippets.
