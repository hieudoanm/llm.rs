# FEATURE-08: UI States (Global)

## Priority

P0

## Description

Every screen implements loading, empty, validation-error, network-error, unexpected-error, and success. DaisyUI `alert`, `loading`, and `skeleton` primitives are required; custom spinners are not.

## States

| State              | Treatment                                      |
| ------------------ | ---------------------------------------------- |
| loading            | Skeleton or button spinner; disable submit     |
| empty              | Illustrated empty + CTA                        |
| validation-error   | Field `text-error` + summary alert             |
| network-error      | Alert with retry                               |
| unexpected-error   | Alert; do not dump raw exception               |
| success            | Toast or inline confirmation; then navigate    |

Polling uses TanStack Query (or SWR) with `refetchInterval` that stops on terminal job status.

## Actionable errors and client states (Phase 12)

Every error alert additionally renders the **guidance** — what the user should do next — and a
primary action, and it never renders a stack trace. The API client applies a request timeout,
retries safe requests with jittered backoff, and cancels in-flight requests when the calling
view unmounts. The catalogue and the contract are normative in
[`FEATURE-18`](./FEATURE-18-Client-Resilience-and-Guidance.md); the rendered treatment is in
[`07-ui-ux-design/system/states.md`](../07-ui-ux-design/system/states.md).
