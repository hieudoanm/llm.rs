# llm-client (llm) — Product Specification

Product strategy · Functional requirements · Architecture · Data model · Delivery roadmap

> This document is the product-strategy source. The normative implementation set is [`00`–`12`](#normative-documents). Where this document and the numbered documents disagree on implementation detail, the numbered documents govern.

## Normative documents

| #   | Document                                                 | Governs                                                      |
| --- | -------------------------------------------------------- | ------------------------------------------------------------ |
| 00  | [`00-overview.md`](00-overview.md)                       | Product vision and scope                                     |
| 01  | [`01-requirements.md`](01-requirements.md)               | Functional/non-functional requirements (`FR-`, `NFR-`)       |
| 02  | [`02-features.md`](02-features.md)                       | User-visible behaviour                                       |
| 03  | [`03-user-stories.md`](03-user-stories.md)               | User goals and acceptance scenarios                          |
| 04  | [`04-architecture.md`](04-architecture.md)               | Module boundaries, request lifecycle, training jobs          |
| 05  | [`05-data-model.md`](05-data-model.md)                   | Schema, constraints, state transitions                       |
| 06  | [`06-api-specification.md`](06-api-specification.md)     | REST contract, status codes, error envelope                  |
| 07  | [`07-ui-ux-design.md`](07-ui-ux-design.md)               | UI tokens, layout, states, accessibility                     |
| 08  | [`08-tech-stack.md`](08-tech-stack.md)                   | Allowed technologies and dependency policy                   |
| 09  | [`09-testing-strategy.md`](09-testing-strategy.md)       | Test layers, fixtures, acceptance criteria                   |
| 10  | [`10-implementation-plan.md`](10-implementation-plan.md) | Phased delivery plan                                         |
| 11  | [`11-deployment.md`](11-deployment.md)                   | CI and local Docker Compose                                  |
| 12  | [`12-operations.md`](12-operations.md)                   | Operations: backup, monitoring, security, rollback, recovery |

---

## 1. Product vision

**llm-client (llm)** is a local, full-stack studio for training machine-learning models from a CSV.

A user uploads a table with features (**X**) and, when the task requires one, a label (**Y**), or declares experience columns for offline reinforcement learning, chooses an allow-listed algorithm, and receives a trained artefact. **ONNX (`.onnx`) is the required export.** Pickle (`.pkl`) and joblib (`.joblib`) are optional extra saves.

The studio supports three supervision modes behind one workflow:

- **Supervised** — classification and regression with a complete label column.
- **Semi-supervised** — self-training classification where some label cells are empty.
- **Unsupervised** — clustering and anomaly detection with no label column.
- **Offline reinforcement learning** — Fitted Q-Iteration from a transition log with experience columns.

The product demonstrates end-to-end software engineering across a Next.js frontend (Tailwind CSS, DaisyUI), a Rust Axum backend (llm), SQLite for job and artefact metadata, and Docker Compose for local service topology.

## Product principles

**ONNX first** — Every successful training job produces a portable `.onnx` model. Other formats are convenience, not the contract.

**Reproducibility by default** — Record the dataset fingerprint, column mapping, algorithm, hyperparameters, library versions, and metrics with every job.

**Honest local scope** — The first release runs on a single machine with Docker Compose. Cloud, multi-tenant SaaS, and GPU clusters are out of scope.

**Working software first** — Deliver one complete train-and-export workflow before adding another supervision mode.

### The product promise

> Go from a CSV to a downloadable ONNX model on your own machine.

llm is a software demonstration, not a production AutoML platform. It does not claim state-of-the-art accuracy, regulatory compliance, or suitability for safety-critical decisions.

## 2. Target users

| Persona            | Primary need                                    | Desired outcome                                   |
| ------------------ | ----------------------------------------------- | ------------------------------------------------- |
| Practitioner       | Train a model from a small CSV, labelled or not | Download ONNX (and optional extras) with metrics  |
| Developer/Operator | Run the stack locally and inspect failed jobs   | Reproducible Compose topology and job diagnostics |

Primary persona: a developer who wants to understand a complete ML training product — upload, validate, configure a task, train, evaluate, export — without a cloud account.

## 3. Scope and priorities

- **P0** — Core platform: necessary for the first usable release.
- **P1** — Enhancement: important for a complete experience.

The first release should deliver one complete training workflow rather than an incomplete version of every algorithm.

## 4. Functional requirements

See [`01-requirements.md`](01-requirements.md) for the complete FR specification.

### P0 — Core platform

CSV upload, column mapping (X/Y where required), model selection, asynchronous training, task-specific metrics, required ONNX export, job history, health checks.

The implemented P0 catalogue covers llm classification and regression. The following P0 extensions are planned in Phases 9–10:

- **Semi-supervised:** self-training classification with partially labelled CSVs.
- **Unsupervised:** clustering (KMeans, MiniBatchKMeans, GaussianMixture) and anomaly detection (IsolationForest).

### P1 — Enhancement

Configurable tabular MLP hyperparameters (`hidden_layer_sizes`, `activation`, `solver`, `alpha`, `early_stopping`), optional `.pkl` and `.joblib` downloads, typed hyperparameter controls beyond the task-critical fields, sample datasets.

## 5. User journeys

### Journey A — Train a classifier

Goal: turn a labelled CSV into an ONNX classifier.

1. Open the application locally.
2. Upload a CSV with a header row.
3. Map feature columns to **X** and the target column to **Y**.
4. Choose classification and an algorithm (for example logistic regression).
5. Start training and watch job status.
6. Review metrics on the hold-out split.
7. Download the `.onnx` artefact.

Success metric: percentage of valid CSVs that complete with an ONNX download.

### Journey B — Train a regressor

Goal: predict a numeric target and export ONNX.

1. Upload a CSV with a numeric label.
2. Map X and Y.
3. Choose regression and an algorithm.
4. Train, review RMSE/MAE/R², download ONNX.

Success metric: successful regression jobs with metric payload present.

### Journey C — Recover from a bad dataset

Goal: understand why training did not start.

1. Upload an invalid or mismatched CSV.
2. See field-level validation (missing header, empty file, unmapped Y, mixed dtypes).
3. Correct the mapping or file and retry.

Success metric: validation errors are actionable without reading server logs.

### Journey D — Train with partial labels

Goal: use a CSV where only some rows have labels.

1. Upload a CSV whose label column is filled for some rows and empty for others.
2. Map X and the partially labelled Y, then choose **Semi-supervised**.
3. Choose self-training and start training.
4. Review classification metrics measured only on held-out labelled rows.
5. Download the ONNX model.

Success metric: a job completes when at least two classes meet the documented labelled-row minimum.

### Journey E — Cluster an unlabelled CSV

Goal: group rows without a target column.

1. Upload a CSV with no label.
2. Map X, choose **Clustering**, and set the number of clusters.
3. Train and review silhouette, Davies–Bouldin, Calinski–Harabasz, and inertia/BIC.
4. Download the ONNX model.

Success metric: cluster metrics are returned and ONNX predictions match llm.

### Journey F — Detect anomalies

Goal: flag unusual rows without a target column.

1. Upload a CSV with no label.
2. Map X, choose **Anomaly detection**, and accept or set the contamination rate.
3. Train and review the anomaly count, rate, and score distribution.
4. Download the ONNX model.

Success metric: ONNX labels and scores match llm.

## 6. Application structure

| Page       | Purpose                                                      |
| ---------- | ------------------------------------------------------------ |
| Home       | Product intro and start-training call to action              |
| New job    | Upload CSV, map columns, choose task, supervision, and model |
| Job status | Poll training progress, task-specific metrics, downloads     |
| Jobs       | History of local training jobs, with filters, sorting, tags, and bulk selection |
| Compare    | Side-by-side comparison of 2–5 jobs                          |
| Algorithms | Catalogue grouped by task type                               |
| Health     | Surface backend and database readiness (operator)            |

## 7. Technical architecture

### 7.1 System design

llm is a **three-service local stack**: Next.js frontend, Axum training API, SQLite. The API accepts uploads, persists metadata, and runs training **asynchronously** so HTTP handlers do not block on fit/export. Artefact files live on a Docker volume; SQLite stores jobs, column maps, metrics, and artefact pointers. See [`04-architecture.md`](04-architecture.md).

### 7.2 Technology stack

| Layer            | Technology                                       | Responsibility                           |
| ---------------- | ------------------------------------------------ | ---------------------------------------- |
| Frontend         | Next.js (App Router), TypeScript                 | Upload, mapping, job polling, download   |
| Styling          | Tailwind CSS + DaisyUI                           | Responsive styling, design tokens        |
| API              | Rust stable+, Axum, serde                  | Validation, jobs, training orchestration |
| ML               | linfa/ndarray (incl. tabular MLPs), ONNX exporters | Fit, evaluate, export                    |
| Database         | SQLite 18                                    | Jobs, datasets, artefacts, metrics       |
| Files            | Docker volume                                    | CSV blobs and model files                |
| Containerisation | Docker, Docker Compose                           | Local topology                           |
| CI/CD            | GitHub Actions                                   | Lint, typecheck, test                    |
| Testing          | pytest, Vitest, Playwright                       | Unit, integration, E2E                   |

## 8. Current implementation status

This snapshot reflects the repository state on 2026-10-19. It is intentionally separate from the product vision above.

### Implemented

- **Backend** (`crates/llm-api`): Axum app serving health endpoints (`/healthz`, `/readyz`), plus routes under `/api/v1` for algorithms, datasets, and jobs; sqlx with SQLite schema; filesystem storage for uploads and artefacts (`LLM_DATA_DIR`, upload cap `LLM_MAX_UPLOAD_MB`); a background training worker with linfa estimators and ONNX export via `onnx-protobuf`; cargo test covering the algorithm catalogue and the API end to end.
- **Frontend** (`packages/frontend`): Next.js App Router UI — home, job wizard (upload → map columns → configure a model), job history, and job detail with per-metric explanations; light/dark DaisyUI themes; Vitest + React Testing Library units over `src/lib` + `src/components` (80% coverage gate); Playwright e2e over the demo scenes.
- **Notebooks**: removed — Python backend deleted; sample datasets exported directly from the Rust backend.
- **Infrastructure**: `docker-compose.yaml` (SQLite 18, backend, frontend), one Dockerfile per app service, GitHub Actions CI (backend, frontend, e2e), and the Turborepo/Husky workspace tooling.
- **Full learning spectrum**: Supervised (classification, regression), semi-supervised (self-training classification), unsupervised (clustering with KMeans, MiniBatchKMeans, GaussianMixture; anomaly detection with IsolationForest), and offline reinforcement learning (Fitted Q-Iteration from experience logs, Phase 11) — all six task types specified with ONNX export.

### Not yet implemented

- Alembic migrations — removed; schema defined in `crates/llm-db/src/schema.rs`.
- Framework neural nets — the neural-net offering is the configurable tabular linfa/ndarray MLPs.

### Roadmap

Phase 0 Foundation → Phase 1 Data model → Phase 2 Dataset upload → Phase 3 Column mapping → Phase 4 Model catalogue → Phase 5 Training jobs → Phase 6 ONNX export → Phase 7 Job history and optional artefacts → Phase 8 Health, observability, documentation → **Phase 9 Semi-supervised learning** (done) → **Phase 10 Unsupervised learning** (done) → **Phase 11 Reinforcement learning** (planned) → **Phase 12 Hardening & insights** (planned). See [`10-implementation-plan.md`](10-implementation-plan.md).

## 9. Non-functional requirements

See [`01-requirements.md`](01-requirements.md) for the complete NFR specification.

### Key targets

- Upload validation response < 2 s (p95) for files within the documented size limit
- Training job accepted (`202`) < 300 ms (p95) after a valid mapping
- ONNX export present on every successful job
- Local availability: Compose healthchecks for frontend, backend, and SQLite
- Security: no secrets in the repo; bounded upload size; path-safe artefact storage
- Observability: structured JSON logs with `requestId` / `jobId`

## 10. Design and UX

See [`07-ui-ux-design.md`](07-ui-ux-design.md).

### Design principles

- Clear, technical, and calm (studio, not marketing splash)
- Accessible (WCAG 2.1 AA)
- Responsive (mobile, tablet, desktop)
- Explicit loading, empty, validation, and failure states
- Themeable light and dark (DaisyUI)

## 11. Testing strategy

See [`09-testing-strategy.md`](09-testing-strategy.md).

### Test layers

Unit (Vitest + React Testing Library for the frontend and pytest for the backend, both with an 80% coverage gate), Integration (real SQLite and filesystem), Contract (OpenAPI), E2E (Playwright: CSV → ONNX), Security (path traversal, oversized upload), Performance (upload/job accept bounds)

## 12. Deployment

See [`11-deployment.md`](11-deployment.md) and [`12-operations.md`](12-operations.md).

### Options

- **Local only:** Docker Compose with `frontend`, `backend`, and `sqlite`
- **CLI:** `llm` — offline train-and-export binary (see
  [`11-deployment/cli.md`](./11-deployment/cli.md) and
  [`crates/llm-server/scripts/install.sh`](../crates/llm-server/scripts/install.sh))
- CI: GitHub Actions quality gates
- Cloud, Kubernetes, and Terraform are **out of scope** for this release

## 13. Contributing

Keep docs in sync with implementation. Significant decisions belong in `docs/decisions/` when that directory is introduced.

### Repository conventions

- Rust: Ruff, pytest
- TypeScript: ESLint, Prettier, `tsc`
- Tasks and hooks: Turborepo; Husky `pre-commit` (`lint-staged` declared, not wired yet)
- Tests: Vitest (frontend units), pytest, and Playwright
- Documentation: numbered `docs/00`–`12` govern implementation detail

## 14. License

This is a portfolio project for demonstration purposes. Not for production AutoML or safety-critical use.
