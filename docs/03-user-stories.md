# llm-client (llm) — User Stories

## 1. Purpose

This document describes user stories for llm, expressing user goals in terms of role, motivation, and acceptance criteria.

It complements `00-overview.md`, `01-requirements.md`, `02-features.md`, `04-architecture.md`, `05-data-model.md`, and `06-api-specification.md`.

## 2. User Roles

| Persona            | Primary goal                                     |
| ------------------ | ------------------------------------------------ |
| Practitioner       | Train a model from a CSV and export ONNX         |
| Developer/Operator | Run Compose, inspect health, recover failed jobs |

llm is a **local single-user** studio. There are no accounts in the initial release.

## User Stories

- [US-001: Upload a CSV](./03-user-stories/US-001.md)
- [US-002: Map features (X) and label (Y)](./03-user-stories/US-002.md)
- [US-003: Choose a task and model](./03-user-stories/US-003.md)
- [US-004: Start a training job](./03-user-stories/US-004.md)
- [US-005: Monitor training status](./03-user-stories/US-005.md)
- [US-006: Download the ONNX model](./03-user-stories/US-006.md)
- [US-007: Download optional pickle or joblib artefacts](./03-user-stories/US-007.md)
- [US-008: Review evaluation metrics](./03-user-stories/US-008.md)
- [US-009: Browse job history](./03-user-stories/US-009.md)
- [US-010: Handle invalid CSV uploads](./03-user-stories/US-010.md)
- [US-011: Handle training failures](./03-user-stories/US-011.md)
- [US-012: Monitor system health](./03-user-stories/US-012.md)
- [US-013: Train with partial labels](./03-user-stories/US-013.md)
- [US-014: Cluster an unlabelled dataset](./03-user-stories/US-014.md)
- [US-015: Detect anomalies without labels](./03-user-stories/US-015.md)
- [US-016: Fit a Q-function from an experience log](./03-user-stories/US-016.md)
