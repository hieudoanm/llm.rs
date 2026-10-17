# llm — Layout

_Part of [07-ui-ux-design.md](../../07-ui-ux-design.md)._

## Shell

- Top navbar (sticky): the llm [mark](../brand/README.md) linking home, links
  **Jobs** / **Algorithms**.
- Main: `max-w-5xl mx-auto` for forms and detail; `max-w-6xl` for the history table.
- Footer: local-only disclaimer ("Not a production AutoML service").

## Breakpoints

| Name    | Width    | Layout                                                                       |
| ------- | -------- | ---------------------------------------------------------------------------- |
| mobile  | < 768    | Single column, stacked mapping, collapsing navbar with dropdown              |
| tablet  | 768–1023 | Two-column mapping where possible                                            |
| desktop | ≥ 1024   | Upload → mapping → model wizard with the history/preview table at full width |

## Navigation

App Router routes:

- `/` — Home
- `/jobs/new` — New job wizard
- `/jobs` — Jobs history
- `/jobs/[jobId]` — Job status

The navbar uses a DaisyUI `navbar` layout: inline links at `md` and above, and a dropdown menu
below `md`.

## Theme

`<html data-theme='skl-dark'>` is always set — the app ships with a single dark theme.
