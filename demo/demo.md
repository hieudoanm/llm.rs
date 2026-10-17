# mlrs Demo Script

This document is a scene-by-scene demo script for recording an MP4 video of the mlrs
machine learning platform. It covers every feature of the frontend and backend so
that AGENTS (or a human narrator) can produce a complete walkthrough video.

## Overview

mlrs = scikit-learn-client. A full-stack ML training platform supporting supervised, semi-supervised, and unsupervised learning:

- **Frontend**: Next.js + Tailwind v4 + DaisyUI, light/dark themes (`mlrs-light` / `mlrs-dark`), TanStack Query polling, CSV upload wizard, job lifecycle status, artefact download.
- **Backend**: FastAPI + SQLAlchemy (async) + PostgreSQL, mlrsearn training pipeline, ONNX/pickle/joblib export, file storage, health/readyz endpoints.
- **Database**: PostgreSQL via asyncpg, datasets/jobs/metrics/artefacts/idempotency tables.
- **Branding**: mlrs logo mark (light/dark variants), Geist sans/mono fonts, custom design tokens.

## Prerequisites for Recording

1. SQLite database at `.mlrs-data/mlrs.db`.
2. Backend running: `cargo run -p mlrs-server` from `crates/` (port 8000).
3. Frontend running: `cd packages/frontend && pnpm run dev` (port 3000).
4. Browser at `http://localhost:3000`, screen resolution 1920×1080, dark/light OS theme set to light for the first half.
5. Sample CSV ready at `/tmp/demo-dataset.csv` (classification) and `/tmp/demo-regression.csv` (regression).

Sample CSV (classification, `demo-dataset.csv`):
```
feature1,feature2,feature3,label
1.2,3.4,red,0
2.1,5.6,blue,1
...
```

Sample CSV (regression, `demo-regression.csv`):
```
sqft,bedrooms,age,price
1500,3,10,350000
2000,4,5,450000
...
```

---

## Scene 1 — Title & Branding (0:00–0:15)

**Screen**: Frontend home page, light theme.

**Actions**:s

1. Open browser to `http://localhost:3000`.
2. Show the home page: hero heading "scikit-learn-client", subcopy "Upload a CSV, choose a model, download ONNX."
3. Hover over "Train a model" and "View jobs" buttons (primary and outline variants).
4. Scroll down to the "Everything you need to train" feature grid (CSV upload, Task-aware models, ONNX export).
5. Scroll to "How it works" three-step section (Upload dataset → Map and configure → Train and download).
6. Scroll to the CTA banner "Ready to train your first model?"

**Narrator**: "mlrs is a focused ML training platform. Upload a CSV, pick your features and label, choose a task and algorithm, train, and download ONNX — no Python runtime needed at inference."

---

## Scene 2 — Theme Toggle (0:15–0:30)

**Screen**: Home page, light theme → dark theme.

**Actions**:

1. Click the theme toggle button in the navbar (sun ↔ moon icon).
2. Wait ~1s for the theme transition. Show the page re-render in dark mode (`mlrs-dark`).
3. Click again to return to light mode.

**Narrator**: "mlrs ships with two themes — mlrs-light and mlrs-dark — driven by custom design tokens and DaisyUI. The theme persists across navigation."

---

## Scene 3 — Navbar Navigation (0:30–0:45)

**Screen**: Navbar + home page.

**Actions**:

1. Click "Home" → stays on `/`.
2. Click "New job" → navigates to `/jobs/new`.
3. Click "Jobs" → navigates to `/jobs` (empty state).
4. Click the mlrs logo → returns to `/`.

**Narrator**: "The navbar gives instant access to Home, New job, and Jobs, plus the theme toggle."

---

## Scene 4 — CSV Upload (0:45–1:15)

**Screen**: `/jobs/new`, step 1 "Dataset".

**Actions**:

