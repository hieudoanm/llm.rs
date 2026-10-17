# Phase 5: Training jobs

Weeks 3–4. `POST /jobs` 202, worker claim, llm fit, metrics, poll UI stepper. Jobs may complete without ONNX **only in a feature branch**; do not merge to main without Phase 6 unless status stays `exporting`. Prefer implementing Phase 6 in the same slice if time allows.

## Acceptance

Poll reaches `evaluating` then waits for export in Phase 6.
