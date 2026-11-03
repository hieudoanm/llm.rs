# llm — visx

Charting for the completed-job result charts
([`charts.md`](../../../07-ui-ux-design/system/charts.md)).

- Charts live in `src/components/charts/`; each is a client component that takes a typed prop and
  renders an `<svg>`.
- Import only what a chart needs: `@visx/scale`, `@visx/axis`, `@visx/shape`, `@visx/group`,
  `@visx/tooltip`, `@visx/responsive`; add `@visx/curve` only for a non-default line curve. Pin exact
  versions.
- Size responsively with `ParentSize` (`@visx/responsive`), or a `viewBox` with `preserveAspectRatio`;
  never assume a fixed pixel width.
- Build scales with `@visx/scale` (`scaleBand`, `scaleLinear`); domains come from the capped payload,
  not from scanning all rows.
- Colour from the design tokens; never hard-code hex. Keep AA contrast on `base-100` in both themes.
- Every chart is `role="img"` with a `<title>`/`aria-label` and a visually-hidden table of its values.
- Tooltips (`useTooltip`) are an enhancement: marks stay understandable without hover, and any
  interactive element is keyboard reachable.
- Respect `prefers-reduced-motion: reduce`; no entry animation by default.
- Tests: Vitest (jsdom) — stub `ResizeObserver`, assert axes/labels and the fallback table;
  Playwright — assert the chart region and key labels for a completed job.

## Package versions

Pin exact versions in `packages/frontend/package.json`:

```json
{
  "dependencies": {
    "@visx/scale": "^3.5.0",
    "@visx/axis": "^3.10.0",
    "@visx/shape": "^3.5.0",
    "@visx/group": "^3.3.0",
    "@visx/tooltip": "^3.5.0",
    "@visx/responsive": "^3.10.0",
    "@visx/curve": "^3.5.0"
  }
}
```

Install with:
```sh
cd packages/frontend
pnpm add @visx/scale@3.5.0 @visx/axis@3.10.0 @visx/shape@3.5.0 @visx/group@3.3.0 @visx/tooltip@3.5.0 @visx/responsive@3.10.0
```

Add `@visx/curve` only if using non-default line curves (e.g., spline interpolation).
