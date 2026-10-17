# llm — States

_Part of [07-ui-ux-design.md](../../07-ui-ux-design.md)._

## Global states

See [`FEATURE-08`](../../02-features/FEATURE-08-UI-States-Global.md). Every screen implements:

| State            | Treatment                                               |
| ---------------- | ------------------------------------------------------- |
| loading          | DaisyUI `loading` spinner or `skeleton`; disable submit |
| empty            | Illustrated empty state + CTA                           |
| validation-error | Field-level `text-error` + summary alert                |
| network-error    | `ApiAlert` with retry                                   |
| unexpected-error | `ApiAlert`; never dump the raw exception                |
| success          | Inline confirmation or toast, then navigate             |

### Actionable errors

([FEATURE-18](../../02-features/FEATURE-18-Client-Resilience-and-Guidance.md))

Every error alert renders, in order:

1. The **title** — a short heading from the central error catalogue.
2. The **message** — the server's human-readable message, or the catalogue's default.
3. The **guidance** — what the user should do next. Always present; this is what turns "an
   error happened" into "do this".
4. The **action** — a primary button (`Retry`, `Upload again`, `Back to mapping`, …) when the
   guidance is actionable, and `btn-ghost` otherwise.
5. A collapsible **details** disclosure with extra bullets.
6. A `text-xs` footer with the `requestId`.

The alert never renders a stack trace, a Rust exception repr, or a raw HTML error page.

## Client states

([FEATURE-18](../../02-features/FEATURE-18-Client-Resilience-and-Guidance.md))

| Client condition          | User-visible treatment                                     |
| ------------------------- | ----------------------------------------------------------- |
| Request in flight         | The existing loading treatment for the screen.               |
| Network failure           | `ApiAlert` with the `NETWORK_ERROR` guidance and a retry button. |
| Request timed out         | `ApiAlert` with the `REQUEST_TIMEOUT` guidance and a retry button. |
| Request aborted by the user (navigated away) | Nothing — the abort is silent and must not raise an alert. |
| Retry in progress         | The retry button shows a spinner and is disabled; the user is not told about the internal retry. |
| Retry exhausted           | The alert is shown with the catalogue guidance for the underlying code. |

## Job states

| Job status | Badge   | User copy                         |
| ---------- | ------- | --------------------------------- |
| queued     | warning | Waiting for worker                |
| validating | warning | Checking CSV, mapping, and labels |
| training   | warning | Fitting model                     |
| evaluating | warning | Computing metrics                 |
| exporting  | warning | Writing ONNX                      |
| retrying   | warning | Retrying after a temporary error  |
| completed  | success | Ready to download                 |
| failed     | error   | Use `error.message`               |

A `queued` job may additionally show **"N jobs ahead"** when the worker is at its concurrency
limit.

Polling uses TanStack Query `refetchInterval`, which stops on a terminal status.
