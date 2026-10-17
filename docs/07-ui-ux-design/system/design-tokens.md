# llm — Design Tokens

_Part of [07-ui-ux-design.md](../../07-ui-ux-design.md)._

All values below are mirrored by:

- the DaisyUI theme `skl-dark` in
  [`packages/desktop/src/styles/globals.css`](../../../packages/desktop/src/styles/globals.css), and
- the `@theme` block in the
  [prototypes](../prototypes/README.md).

Change a token here, then change it in both places.

## Brand

Product name: **llm.rs**. Wordmark in the UI navbar and marketing headers; no
third-party AutoML marks. See [`brand/README.md`](../brand/README.md) for the marks themselves.

The brand uses its own warm palette, shared by the landing page and the desktop dark theme. These
are brand tokens rather than UI-state tokens, so the product can keep its blue primary action
colour while the mark stays orange.

| Token                | Dark      | Use                       |
| -------------------- | --------- | ------------------------- |
| `brand-mark`         | `#F06B4D` | Mark box                  |
| `brand-mark-content` | `#0B1018` | Diagonal line in the mark |
| `brand-wordmark`     | `#F7F1E8` | `llm.rs` wordmark         |
| `brand-bg`           | `#0B1018` | Brand page background     |
| `brand-panel`        | `#111923` | Brand cards and panels    |
| `brand-ink`          | `#F7F1E8` | Primary brand text        |
| `brand-muted`        | `#9DA8B7` | Secondary brand text      |
| `brand-accent`       | `#FFAC70` | Warm accent               |
| `brand-mint`         | `#76D5A5` | Success and confirmation  |
| `brand-line`         | `#24303D` | Brand borders and rules   |

## Colour

Tokens are exposed to Tailwind as `--color-<name>`, so `--color-text-primary` is the utility
`text-text-primary`. DaisyUI semantic tokens (`primary`, `base-100`, …) are also available as
`btn-primary`, `bg-base-100`, and so on.

### Semantic (DaisyUI)

| Token               | Dark      | Use                             |
| ------------------- | --------- | ------------------------------- |
| `base-100`          | `#0F172A` | Surface                         |
| `base-200`          | `#1E293B` | Raised surface / default button |
| `base-300`          | `#334155` | Borders                         |
| `base-content`      | `#E2E8F0` | Text on base                    |
| `primary`           | `#60A5FA` | Train CTA                       |
| `primary-content`   | `#0B1120` | Text on primary                 |
| `secondary`         | `#94A3B8` | Secondary actions               |
| `secondary-content` | `#0B1120` | Text on secondary               |
| `accent`            | `#2DD4BF` | ONNX / success emphasis         |
| `accent-content`    | `#0B1120` | Text on accent                  |
| `neutral`           | `#1E293B` | Neutral surfaces                |
| `neutral-content`   | `#E2E8F0` | Text on neutral                 |
| `info`              | `#60A5FA` | Informational                   |
| `info-content`      | `#0B1120` | Text on info                    |
| `success`           | `#34D399` | Completed                       |
| `success-content`   | `#0B1120` | Text on success                 |
| `warning`           | `#FBBF24` | Queued / in progress            |
| `warning-content`   | `#0B1120` | Text on warning                 |
| `error`             | `#F87171` | Failed / validation             |
| `error-content`     | `#0B1120` | Text on error                   |

### llm extended

| Token                       | Dark                  | Use                        |
| --------------------------- | --------------------- | -------------------------- |
| `background`                | `#0B1120`             | Page background            |
| `background-secondary`      | `#0F172A`             | Alternating section        |
| `surface`                   | `#0F172A`             | Card surface               |
| `surface-raised`            | `#1E293B`             | Inset / icon surface       |
| `border`                    | `#334155`             | Default border             |
| `border-light`              | `#1E293B`             | Subtle divider             |
| `border-strong`             | `#475569`             | Emphasised border          |
| `text-primary`              | `#E2E8F0`             | Body text                  |
| `text-secondary`            | `#94A3B8`             | Secondary text             |
| `text-tertiary`             | `#64748B`             | Helper text                |
| `text-inverse`              | `#0B1120`             | Text on dark block         |
| `primary-hover`             | `#93C5FD`             | Primary hover              |
| `primary-active`            | `#2563EB`             | Primary active             |
| `primary-muted`             | `#172554`             | Primary tint               |
| `success-bg` / `success-fg` | `#064E3B` / `#34D399` | Success badge              |
| `warning-bg` / `warning-fg` | `#451A03` / `#FCD34D` | Warning badge              |
| `error-bg` / `error-fg`     | `#450A0A` / `#FCA5A5` | Error badge                |
| `info-bg` / `info-fg`       | `#1E3A8A` / `#93C5FD` | Info badge                 |
| `focus-ring`                | `#60A5FA`             | `:focus-visible` outline   |

Rules:

1. Do not use raw hex in components when a semantic class exists.
2. Status colours are always used as a pair (`bg-<x>-bg` with `text-<x>-fg`) so contrast holds.
3. Only dark values are defined; the app ships with a single dark theme.

## Typography

- Sans: **Inter** (the product and brand typeface), loaded with `next/font` and exposed as
  `--font-sans` / `font-sans`, with a system sans-serif fallback.
- Monospace: **JetBrains Mono**, loaded with `next/font` and exposed as `--font-mono` /
  `font-mono`, for hashes, job ids, dtypes, and metrics at `text-sm`.

Scale: `text-xs` helper, `text-sm` body, `text-xl` / `text-2xl` section headings, `text-3xl`
page/hero headings.

## Spacing

Use the Tailwind scale: page padding `p-4 md:p-8`, card `p-6`, stack `gap-4`.

## Radius and shadow

Use DaisyUI defaults (`rounded-box`, `shadow`, `shadow-sm`). Cards for upload, mapping, and
metrics. Radius is configured once in the theme (`--radius-selector`, `--radius-field`,
`--radius-box`).
