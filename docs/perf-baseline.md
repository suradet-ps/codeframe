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
