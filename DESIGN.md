# DESIGN.md — CodeShot Design System

> The visual language of CodeShot: an editorial monochrome system built for
> focus, restraint, and readability. This file is the single source of truth
> for any UI work.

---

## Philosophy

CodeShot is a tool for presenting code, not for decorating it. The design
system follows three principles:

1. **Restraint over expression.** The UI should disappear. The code — and
   the image it becomes — is the only thing that matters. Every color,
   every shadow, every border exists only to serve that goal.

2. **Monochrome as a system, not a limitation.** Five tiers of grey handle
   every surface, text, and divider. Accent color comes from the *canvas
   output* (the code's background gradient), not from the chrome. The UI
   is neutral so the exported image can be vivid.

3. **Hairline over shadow.** Depth is communicated through tonal surface
   shifts and 1px hairlines, not through drop shadows or heavy borders.
   The only shadow in the entire UI is on the exported code card — and
   that is part of the *output*, not the chrome.

---

## Design Tokens

All tokens are CSS custom properties in `:root` (`style.css:55–93`). The
same token names are reused across all three themes (`light`, `dark`,
`sepia`) — only the values change.

### Brand & Accent

| Token | Light | Purpose |
|-------|-------|---------|
| `--primary` | `#000000` | Primary action background (export button) |
| `--on-primary` | `#ffffff` | Text on primary action |
| `--accent` | `#111111` | Subtle accent (hover states, active borders) |

### Surfaces

| Token | Light | Purpose |
|-------|-------|---------|
| `--canvas` | `#ffffff` | Page background |
| `--canvas-warm` | `#fafafa` | Slightly warm surface (textarea, inputs, preview area) |
| `--surface-cool` | `#f5f5f5` | Cool surface (unused in v1, reserved) |
| `--surface-elevated` | `#f0f0f0` | Elevated surface (unused in v1, reserved) |
| `--hairline` | `#e5e5e5` | 1px dividers, borders |
| `--hairline-soft` | `#eeeeee` | Softer dividers (unused in v1, reserved) |

### Text

| Token | Light | Purpose |
|-------|-------|---------|
| `--ink` | `#0a0a0a` | Primary text, headings |
| `--ink-soft` | `#333333` | Secondary text |
| `--graphite` | `#555555` | Tertiary text |
| `--slate` | `#777777` | Muted text (slider values) |
| `--stone` | `#999999` | Labels, placeholders |
| `--ash` | `#bbbbbb` | Disabled text |
| `--mute` | `#dddddd` | Very muted (borders on disabled) |

### Semantic

| Token | Light | Purpose |
|-------|-------|---------|
| `--warning` | `#b45309` | Warning text (ligature hint) |
| `--error-bg` | `#fef2f2` | Error banner background |
| `--error-border` | `#fecaca` | Error banner border |
| `--error-text` | `#991b1b` | Error banner text |

### Spacing (8px grid)

| Token | Value | Use |
|-------|-------|-----|
| `--sp-xxs` | 4px | Tight gaps |
| `--sp-xs` | 8px | Standard small gap |
| `--sp-sm` | 12px | Input padding |
| `--sp-md` | 16px | Section gaps, inner padding |
| `--sp-lg` | 24px | Sidebar section gaps |
| `--sp-xl` | 32px | Preview area padding |
| `--sp-xxl` | 48px | Outer preview padding |
| `--sp-section` | 64px | (unused in v1, reserved) |

---

## Typography

### UI Chrome

| Element | Font | Size | Weight | Color |
|---------|------|------|--------|-------|
| Brand name | System sans-serif | 15px | 500 | `--ink` |
| Brand tag | System sans-serif | 13px | 400 | `--stone` |
| Control labels | System sans-serif | 11px | 500 | `--stone` |
| Selects, inputs | System sans-serif | 13px | 400 | `--ink` |
| Toggle text | System sans-serif | 13px | 400 | `--ink-soft` |
| Slider value | System sans-serif | 12px | 400 | `--slate` |
| Export button | System sans-serif | 14px | 600 | `--on-primary` |
| Hint text | System sans-serif | 12px | 400 | `--warning` |
| Error banner | System sans-serif | 13px | 400 | `--error-text` |

### Code Input (textarea)

| Property | Value |
|----------|-------|
| Font | "JetBrains Mono", monospace |
| Size | 12.5px |
| Line height | 1.5 |
| Tab size | 4 |

### Exported Code (canvas)

| Property | Default | Range |
|----------|---------|-------|
| Font family | JetBrains Mono | JetBrains Mono / Fira Code / Cascadia Code |
| Font size | 14px | 10–24px |
| Line height | 1.5 | 1.0–2.5 |

---

## Elevation Model

CodeShot uses tonal shifts, not shadows, for depth:

| Level | Surface | Use |
|-------|---------|-----|
| 0 | `--canvas` | Page background, topbar |
| 1 | `--canvas-warm` | Textarea, selects, preview area background |
| — | `--hairline` | Dividers between levels (1px) |

The only shadow in the UI is on the **exported code card** (not the chrome):
`0 1px 3px rgba(0, 0, 0, 0.08), 0 8px 32px rgba(0, 0, 0, 0.12)` on
`.preview-canvas`. This is part of the output image, not the app UI.

---

## Canvas Rendering Constants

These live in `crates/renderer/src/layout.rs` and define the exported
image's internal geometry:

| Constant | Value | Purpose |
|----------|-------|---------|
| `TAB_WIDTH` | 4 | Spaces per tab stop |
| `INNER_PADDING` | 16.0px | Gap between card edge and code text |
| `FRAME_HEADER_HEIGHT` | 40.0px | macOS window-frame header bar height |
| `TRAFFIC_LIGHT_RADIUS` | 6.0px | Radius of each traffic-light dot |
| `TRAFFIC_LIGHT_PITCH` | 22.0px | Horizontal distance between dot centers |
| `TRAFFIC_LIGHT_OFFSET_X` | 22.0px | X offset of first dot from card edge |
| `GUTTER_GAP_CELLS` | 1.5 | Gap between line-number gutter and code (in char cells) |

### Traffic-Light Colors (hardcoded in canvas.rs)

| Dot | Color | Hex |
|-----|-------|-----|
| Close | Red | `#ff5f57` |
| Minimize | Yellow | `#febc2e` |
| Zoom | Green | `#28c840` |

> **Note:** These are macOS system colors, not part of the design token
> system. They are the same across all themes because they represent a
> specific UI chrome element (macOS window controls), not an application
> design choice.

### Rendering Layer Order (bottom to top)

1. Background gradient or solid color (per `ExportOptions.background`)
2. Code card: rounded rect filled with `palette.background`, optional drop shadow
3. Window-frame header: clipped semi-transparent band + traffic-light dots
4. Token text: `fillText` per token, manually advancing the x cursor
5. Line numbers: right-aligned in gutter, `foreground` at 45% alpha

---

## Component Patterns

### Primary Action Button (`.export-btn`)

- Black pill: `background: var(--primary)`, `border-radius: 9999px`
- Height: 40px, padding: 0 var(--sp-lg)
- Hover: `opacity: 0.85`
- Disabled: `opacity: 0.5`, `cursor: wait`

### Controls Sidebar (`.controls`)

- Width: 320px (fixed via grid)
- Background: `var(--canvas)`
- Right border: `1px solid var(--hairline)`
- Sections separated by `gap: var(--sp-lg)`
- Labels: 11px uppercase, `var(--stone)`, letter-spacing 0.035em

### Code Input (`.code-input`)

- Background: `var(--canvas-warm)`
- No border, bottom hairline only
- Focus: bottom border becomes `var(--ink)`
- No border-radius

### Selects and Inputs

- Same pattern as code input
- Background: `var(--canvas-warm)`
- Bottom hairline, no border-radius

### Segmented Control (`.segmented`)

- Row of buttons with bottom hairline
- Active state: bottom border becomes `var(--ink)`

### Background Swatches (`.swatches`)

- 6-column grid of square buttons
- Active: 2px border in `var(--ink)`
- Hover: border in `var(--stone)`

### Preview Area (`.preview-area`)

- Background: `var(--canvas-warm)` with radial dot pattern
  (`radial-gradient(circle, var(--hairline) 1px, transparent 1px)`,
  24px grid)
- Canvas centered with flexbox
- Canvas shadow: `0 1px 3px rgba(0, 0, 0, 0.08),
  0 8px 32px rgba(0, 0, 0, 0.12)`

---

## Themes

### Light (default)

Values as listed in the token tables above. Page background is white,
surfaces are warm grey, text is near-black.

### Dark (`[data-theme="dark"]`)

Same token names, inverted values:
- `--canvas`: dark grey (~`#1a1a1a`)
- `--canvas-warm`: slightly lighter (~`#222222`)
- `--ink`: light (~`#f0f0f0`)
- `--hairline`: dark divider (~`#333333`)
- All text tokens shift to lighter greys

### Sepia (`[data-theme="sepia"]`)

Warm reading mode:
- `--canvas`: warm paper (~`#f5f0e8`)
- `--canvas-warm`: slightly warmer (~`#faf5ed`)
- `--ink`: dark brown (~`#3d3427`)
- `--hairline`: warm divider (~`#d4c9b8`)

---

## Responsive Behavior

| Breakpoint | Layout |
|------------|--------|
| > 900px | Sidebar (320px) + main area, 2-column grid |
| ≤ 900px | Single column: header → main → controls (controls get `max-height: 48vh`) |

---

## What This System Does Not Do

- **No drop shadows on chrome.** Depth comes from tonal shifts and hairlines.
- **No accent colors in the UI.** The UI is monochrome. Color comes from
  the exported image (the code's background gradient).
- **No rounded corners on UI elements.** Inputs, selects, segmented buttons,
  and swatches use `border-radius: 0`. The only rounded element is the
  export button (`border-radius: 9999px`) and the preview canvas
  (`border-radius: 8px`).
- **No animations or transitions** beyond `opacity 0.15s ease` on the
  export button. The UI is still.
