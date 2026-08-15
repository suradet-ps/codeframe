# SECURITY.md - CodeFrame

> CodeFrame's security posture. Read this before filing a vulnerability report.

---

## What CodeFrame Is

A **static website** deployed to Vercel. The entire application is:

- HTML
- CSS
- WebAssembly (compiled from Rust)
- Bundled fonts (woff2/ttf)
- Bundled theme files (.tmTheme)

There is **no backend**, **no API server**, **no database**, and **no
server-side logic** of any kind. The browser does everything.

---

## What CodeFrame Does Not Collect

- **No cookies.** Zero. The app sets no cookies and reads no cookies.
- **No localStorage beyond theme preference.** The only thing stored in
  `localStorage` is the user's UI theme choice (`light`/`dark`/`sepia`).
  No code, no settings, no identifiers.
- **No analytics.** No Google Analytics, no Plausible, no Fathom, no
  telemetry of any kind.
- **No tracking.** No fingerprinting, no beacons, no third-party scripts.
- **No accounts.** No user registration, no login, no sessions.
- **No server-side processing.** Code entered by the user never leaves the
  browser. It is highlighted client-side via `syntect` (WASM) and rendered
  to a canvas. No network request is made with the user's code.

---

## Data Flow

```
User types code → syntect (WASM, local) → Canvas2D (local) → PNG blob
                                                          ↓
                                                    Downloaded to disk
                                                    (or clipboard)
```

At no point does the code leave the browser. There is no upload, no fetch,
no WebSocket, no Server-Sent Events. The only network requests the app
makes are:

1. **Initial page load** - HTML, CSS, WASM, fonts, themes (all static assets).
2. **Nothing else.** The app has no API calls after boot.

---

## Content Security Policy

The `vercel.json` deploys security headers on every response:

| Header | Value | Purpose |
|--------|-------|---------|
| `Content-Security-Policy` | See below | Deny-by-default, self-origin only |
| `X-Content-Type-Options` | `nosniff` | Prevents MIME-type sniffing |
| `X-Frame-Options` | `DENY` | Prevents framing (clickjacking) |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Limits referrer leakage |
| `Permissions-Policy` | `camera=(), microphone=(), geolocation=()` | Disables unused APIs |

CSP directives enforced:

```
default-src 'none';
script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval';
style-src 'self' 'unsafe-inline';
img-src 'self' blob:;
font-src 'self';
connect-src 'self';
worker-src 'self';
manifest-src 'self';
```

No `unsafe-eval`. The `unsafe-inline` for styles is required by Leptos's
CSR rendering. The `unsafe-inline` for scripts is required because Trunk
generates an inline `<script type="module">` to bootstrap WASM — its
content (including hashed filenames) changes per build, making a static
SHA-256 hash non-viable. The `wasm-unsafe-eval` is required because
WebAssembly.instantiateStreaming() needs it; this is more restrictive
than `unsafe-eval` — it only permits WASM compilation, not arbitrary
`eval()`. CI enforces that the CSP header is present and valid via the
`csp-verify` job in `.github/workflows/ci.yml`.

---

## Build Integrity

- `#![deny(unsafe_code)]` is enforced at crate level in every Rust crate.
- `Cargo.lock` is committed, ensuring reproducible builds.
- CI runs `cargo audit` (advisory check) and `cargo deny` (license +
  yanked-crate check) on every PR.
- The `trunk build --release` output is what Vercel deploys. No post-build
  transforms, no server-side processing.

---

## Known Limitations

- **Client-side only.** There is no server to enforce anything. A motivated
  user could modify the WASM or DOM. This is by design - CodeFrame is a
  tool, not a DRM system. The output image is what it is.
- **No HTTPS enforcement on localhost.** During development (`trunk serve`),
  the app runs on `http://127.0.0.1:8080`. This is expected and only
  affects local development.

---

## Reporting a Vulnerability

If you discover a security issue:

1. **Do not open a public GitHub issue.**
2. Email [SECURITY_EMAIL] (or open a private security advisory on GitHub).
3. Include: what you found, how to reproduce it, and the potential impact.
4. We will acknowledge within 48 hours and work with you on a fix.

We do not offer bug bounties, but we will credit reporters (with permission)
in the release notes.

---

## Supply Chain

- **Rust dependencies:** Audited via `cargo audit` (advisories) and
  `cargo deny` (licenses, yanked crates, duplicates) in CI.
- **Fonts:** Bundled locally (woff2/ttf). No external font CDN.
- **Themes:** Bundled locally (.tmTheme). No external theme CDN.
- **No npm, no node_modules, no JavaScript dependencies.** The build tool
  (Trunk) is a Rust binary, not a Node package.

---

## Summary

CodeFrame is one of the simplest possible web applications: a static site
that converts text to pixels in the browser. There is no attack surface
beyond what the browser itself provides. Your code never leaves your machine.
