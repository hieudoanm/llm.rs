# llm-client (llm) — Project Overview

## 1. Project Summary

Support supervised, semi-supervised, unsupervised, and offline reinforcement-learning tasks on tabular CSVs. A user uploads features (**X**) and, when the task needs one, a label (**Y**), or declares experience columns for offline reinforcement learning, chooses an allow-listed algorithm, and downloads a trained model. The **canonical artefact is ONNX (`.onnx`)**. Pickle (`.pkl`) and joblib (`.joblib`) are optional.

The project showcases end-to-end engineering: Next.js with Tailwind CSS and DaisyUI, Axum with llm, SQLite, and Docker Compose on a single machine.

llm is an independent portfolio project. It must not copy proprietary AutoML products, branding, source code, or confidential workflows.

## 2. Problem Statement

Training a first machine-learning model still involves too many disconnected steps: cleaning a CSV, choosing an estimator, fitting, evaluating, and exporting something another runtime can load.

A useful local studio needs to make it possible to:

- Upload a CSV with a header row and inspect inferred columns.
- Map feature columns to **X** and, for supervised or semi-supervised tasks, the target to **Y**. For offline reinforcement learning, map state features to **X** and declare action/reward/next-state or episode columns in the experience mapping.
- Choose a task — classification, regression, semi-supervised (self-training), clustering, anomaly detection, or reinforcement learning (offline FQI) — and an algorithm.
- Train asynchronously with visible job status.
- Evaluate with task-appropriate metrics: a hold-out for supervised, semi-supervised, and offline RL jobs (episodes or transitions); in-sample cluster or anomaly scores for unsupervised jobs.
- Export **ONNX** every time, with optional llm pickle or joblib saves.
- Keep a local history of datasets, jobs, metrics, and artefact paths.
- Run the whole system with Docker Compose: frontend, backend, SQLite.

llm brings these workflows together in one coherent application.

## 3. Product Vision

Build a polished, production-style local ML studio that demonstrates how modern full-stack engineering can support a complete learning loop.

The product should be sufficiently complete that a reviewer can upload a sample CSV, choose a task, train, and download `.onnx` without using a notebook, the database, or the API by hand.

The project should prioritise correctness of the training contract, artefact integrity, and operational clarity over the number of algorithms.

## 4. Target Users

### 4.1 Practitioner

A person who has a tabular dataset and wants a portable model.

Primary needs:

- Upload a CSV with a header row.
- Declare which columns are features and, when the task requires one, which column is the label.
- Pick a familiar algorithm with safe defaults.
- See whether training succeeded and how it scored with task-appropriate metrics.
- Download ONNX for use in another runtime.

### 4.2 Developer or Operator

A person responsible for running llm locally.

Primary needs:

- Start frontend, backend, and SQLite with Compose.
- Diagnose failed jobs from logs and job error payloads.
- Apply schema migrations safely.
- Back up the Postgres volume and artefact volume.
- Confirm health of each service.

## 5. Product Goals

### Goal 1: Deliver a complete training workflow

A user can move from CSV upload to ONNX download inside the web application.

### Goal 2: Demonstrate full-stack ownership

The implementation covers frontend development, API design, relational modelling, ML export, testing, and containerised local deployment.

### Goal 3: Make ONNX the contract

Every successful job writes a valid `.onnx` file. Optional formats never replace it.

### Goal 4: Keep training out of the request thread

HTTP handlers validate and enqueue. Fit, evaluate, and export run in a worker so the API stays responsive.

### Goal 5: Record enough to reproduce a job

Dataset hash, column map, algorithm id, hyperparameters, library versions, split seed, and metrics are stored with the job.

### Goal 6: Provide a maintainable codebase

Clear package boundaries, automated tests, reproducible Compose setup, and documented engineering decisions.

### Goal 7: Cover the learning spectrum

Support supervised, semi-supervised, unsupervised, and offline reinforcement-learning tasks behind one upload → configure → train → export workflow, while keeping ONNX as the single export contract.

## 6. Non-Goals

The initial release will not:

- Support online reinforcement learning, sequence models, or time-series forecasting. Offline reinforcement learning from experience logs is in scope for Phase 11 and is specified in [`FEATURE-11`](./02-features/FEATURE-11-Reinforcement-Learning.md).
- Add estimators without a verified ONNX path — this currently excludes DBSCAN, AgglomerativeClustering, LabelSpreading, and LabelPropagation.
- Provide a hosted multi-tenant SaaS, accounts, or billing.
- Deploy to Kubernetes, AWS, or GCP.
- Train on GPUs or distributed clusters.
- Implement a feature store, experiment tracker (MLflow), or model registry SaaS.
- Serve online inference at scale (a later optional `/predict` may exist for smoke tests only).
- AutoML / Bayesian hyperparameter search.
- Handle datasets larger than the documented upload limit.
- Claim production accuracy, fairness audits, or regulatory compliance.

These features may be considered later, but they must not compromise the initial delivery.

## 7. Training Data

The demonstration uses user-supplied or synthetic CSVs. No production personal data should be committed to the repository.

The application must preserve, for every job:

