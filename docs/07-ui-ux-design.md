# llm.rs — UI/UX Design

## 1. Purpose

This document defines the UI/UX design for llm.rs: tokens, components, layout, screens, states,
accessibility, responsive behaviour, motion, iconography, brand, and the static prototypes. It is
the source of truth for the desktop frontend in `packages/desktop`.

The design must be:

- Clear and technical (local ML studio).
- Accessible (WCAG 2.1 AA).
- Responsive (mobile, tablet, desktop).
- Consistent across screens.
- Explicit about loading, empty, validation-error, network-error, unexpected-error, and success.
- Themeable in light and dark via DaisyUI, using the `llm-light` and `llm-dark` themes.

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
| [`og/svg/og.svg`](./07-ui-ux-design/og/svg/og.svg)     | Social card artwork                               |

The brand and the landing page share the warm, dark-first palette: `#0B1018` night surface,
`#111923` panels, `#F7F1E8` cream text, `#9DA8B7` muted text, `#F06B4D` signal orange,
`#FFAC70` warm accent, `#76D5A5` success mint, and `#24303D` borders. The marks use the orange
for the box and the surface colour for the diagonal line; see the brand document for the light
and dark variants.

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
