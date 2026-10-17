# llm-client (llm) — Features

## 1. Purpose

This document describes how llm features should behave from the user's perspective and how their behaviour should be implemented.

It complements:

- `00-overview.md` — product vision and scope.
- `01-requirements.md` — functional and non-functional requirements.
- `03-user-stories.md` — user goals and acceptance scenarios.
- `04-architecture.md` — system boundaries and technical responsibilities.
- `05-data-model.md` — entities, relationships, and constraints.
- `06-api-specification.md` — REST endpoints and request/response models.
- `08-tech-stack.md` — technology choices and rationale.
- `07-ui-ux-design.md` — interface and interaction design.

Features should be delivered in vertical slices: a user should complete CSV upload → mapping → train → ONNX download through the frontend, API, database, and worker before the next major workflow is started.

The canonical entities are `datasets`, `dataset_insights`, `jobs`, `job_metrics`, and `artefacts`.

llm is a software demonstration. It does not provide production AutoML or safety-critical guarantees. The reinforcement-learning extension (Phase 11) fits the same local-only constraints. The Phase 12 features (FEATURE-13..FEATURE-18) harden the platform and add discovery, comparison, and insight features on top of the same workflow.

Each feature section states **user-visible behaviour**, **inputs**, **outputs**, **states**, **edge cases**, and **acceptance conditions**. Priority maps to P0–P1 and to `FR-` identifiers.

## 2. Feature Priorities

| Priority | Meaning                          | Product stage                 |
| -------- | -------------------------------- | ----------------------------- |
| P0       | Required for the committed scope | Core platform and Phases 9–11 |
| P1       | Important for a complete studio  | Enhancement                   |

## 3. Feature Completion Definition

A feature is complete when: behaviour documented, FE/BE implement the contract, validation/failure handled, tests pass, migrations added, logs adequate, docs updated, quality gates pass.

## 4. MVP Definition of Done

MVP ready when the core workflow works end-to-end: upload CSV → map X/Y → choose llm model → train asynchronously → view metrics → download `.onnx`. Error handling complete, UI states implemented, health checks in place.

The semi-supervised and unsupervised extensions (FEATURE-09 and FEATURE-10) reuse the same workflow with a conditional label column and task-specific metrics. They are delivered in Phases 9–10. FEATURE-11 (reinforcement learning) reuses the same workflow with experience columns instead of a label; it is delivered in Phase 11.

## 5. Features

Each feature is documented in a separate file:

- [FEATURE-01: Dataset Upload](./02-features/FEATURE-01-Dataset-Upload.md)
- [FEATURE-02: Column Mapping](./02-features/FEATURE-02-Column-Mapping.md)
- [FEATURE-03: Model Selection](./02-features/FEATURE-03-Model-Selection.md)
- [FEATURE-04: Training Job](./02-features/FEATURE-04-Training-Job.md)
- [FEATURE-05: Model Export](./02-features/FEATURE-05-Model-Export.md)
- [FEATURE-06: Job History](./02-features/FEATURE-06-Job-History.md)
- [FEATURE-07: Error Handling](./02-features/FEATURE-07-Error-Handling.md)
- [FEATURE-08: UI States (Global)](./02-features/FEATURE-08-UI-States-Global.md)
- [FEATURE-09: Semi-Supervised Learning](./02-features/FEATURE-09-Semi-Supervised-Learning.md)
- [FEATURE-10: Unsupervised Learning](./02-features/FEATURE-10-Unsupervised-Learning.md)
- [FEATURE-11: Reinforcement Learning](./02-features/FEATURE-11-Reinforcement-Learning.md)
- [FEATURE-12: Metrics, Charts & Download](./02-features/FEATURE-12-Metrics-Charts-Download.md)
- [FEATURE-13: Job Discovery](./02-features/FEATURE-13-Job-Discovery.md) — search, filters, sorting, tags, rename
- [FEATURE-14: Job Comparison](./02-features/FEATURE-14-Job-Comparison.md) — side-by-side comparison with warnings
- [FEATURE-15: Bulk Artefact Download](./02-features/FEATURE-15-Bulk-Artefact-Download.md) — ZIP bundles (per-job and multi-job)
- [FEATURE-16: Dataset Insights](./02-features/FEATURE-16-Dataset-Insights.md) — column profiles, warnings, preview
- [FEATURE-17: Reliable Job Execution](./02-features/FEATURE-17-Reliable-Job-Execution.md) — bounded concurrency, timeout, retries, recovery
- [FEATURE-18: Client Resilience & Actionable Guidance](./02-features/FEATURE-18-Client-Resilience-and-Guidance.md) — resilient fetch, error catalogue, wizard feedback

## 6. Feature → requirement map

| Feature | Requirements   | Phase | Notes                                                              |
| ------- | -------------- | ----- | ------------------------------------------------------------------ |
| FEATURE-01 | FR-001..009 | 2    | Phase 12 adds content-type and magic-byte validation.            |
| FEATURE-02 | FR-010..015 | 3    | —                                                                  |
| FEATURE-03 | FR-020..027 | 4    | —                                                                  |
| FEATURE-04 | FR-030..040 | 5    | Phase 12 adds the timeout, concurrency, and retry contract.       |
| FEATURE-05 | FR-050..057 | 6    | Phase 12 adds the ZIP bundle.                                     |
| FEATURE-06 | FR-060..063 | 7    | Phase 12 adds filtering, sorting, tags, and the comparison view.   |
| FEATURE-07 | —            | 8    | Phase 12 adds `UNSUPPORTED_FILE_TYPE`, `MALFORMED_CSV`, `JOB_TIMEOUT`, `JOB_RETRY_EXHAUSTED`. |
| FEATURE-08 | —            | 8    | —                                                                  |
| FEATURE-09 | FR-080..086 | 9    | —                                                                  |
| FEATURE-10 | FR-090..105 | 10   | —                                                                  |
| FEATURE-11 | FR-110..115 | 11   | —                                                                  |
| FEATURE-12 | —            | 11   | —                                                                  |
| FEATURE-13 | FR-120..127 | 12   | New: search, filter, sort, tags.                                  |
| FEATURE-14 | FR-130..134 | 12   | New: side-by-side comparison.                                     |
| FEATURE-15 | FR-140..144 | 12   | New: ZIP bundles.                                                  |
| FEATURE-16 | FR-150..156 | 12   | New: column profiles and warnings.                                |
| FEATURE-17 | FR-160..169 | 12   | New: bounded, interruptible, retryable worker.                    |
| FEATURE-18 | FR-170..178 | 12   | New: resilient client, actionable errors, wizard feedback.        |