1. Show the FileDrop zone: "Drag & drop your CSV here" with upload icon.
2. Drag `demo-dataset.csv` onto the dropzone (or click browse and select).
3. Show the upload spinner ("Uploading CSV…").
4. On success, the dataset table appears showing columns: feature1, feature2, feature3, label with inferred dtypes and null counts.
5. Show the filename + row count pill ("demo-dataset.csv (N rows)").
6. Click "Change file" → clears selection, then re-upload the same file.

**Narrator**: "Step one: upload a CSV. Files are validated (UTF-8, non-empty, max 10 MB), hashed with SHA-256, and persisted. Columns are previewed with inferred dtypes before you map features and labels."

---

## Scene 5 — Column Mapping (1:15–1:45)

**Screen**: `/jobs/new`, step 2 "Mapping".

**Actions**:

1. Click "Continue to mapping".
2. Show feature column checkboxes: check feature1, feature2, feature3 (leave label unchecked).
3. Show the Label column dropdown: select "label".
4. Show the Task dropdown: switch between "Classification" and "Regression" (note the hint "For regression, the label column must be numeric").
5. Click "Continue to model".

**Narrator**: "Step two: map your data. Select feature columns, the label column, and the task type. Validation ensures the label isn't also a feature, and for regression the label must be numeric."

---

## Scene 6 — Model Configuration (1:45–2:15)

**Screen**: `/jobs/new`, step 3 "Model".

**Actions**:

1. Show the Algorithm dropdown populated with classification algorithms: logistic_regression, random_forest_classifier, gradient_boosting_classifier.
2. Select "random_forest_classifier".
3. Show export checkboxes: pickle, joblib. Check pickle and joblib.
4. Click "Start training".
5. Show the "Training failed" ApiAlert if validation error occurs (optional: demonstrate by submitting with no algorithm selected).

**Narrator**: "Step three: choose your algorithm and export formats. Algorithms are filtered by task type. You can export ONNX (always), plus optional pickle and joblib bundles."

---

## Scene 7 — Job Lifecycle & Polling (2:15–3:30)

**Screen**: `/jobs/{jobId}`, job status page.

**Actions**:

1. After clicking "Start training", the page redirects to the job detail.
2. Show the status progress bar (steps): queued → validating → training → evaluating → exporting → completed.
3. Watch the StatusBadge change color (warning → success) and label text ("Waiting for worker" → "Fitting model" → "Scoring hold-out split" → "Writing ONNX" → "Ready to download").
4. When complete, show the Metrics table (accuracy, precisionMacro, recallMacro, f1Macro for classification).
5. Show the Artefacts section: model.onnx (primary button), model.pkl, model.joblib (ghost buttons).
6. Click the ONNX download link → browser downloads the file.

**Narrator**: "Training runs in the background through a six-stage lifecycle: queued, validating, training, evaluating, exporting, then completed or failed. The frontend polls every second until terminal. When done, you see metrics and can download ONNX, pickle, or joblib artefacts."

---

## Scene 8 — Jobs History (3:30–4:00)

**Screen**: `/jobs`.

**Actions**:

1. Click "Back to jobs" or navigate to `/jobs`.
2. Show the jobs table listing the completed job with timestamp, status badge (completed), task type, algorithm.
3. Click "Open" on the job row → returns to the job detail page.
4. (Optional) Upload a second regression dataset and train a linear_regression job to show a second row in the table.

**Narrator**: "The Jobs page lists all training runs with cursor pagination. Click Open to revisit any job's status, metrics, and artefacts."

---

## Scene 9 — Artefact Download & Error Handling (4:00–4:30)

**Screen**: Job detail page + error demo.

**Actions**:

1. On a completed job, click each artefact download link (onnx, pkl, joblib) and show the browser download starting.
2. Demonstrate a validation error: start a new job, on step 3 submit with no algorithm selected → ApiAlert shows "Training failed" with the backend error message.
3. Demonstrate 404: navigate to `/jobs/00000000-0000-0000-0000-000000000000` → ApiAlert "Job not found."

