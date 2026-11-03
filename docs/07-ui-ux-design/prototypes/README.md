# llm — Design Prototypes

Static HTML prototypes for the llm-client (llm) interface, with scripts to capture them as images. They implement the tokens, components, and layouts defined in [`../../07-ui-ux-design.md`](../../07-ui-ux-design.md) and exist to validate the design system in a real browser before production code is written.

## 1. What this is

- **Static, dependency-free HTML.** Each page is a single self-contained file you can open directly in a browser — no build step, no server.
- **Styling via CDN.** Tailwind CSS v4 (browser build) and DaisyUI v5 are loaded from a CDN and configured in an inline `@theme` block that mirrors the design tokens. **Prototype-only** — production uses a compiled, purged stylesheet.
- **Dark theme only.** Every page ships with `skl-dark`, matching the production app.
- **Reproducible screenshots.** A Playwright script renders every page in dark mode to `images/dark/`.

## 2. Structure

```text
07-ui-ux-design/prototypes/
├── html/              # HTML prototypes (numbered 00-05)
├── images/
│   └── dark/          # skl-dark PNG screenshots
├── screenshot.py      # Renders every HTML file in dark theme
├── screenshot.sh      # Runs setup + screenshot.py (uv + Playwright)
├── pyproject.toml     # Rust dependencies (managed with uv)
└── README.md          # This file
```

## 3. Viewing

Open any file in `html/` directly:

```sh
open html/01-home.html
```

Or serve the folder if your browser restricts local resources:

```sh
rust3 -m http.server 8080 --directory html
```

## 4. Files

| HTML                                                 | PNG                 | Screen     | Notes                                                                |
| ---------------------------------------------------- | ------------------- | ---------- | -------------------------------------------------------------------- |
| [`html/00-components.html`](html/00-components.html) | `00-components.png` | Components | Colour tokens, buttons, badges, alerts, form fields, loading states, charts. |
| [`html/01-home.html`](html/01-home.html)             | `01-home.png`       | Home       | Public landing: hero, "Train a model" CTA, "View jobs" link.         |
| [`html/02-new-job.html`](html/02-new-job.html)       | `02-new-job.png`    | New job    | Wizard: Dataset → Mapping → Model, with validation-error state.      |
| [`html/03-job-status.html`](html/03-job-status.html) | `03-job-status.png` | Job status | Lifecycle steps, metrics, Model report (charts), ONNX download (primary). |
| [`html/04-jobs.html`](html/04-jobs.html)             | `04-jobs.png`       | Jobs       | History table with status badges, empty state variant noted.         |
| [`html/05-algorithms.html`](html/05-algorithms.html) | `05-algorithms.png` | Algorithms | Catalogue cards grouped by all five task types.                      |

The routes map to the app router: `/`, `/jobs/new`, `/jobs/[jobId]`, `/jobs`, `/algorithms`.

Each screen is captured in dark theme under its own name: `images/dark/<n>-<name>.png` is the `skl-dark` capture.

## 5. Generating screenshots

Screenshots use [Playwright](https://playwright.dev/rust/) via `uv`. Run the wrapper script, which installs dependencies and the browser as needed:

```sh
./docs/07-ui-ux-design/prototypes/screenshot.sh
```

Or run the steps manually:

```sh
cd docs/07-ui-ux-design/prototypes
uv sync
uv run playwright install chromium
uv run screenshot.py
```

The script:

1. Reads every `html/*.html` file (sorted by name).
2. Opens it in headless Chromium, sets `data-theme='skl-dark'` on `<html>`, and reloads.
3. Takes a full-page screenshot at a 1920×1080 viewport.
4. Writes `images/dark/<name>.png`.

The `.venv/` directory and `uv.lock` are created by `uv` and are gitignored.

## 6. Tokens

The prototypes use the design tokens directly through utility classes generated from the `@theme` block (for example `bg-base-100`, `text-text-primary`, `bg-primary`, `text-error-fg`). The canonical table lives in [`design-tokens.md`](../system/design-tokens.md#colour) — if a token changes there, update `@theme` and the `[data-theme='skl-dark']` block in these files to match.

Job status colours follow [`states.md`](../system/states.md): queued/validating/training/evaluating/exporting use `warning`, completed uses `success`, failed uses `error` — status colour pairs are always used together (`badge badge-warning` with `text-warning-content`) to preserve contrast.

## 7. Keeping prototypes in sync

These files are illustrative. When the design doc changes:

1. Update the token values in the `@theme` block and the `[data-theme='skl-dark']` block.
2. Update the affected screens.
3. Regenerate screenshots with `./screenshot.sh` and commit the HTML and PNG changes together.
4. Keep the pages consistent with the component rules in [`../../07-ui-ux-design.md`](../../07-ui-ux-design.md) — DaisyUI first, Tailwind second, custom CSS last.

## 9. Phase 9–10 updates

Phases 9 (semi-supervised) and 10 (unsupervised) are implemented, so the prototypes and screenshots now cover all five task types:

- `01-home.html` — hero sub-line "Upload a CSV, choose a model, download ONNX.", a task-aware feature grid, and null counts in the "How it works" strip.
- `02-new-job.html` — five-task select grouped by supervision, conditional label control with the blank-label helper, and threshold/max-iteration fields.
- `03-job-status.html` — status/task/algorithm/created summary cards, a task-specific metrics definition list (semi-supervised shown), the clustering/anomaly metric sets, and a Model report section (charts planned, not yet implemented).
- `04-jobs.html` — task badges for all five task types; no label column for unsupervised rows.
- `05-algorithms.html` — five catalogue groups covering 25 estimators.
- `og/svg/og.svg` — tagline without the "supervised" qualifier.