- Dataset identifier and content hash (SHA-256).
- Column names, dtypes, and the X/Y mapping (Y is absent for unsupervised and reinforcement-learning tasks).
- Task type (`classification`, `regression`, `semi_supervised`, `clustering`, `anomaly_detection`, `reinforcement_learning`) and algorithm id.
- For `reinforcement_learning`: experience column mapping in `rl_columns` (action, reward, next-state or episode, optional done).
- Labelled and unlabelled row counts for semi-supervised jobs; the stored CSV preserves the empty-cell mask.
- Hyperparameters JSON and random seed.
- Train/test split ratio and seed (used for evaluation by supervised, semi-supervised, and reinforcement-learning jobs; unsupervised jobs ignore it).
- Task-specific metrics JSON.
- Artefact records (`onnx` required; `pkl`, `joblib` optional).
- Job status and timestamps (ISO 8601 UTC).
- Correlation / request id for tracing.

## 8. Primary User Workflow

1. A user opens the local web application.
2. The user uploads a CSV (header required).
3. llm stores the file, computes a hash, and returns inferred columns.
4. The user maps feature columns to **X** and, when the task requires one, exactly one column to **Y**. For `reinforcement_learning`, the user maps state features to **X** and declares `rlColumns` (action, reward, and either next-state columns or an episode column).
5. The user selects a task — classification, regression, semi-supervised, clustering, anomaly detection, or reinforcement learning — and an algorithm from the catalogue.
6. The user starts a training job (optional hyperparameter overrides).
7. The API returns `202 Accepted` with a `jobId` and status `queued`.
8. A worker validates the mapping against the file, fits the model, evaluates with task-appropriate metrics, and exports ONNX.
9. The user polls job status until `completed` or `failed`.
10. On success, the user downloads `.onnx` and optionally `.pkl` or `.joblib`.

Every stage should provide loading, validation, progress, and recovery feedback.

## 9. Technology Direction

- **Frontend:** Next.js (App Router) with TypeScript; Tailwind CSS and DaisyUI.
- **Backend:** Rust stable+, Axum, serde, sqlx with SQLite.
- **ML:** linfa/ndarray for all estimators, including tabular neural nets; ONNX export via `onnx-protobuf` (pinned versions).
- **Database:** SQLite 18 for metadata.
- **Files:** named Docker volume for uploads and artefacts.
- **Containerisation:** one Dockerfile for the frontend, one for the backend; Compose wires both.
- **CI/CD:** GitHub Actions (lint, typecheck, test). No cloud deploy.
- **Testing:** cargo test (backend module and API boundary), Vitest + React Testing Library (frontend units, 80% coverage gate), Playwright E2E.

Local commits run Husky's `pre-commit` hook (`pnpm turbo run lint`, then `pnpm turbo run format`); `lint-staged` is declared but not wired up yet. CI is the authoritative quality gate.

## 10. Security and Privacy Principles

Security is part of the architecture even though the first deployment is local-only.

The application must:

- Bound upload size and reject non-CSV content types.
- Store files outside the web root with generated names (never user-supplied paths).
- Use parameterised SQL (no string interpolation).
- Keep secrets (Postgres password) out of source control and frontend bundles.
- Avoid logging raw dataset rows by default.
- Validate algorithm ids against an allow-list (no arbitrary pickle load of user models).
- Treat `.pkl` as an optional export only; never unpickle untrusted user files.

llm must not claim GDPR certification, SOC 2, or production MLOps compliance.

## 11. MVP Definition of Done

The canonical release definition of done lives in [`10-implementation-plan/definition-of-done.md`](10-implementation-plan/definition-of-done.md); this section summarises it and defers to it.

The minimum viable product is complete when:

- A developer can run the stack with documented Docker Compose instructions.
- A user can upload a valid CSV and see inferred columns.
- A user can map X and, when required, Y, then start a llm job.
- Training runs asynchronously and status is pollable.
- A successful job always produces `.onnx`.
- Metrics for the declared task type are returned.
- Invalid CSVs and failed fits produce understandable errors.
- Automated tests cover the core workflow.
- CI runs lint, typecheck, and tests.
- Local operational limits are documented.

The semi-supervised and unsupervised tasks planned in Phases 9–10 extend the studio after the MVP; they do not change the MVP contract above. Reinforcement learning (Phase 11) reuses the same contract with experience columns instead of a label.

## 12. Success Criteria

| Area            | Success criterion                                                               |
| --------------- | ------------------------------------------------------------------------------- |
| User experience | A reviewer completes CSV → ONNX without developer assistance.                   |
| Frontend        | Typed Next.js UI with loading, empty, error, and success states.                |
| Backend         | Documented REST endpoints with serde models.                                 |
| Database        | Versioned SQL migrations and parameterised queries.                             |
| ML contract     | Every success path writes ONNX; optional formats are gated by algorithm family. |
| Testing         | Repeatable checks covering validation, training, and export.                    |
| DevOps          | Compose topology with healthchecks for three services.                          |
| Portfolio       | Clear README, architecture diagram, demo CSV, and documented trade-offs.        |

## 13. Guiding Principles

1. **Working software first:** one complete vertical slice before expanding the catalogue.
2. **ONNX is the product contract;** pickle and joblib are optional.
3. **Training is asynchronous:** never fit inside a request handler.
4. **Explicit contracts:** OpenAPI, serde, and SQL migrations.
5. **Allow-listed algorithms only.**
6. **Simple architecture:** frontend, backend, SQLite — no extra brokers for MVP.
7. **Reproducibility:** hash, mapping, params, versions, seed.
8. **Evidence over claims:** document only implemented behaviour.
9. **Incremental delivery:** every milestone is runnable with Compose.
10. **No production ML claims:** llm is a software demonstration.
