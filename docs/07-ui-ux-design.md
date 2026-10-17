# scikit-learn-client (mlrs) — UI/UX Design

## 1. Purpose

This document defines the UI/UX design for mlrs: tokens, components, layout, screens, states,
accessibility, responsive behaviour, motion, iconography, brand, and the static prototypes. It is
the source of truth for the frontend in `packages/frontend`.

The design must be:

- Clear and technical (local ML studio).
- Accessible (WCAG 2.1 AA).
- Responsive (mobile, tablet, desktop).
- Consistent across screens.
- Explicit about loading, empty, validation-error, network-error, unexpected-error, and success.
- Themeable in light and dark via DaisyUI, using the `mlrs-light` and `mlrs-dark` themes.

Every subsystem document below is normative. Where a subsystem document and this index disagree,
the subsystem document governs.

## 2. Design Tokens

| Document                                                        | Covers                                      |
| --------------------------------------------------------------- | ------------------------------------------- |
| [`design-tokens.md`](./07-ui-ux-design/system/design-tokens.md) | Colour, type, space, radius, shadow, themes |

## 3. Components

| Document                                                  | Covers                                               |
| --------------------------------------------------------- | ---------------------------------------------------- |
| [`components.md`](./07-ui-ux-design/system/components.md) | Buttons, file drop, selects, stepper, badges, alerts |

## 4. Layout

| Document                                          | Covers                                             |
| ------------------------------------------------- | -------------------------------------------------- |
| [`layout.md`](./07-ui-ux-design/system/layout.md) | Navbar, footer, content width, breakpoints, routes |

## 5. Screen Designs

| Document                                            | Covers                          |
| --------------------------------------------------- | ------------------------------- |
| [`screens.md`](./07-ui-ux-design/system/screens.md) | Home, New job, Job status, Jobs |

The job status screen also shows the completed-job **Model report**; the chart selection and
rendering rules are in [`charts.md`](./07-ui-ux-design/system/charts.md).

## 6. States

| Document                                          | Covers                   |
| ------------------------------------------------- | ------------------------ |
| [`states.md`](./07-ui-ux-design/system/states.md) | Global and job UI states |

## 7. Accessibility

| Document                                                        | Covers                                     |
| --------------------------------------------------------------- | ------------------------------------------ |
| [`accessibility.md`](./07-ui-ux-design/system/accessibility.md) | Keyboard, screen readers, contrast, motion |

## 8. Responsive Design

| Document                                                  | Covers                             |
| --------------------------------------------------------- | ---------------------------------- |
| [`responsive.md`](./07-ui-ux-design/system/responsive.md) | Viewports and responsive behaviour |

## 9. Animation

| Document                                                | Covers |
| ------------------------------------------------------- | ------ |
| [`animation.md`](./07-ui-ux-design/system/animation.md) | Motion |

## 10. Iconography

| Document                                                    | Covers              |
| ----------------------------------------------------------- | ------------------- |
| [`iconography.md`](./07-ui-ux-design/system/iconography.md) | Icon set and sizing |

## 11. Brand

| Document                                               | Covers                                            |
| ------------------------------------------------------ | ------------------------------------------------- |
| [`brand/README.md`](./07-ui-ux-design/brand/README.md) | Logo marks, wordmark, colours, clear space, usage |

## 12. Prototypes

| Document                                                         | Covers                                             |
| ---------------------------------------------------------------- | -------------------------------------------------- |
| [`prototypes/README.md`](./07-ui-ux-design/prototypes/README.md) | Static HTML prototypes, themes, screenshot tooling |

## 13. Result Charts

| Document                                          | Covers                                                        |
| ------------------------------------------------- | ------------------------------------------------------------- |
| [`charts.md`](./07-ui-ux-design/system/charts.md) | Completed-job charts: selection per task, data, accessibility |

Proposed: the completed-job Model report and the `charts` data contract. Not yet implemented; the
job status screen shows the metrics list until then.
