# Bundled fonts

Downloaded at build time and served as static assets (declared via `@font-face` in `style.css`).

| File | Font | Version / source | License |
|---|---|---|---|
| `jetbrains-mono-*.woff2` | JetBrains Mono | `@fontsource/jetbrains-mono@5` (JetBrains) | SIL OFL 1.1 |
| `fira-code-*.woff2` | Fira Code | `@fontsource/fira-code@5` (Nikita Prokopov) | SIL OFL 1.1 |
| `cascadia-code.ttf` | Cascadia Code (variable weight) | microsoft/cascadia-code release v2407.24 | SIL OFL 1.1 |

Note: all three fonts contain ligature glyphs, but Canvas2D `fillText` does not
shape ligatures — the UI surfaces this as a warning (see AGENTS.md §5).
