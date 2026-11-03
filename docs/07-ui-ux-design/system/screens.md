# llm — Screen Designs

_Part of [07-ui-ux-design.md](../../07-ui-ux-design.md)._

## Home (`/`)

- Hero: heading "llm-client", sub "Upload a CSV, choose a model, download ONNX."
- Primary CTA: "Train a model" → `/jobs/new`. Secondary: "View jobs" → `/jobs`.
- Below the fold: a three-card feature grid (CSV upload, task-aware models, ONNX export) and a
  three-step "How it works" strip, then a closing CTA band.

## New job (`/jobs/new`)

Wizard (one page, three cards) with a `steps` indicator:

1. **Dataset** — file drop, then a column preview table (name, inferred dtype, missing count,
   sample values).
2. **Mapping** — feature-column checkboxes; task select grouped by supervision:
   - Supervised: Classification / Regression.
   - Semi-supervised: Semi-supervised.
   - Unsupervised: Clustering / Anomaly detection.

   The label select is required for supervised and semi-supervised tasks and hidden for
   unsupervised tasks. Semi-supervised shows the number of labelled and unlabelled rows and the
   helper "Leave a label cell empty to treat that row as unlabelled." Unsupervised shows
   "No label column is used."

3. **Model** — algorithm select filtered by task, optional export checkboxes (`pickle`, `joblib`),
   and task-critical hyperparameters:
   - Semi-supervised: threshold, max iterations.
   - Clustering: number of clusters (2 to rows − 1, default 3).
   - Anomaly detection: contamination (`auto` or 0 < value ≤ 0.5).

   Start training.

States: uploading, validation-error (CSV only, size limit, label rules), network-error,
unexpected-error. Success navigates to `/jobs/{jobId}`.

**Validation feedback** ([FEATURE-18](../../02-features/FEATURE-18-Client-Resilience-and-Guidance.md)):

- Each step validates live. Problems are shown as field-level `text-error` messages linked to
  the control with `aria-describedby`, and as a collapsible **"Fix {n} issues to continue"**
  summary panel whose entries are buttons that focus the offending control.
- The step indicator shows `complete`, `current`, or `blocked` per step.
- The primary action is **never disabled** on an invalid step. Clicking it expands the summary
  and moves focus to the first invalid control.
- The wizard's in-progress state is persisted to `sessionStorage` and restored on reload with a
  dismissible "Restored your unfinished job setup." notice.
- The hyperparameter form is schema-driven: the catalogue's `min`/`max`/`step` become a Zod schema
  that both the form and the validation summary share.
- **Alt+→** advances to the next step (validation-gated), **Alt+←** returns, and **Alt+1**/*2*/*3*
  jump to a step by number. Refreshing mid-wizard raises the browser's unsaved-changes confirmation.

**Dataset insights** ([FEATURE-16](../../02-features/FEATURE-16-Dataset-Insights.md)):

- After upload, the dataset card shows a **Dataset insights** panel: row/column counts, total
  missing cells, a warning list with suggested actions, a per-column table (inferred type,
  missing count and rate, distinct count, numeric summary or most frequent values), and the
  first 10 data rows.
- The panel is advisory; it never blocks starting a job.

## Job status (`/jobs/[jobId]`)

- `steps` for the lifecycle (queued → validating → training → evaluating → exporting → completed),
  with a failed state that highlights the error.
- Status summary cards (status, task, algorithm, created).
- Metrics as a definition list (`<dl>`) on completed: each score is labelled with its
  plain-language meaning and direction (see [FEATURE-04](../../02-features/FEATURE-04-Training-Job.md)).
  - Classification / semi-supervised: accuracy, precision, recall, F1; semi-supervised also shows
    labelled/unlabelled rows and iterations.
  - Regression: RMSE, MAE, R².
  - Clustering: silhouette, Davies–Bouldin, Calinski–Harabasz, inertia/BIC.
  - Anomaly detection: anomaly count, rate, mean score, score standard deviation.
- **Model report** on completed: the task-specific charts (see [`charts.md`](./charts.md)),
  rendered between Metrics and Downloads. Charts are hidden when their data is absent and never
  replace the metrics list. Each chart card has a download dropdown (hover/focus) with **SVG** and
  **PNG** options ([FEATURE-12](../../02-features/FEATURE-12-Metrics-Charts-Download.md)).
- Download links per artefact; ONNX emphasised (`btn-primary`). A **Download metrics.json** button
  appears for completed jobs ([FEATURE-12](../../02-features/FEATURE-12-Metrics-Charts-Download.md)).
  Links/buttons are only interactive once artefacts/metrics exist.
- Error alert on failed, using `error.message`.
- Polls every second and stops on a terminal status (`completed` / `failed`); the status region is
  `aria-live="polite"`.
- The in-flight poll is aborted when the view unmounts, so no request outlives the screen.
- The downloads card gains a **Download all** button that fetches `{jobId}.zip`
  ([FEATURE-15](../../02-features/FEATURE-15-Bulk-Artefact-Download.md)).
- The header shows the job's optional **name** and its **tags**; both are editable in place
  ([FEATURE-13](../../02-features/FEATURE-13-Job-Discovery.md)).

## Job comparison (`/jobs/compare?ids=…`)

([FEATURE-14](../../02-features/FEATURE-14-Job-Comparison.md))

- One column per job, one row per attribute: header (name, status, algorithm, task), dataset,
  feature columns, label column, split, seed, hyperparameters, metrics, artefact formats.
- The best value per metric is marked, using the documented direction rules; metrics with an
  unknown direction are never marked.
- A warning banner appears when the compared jobs use different task types, different datasets,
  or different feature columns, or when one of them is not completed.
- The selection is carried in the URL so the view can be bookmarked.

## Jobs (`/jobs`)

- Table: created, status badge, task, algorithm, tags, name, actions (open).
- Task labels for all five types; unsupervised rows show no label.
- A **filter bar** above the table: status, task type, algorithm, tag, created-between, and a
  free-text search box. Active filters are shown as removable chips with a **Clear all** action.
  The filter state is mirrored in the URL query string.
- The **Created**, **Status**, and **Algorithm** column headers are sortable buttons; the active
  sort is indicated with `aria-sort`.
- A **selection bar** appears at the bottom when rows are selected for comparison or bulk
  download, offering **Compare** and **Download all (N)**.
- Empty: "No jobs yet" + CTA. Empty *because of a filter*: "No jobs match these filters" +
  **Clear filters**.
- Loading: spinner. Error: alert with retry (network-error state) and the actionable guidance
  from [FEATURE-18](../../02-features/FEATURE-18-Client-Resilience-and-Guidance.md).

## Algorithms (`/algorithms`)

- Catalogue cards grouped by task: Classification, Regression, Semi-supervised, Clustering,
  Anomaly detection.
- Each card shows name, family, description, default hyperparameters, and export formats.
- Counts update automatically from `GET /api/v1/algorithms`; no hard-coded estimator list.

## Footer

- A link to the **API reference** (`NEXT_PUBLIC_API_DOCS_URL`, default
  `${NEXT_PUBLIC_API_URL}/docs`) opens the interactive OpenAPI documentation in a new tab
  ([06-api-specification.md §13](../../06-api-specification.md#13-interactive-api-reference)).