**Narrator**: "Artefacts are served directly from the backend with proper content types. Validation errors return structured ApiError responses, and missing jobs show a clear 404."

---

## Scene 10 — Backend API & Health (4:30–5:00)

**Screen**: Browser + terminal (split or picture-in-picture).

**Actions**:

1. Open `http://127.0.0.1:8000/healthz` → `{"status":"ok"}`.
2. Open `http://127.0.0.1:8000/readyz` → `{"status":"ok"}` (PostgreSQL reachable).
3. Open `http://127.0.0.1:8000/api/v1/algorithms?taskType=classification` → JSON list of classification algorithms with default hyperparameters and supported exports.
4. Open `http://127.0.0.1:8000/api/v1/algorithms?taskType=regression` → regression algorithms.
5. Show terminal with uvicorn logs as requests come in.

**Narrator**: "The backend exposes health probes, an algorithm catalogue, dataset upload, job CRUD, and artefact download — all backed by PostgreSQL."

---

## Scene 11 — Regression Workflow (5:00–5:45)

**Screen**: Full end-to-end regression training.

**Actions**:

1. New job → upload `demo-regression.csv`.
2. Map features: sqft, bedrooms, age; label: price; task: Regression.
3. Model: linear_regression, export ONNX only.
4. Start training → watch lifecycle → completed.
5. Show regression metrics: rmse, mae, r2.
6. Download model.onnx.

**Narrator**: "mlrs supports both classification and regression. Regression jobs validate that the label is numeric and produce RMSE, MAE, and R² metrics."

---

## Scene 12 — Dark Theme Full Walkthrough (5:45–6:15)

**Screen**: Repeat key flows in dark theme.

**Actions**:

1. Toggle to dark theme.
2. Quick pass through: home page → new job → upload → mapping → model → job status → jobs list.
3. Show that all components (badges, alerts, cards, buttons, dropzone) render correctly in dark mode.
4. Toggle back to light theme at the end.

**Narrator**: "Every component adapts to mlrs-dark. The design tokens ensure consistent contrast and branding in both themes."

---

## Scene 13 — Outro (6:15–6:30)

**Screen**: Home page, light theme.

**Actions**:

1. Return to `/`.
2. Show hero section + CTA "Start training".
3. Fade out.

**Narrator**: "From CSV to ONNX in under a minute. That's mlrs — scikit-learn-client."

---

## Timing Summary

| Scene | Duration  | Content                    |
| ----- | --------- | -------------------------- |
| 1     | 0:00–0:15 | Title & branding           |
| 2     | 0:15–0:30 | Theme toggle               |
| 3     | 0:30–0:45 | Navbar navigation          |
| 4     | 0:45–1:15 | CSV upload                 |
| 5     | 1:15–1:45 | Column mapping             |
| 6     | 1:45–2:15 | Model configuration        |
| 7     | 2:15–3:30 | Job lifecycle & polling    |
| 8     | 3:30–4:00 | Jobs history               |
| 9     | 4:00–4:30 | Artefact download & errors |
| 10    | 4:30–5:00 | Backend API & health       |
| 11    | 5:00–5:45 | Regression workflow        |
| 12    | 5:45–6:15 | Dark theme walkthrough     |
| 13    | 6:15–6:30 | Outro                      |

**Total**: ~6:30

---

## Recording Notes for AGENTS

- Record at 1920×1080, 30 fps.
- Use a smooth mouse cursor; pause ~0.5s after each click for clarity.
- Highlight the element being interacted with (optional cursor spotlight).
- Keep the browser address bar visible so viewers can see the URLs.
- If a job takes longer than ~10s to complete, use jump cuts between lifecycle stages.
- For the backend API scene, use `curl` or the browser address bar directly.
- Ensure both `demo-dataset.csv` and `demo-regression.csv` are in `/tmp/` before starting.
- The backend must be running on port 8000 and the frontend on port 3000 before recording begins.
