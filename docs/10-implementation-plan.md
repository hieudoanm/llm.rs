# llm-client (llm) — Implementation Plan

## 1. Purpose

Phased delivery over approximately 8 weeks: the MVP in Phases 0–8 (6 weeks), then the semi-supervised and unsupervised extensions in Phases 9–10 (2 weeks), then offline reinforcement learning in Phase 11 (1 week). Each phase is runnable with Docker Compose once Phase 0 exists.

## 2. Principles

Vertical slices, ONNX contract first, tests alongside code, docs updated, incremental Compose.

## 3. Phases Overview

| Phase | Document                                                                                                  | Focus                         | Duration  |
| ----- | --------------------------------------------------------------------------------------------------------- | ----------------------------- | --------- |
| 0     | [`phase-0-foundation.md`](./10-implementation-plan/phase-0-foundation.md)                                 | Monorepo, Compose, CI         | Week 1    |
| 1     | [`phase-1-data-model.md`](./10-implementation-plan/phase-1-data-model.md)                                 | Alembic schema                | Week 1    |
| 2     | [`phase-2-dataset-upload.md`](./10-implementation-plan/phase-2-dataset-upload.md)                         | Upload API + UI               | Week 2    |
| 3     | [`phase-3-column-mapping.md`](./10-implementation-plan/phase-3-column-mapping.md)                         | Mapping UI/validation         | Week 2    |
| 4     | [`phase-4-model-catalogue.md`](./10-implementation-plan/phase-4-model-catalogue.md)                       | Algorithms API                | Week 3    |
| 5     | [`phase-5-training-jobs.md`](./10-implementation-plan/phase-5-training-jobs.md)                           | Worker, metrics               | Weeks 3–4 |
| 6     | [`phase-6-onnx-export.md`](./10-implementation-plan/phase-6-onnx-export.md)                               | Required ONNX                 | Week 4    |
| 7     | [`phase-7-history-optional-artefacts.md`](./10-implementation-plan/phase-7-history-optional-artefacts.md) | History, pkl/joblib            | Week 5    |
| 8     | [`phase-8-health-docs.md`](./10-implementation-plan/phase-8-health-docs.md)                               | Observability, demo           | Week 6    |
| 9     | [`phase-9-semi-supervised-learning.md`](./10-implementation-plan/phase-9-semi-supervised-learning.md)     | Self-training, partial labels | Week 7    |
| 10    | [`phase-10-unsupervised-learning.md`](./10-implementation-plan/phase-10-unsupervised-learning.md)         | Clustering, anomaly detection | Week 8    |
| 11    | [`phase-11-reinforcement-learning.md`](./10-implementation-plan/phase-11-reinforcement-learning.md)         | Offline RL from experience log | Week 9    |
| 12    | [`phase-12-hardening-and-insights.md`](./10-implementation-plan/phase-12-hardening-and-insights.md)             | Config, worker reliability, discovery, insights | Week 10 |

## 4. Proposed Enhancements

The following features are fully specified but not yet scheduled for implementation:

- **Model report with result charts** — See [`07-ui-ux-design/system/charts.md`](./07-ui-ux-design/system/charts.md). Adds task-specific charts (confusion matrix, precision/recall bars, predicted vs actual, etc.) to the completed job status screen. Data model and API are updated (`job_metrics.charts` column, `charts` field in poll response), and the prototype includes the section. Implementation would require the worker to collect bounded evaluation arrays and the frontend to render them with visx.

Phase 12 delivers the hardening and insight work as a single vertical slice:
[`phase-12-hardening-and-insights.md`](./10-implementation-plan/phase-12-hardening-and-insights.md).

## 5. Milestones

See [`milestones.md`](./10-implementation-plan/milestones.md).

## 6. Risks

See [`risk-mitigation.md`](./10-implementation-plan/risk-mitigation.md).

## 7. Definition of Done

[`definition-of-done.md`](./10-implementation-plan/definition-of-done.md).

## 8. Dependency Graph

```
Phase 0 → 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10 → 11 → 12
```

Phase 9 extends the label rules, data model, catalogue, worker, API, and UI.
Phase 10 reuses that plumbing and adds the label-less task strategies. Phase 11
adds experience-column plumbing and offline RL. Phase 12 centralises configuration,
hardens the upload and the worker, and adds job discovery, comparison, bulk download, and
dataset insights. All phases preserve the ONNX-on-success contract and the existing
supervised behaviour.

## 9. Cross-references

- Requirements [`01-requirements.md`](01-requirements.md)
- Architecture [`04-architecture.md`](04-architecture.md)
