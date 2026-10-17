# FEATURE-12: Metrics & Charts Download

## Priority

P1 (enhancement). Depends on job being `completed` with `metrics` and `charts`.

## Description

When a job reaches `completed` status, users can download:

1. **`metrics.json`** — the full metrics object returned by the API (all scalar metrics).
2. **Each chart as SVG** — the raw SVG rendered by visx (vector, theme-aware).
3. **Each chart as PNG** — rasterised from the SVG at 2× device pixel ratio.

Downloads are client-side only; no server storage or new API endpoints.

## User-visible behaviour

### Downloads section (extends [FEATURE-05](./FEATURE-05-Model-Export.md))

When `status === 'completed'` and `metrics`/`charts` exist, the Downloads section shows:

- **Model artefacts** (ONNX, pkl, joblib) — existing buttons.
- **Download metrics.json** — always present for completed jobs with metrics.
- **Model report charts** — one "Download" dropdown per chart card with:
  - **Download SVG** — saves the chart's `<svg>` element as `{jobId}-{chartKey}.svg`.
  - **Download PNG** — rasterises the SVG to a 2× DPI PNG as `{jobId}-{chartKey}.png`.

### Chart cards

Each chart card in the Model report gets a small action button (top-right, icon-only) opening a dropdown with the two export options. The button is only visible on hover/focus to keep the report clean.

### Filename convention

| Content     | Pattern                  | Example                        |
| ----------- | ------------------------ | ------------------------------ |
| Metrics     | `{jobId}-metrics.json`   | `abc123-metrics.json`          |
| Chart (SVG) | `{jobId}-{chartKey}.svg` | `abc123-confusionMatrix.svg`   |
| Chart (PNG) | `{jobId}-{chartKey}.png` | `abc123-featureImportance.png` |

`chartKey` is the camelCase `charts` payload key (e.g., `confusionMatrix`, `featureImportance`, `mlpProgress`).

### Disabled state

- While `status !== 'completed'` → all download buttons disabled with tooltip "Available when training completes".
- If a chart is not rendered (data absent) → no dropdown for that chart.

## Implementation notes

### Metrics download

```ts
// Client-side blob download
const blob = new Blob([JSON.stringify(metrics, null, 2)], { type: 'application/json' });
saveAs(blob, `${jobId}-metrics.json`);
```

### SVG download

Each chart component (`BarChart`, `LineChart`, `Heatmap`, etc.) exposes its root `<svg>` via a `ref`. The parent `ModelReport` collects refs and provides a `downloadChart(chartKey, format: 'svg' | 'png')` callback.

```ts
const svgElement = refs.current[chartKey]?.current;
if (!svgElement) return;
const svgMarkup = new XMLSerializer().serializeToString(svgElement);
const blob = new Blob([svgMarkup], { type: 'image/svg+xml' });
saveAs(blob, `${jobId}-${chartKey}.svg`);
```

### PNG download (2× DPI)

Uses a tiny custom utility (`packages/frontend/src/lib/svgToPng.ts`) that rasterises the SVG to an
offscreen canvas at `pixelRatio = 2` using native browser APIs:

```ts
const svgMarkup = new XMLSerializer().serializeToString(svgElement);
const img = await loadImage(`data:image/svg+xml;base64,${btoa(svgMarkup)}`);
const canvas = document.createElement('canvas');
canvas.width = img.width * 2;
canvas.height = img.height * 2;
const ctx = canvas.getContext('2d')!;
ctx.scale(2, 2);
ctx.drawImage(img, 0, 0);
const blob = await new Promise<Blob>((resolve, reject) =>
  canvas.toBlob(b => b ? resolve(b) : reject(new Error('PNG failed')), 'image/png')
);
saveAs(blob, `${jobId}-${chartKey}.png`);
```

No npm dependency; ~500 bytes gzipped. Handles DaisyUI theme colours via CSS variables (SVG uses
`currentColor` / CSS vars), transparent background.

`loadImage` is a small Promise wrapper around `new Image()`.

### Theme handling

- SVG download preserves the current DaisyUI theme (`data-theme="mlrs-light"` or `mlrs-dark`) because the SVG uses CSS variables for colours.
- PNG download captures the current theme as rendered (light or dark). No theme switching is offered; the user downloads what they see.

### Accessibility

- Download dropdown is a native `<details>`/`<summary>` (or headless UI `Menu`) with keyboard navigation.
- Each option has an `aria-label` like "Download confusion matrix as SVG".
- Tooltip on disabled state explains why.

## Acceptance conditions

1. On a completed job page, `metrics.json` downloads and contains all scalar metrics from the API.
2. Each visible chart card has a download dropdown with SVG and PNG options.
3. SVG download produces valid, viewable SVG files with correct colours for the current theme.
4. PNG download produces raster images at ~2× DPI, matching the on-screen appearance.
5. Filenames follow the convention above.
6. No server changes; no new API endpoints; no additional DB columns.
7. Works in both light and dark themes.
8. Unit tests for the download utilities (blob creation, filename generation).

## Edge cases

- **Large charts** (confusion matrix 50×50): SVG may be ~500 KB; PNG at 2× ~1 MB. Acceptable.
- **Missing chart key**: dropdown not rendered (already handled by conditional rendering).
- **Job failed / not completed**: all buttons disabled with tooltip.
- **Firefox/Safari SVG serialization**: `XMLSerializer` works cross-browser.
- **CSP**: no inline scripts; `html-to-image` uses canvas, no eval.

## Related

- [`FEATURE-05`](./FEATURE-05-Model-Export.md) — model artefact downloads (extended by this feature).
- [`charts.md`](../07-ui-ux-design/system/charts.md) — chart data contract and rendering.
- [`components.md`](../07-ui-ux-design/system/components.md) — card/component patterns.
