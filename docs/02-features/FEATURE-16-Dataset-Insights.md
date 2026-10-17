# FEATURE-16: Dataset Insights

## Priority

P1 (FR-150..FR-157)

## Description

The current dataset response returns only column names and inferred dtypes. The user is asked to
map columns to **X** and **Y** without being told anything about the data: how many values are
missing, whether a column is constant, whether a column looks like an identifier, whether a
numeric column has an extreme outlier range. The result is a failed job or a bad model.

This feature adds a **dataset insights** view: a per-column profile and a dataset-level
readiness report computed once at upload time and re-readable afterwards.

The profile is a *diagnostic aid*, not a gate. Nothing in this feature blocks training; it tells
the user what to fix.

## User-visible behaviour

### The insights panel

After a CSV is uploaded — and reachable later from the job detail page — the dataset card shows
a **Dataset insights** panel:

**Dataset summary**

- Row count, column count, total missing cells, and the overall missing rate.
- A list of warnings (see below), each with a plain-language explanation and a suggested action.

**Column table**

| Column             | Type        | Missing            | Unique | Notes                                     |
| ------------------ | ----------- | ------------------ | ------ | ----------------------------------------- |
| `age`              | numeric     | 3 (0.5%)           | 87     | range 18–79, mean 41.2, std 12.9          |
| `city`             | categorical | 0                  | 12     | top: London (31), Paris (28), Berlin (17) |
| `id`               | numeric     | 0                  | 150    | unique per row — possible identifier       |
| `target`           | categorical | 0                  | 2      | balanced: setosa 50, versicolor 50        |
| `notes`            | text        | 12 (2.0%)          | 138    | free text — usually not a usable feature   |

Inferred types:

| Type        | Rule                                                                   |
| ----------- | ---------------------------------------------------------------------- |
| `numeric`   | Every non-empty value parses as a number.                             |
| `boolean`   | Every non-empty value is in {true, false, yes, no, 0, 1}.              |
| `categorical`| Every non-empty value is a string, and cardinality ≤ 50 **or** the column is a candidate label. |
| `text`      | Every non-empty value is a string and cardinality > 50.                |
| `datetime`  | Every non-empty value parses as a date/timestamp.                      |

A column that mixes numbers and non-numbers is `categorical` and raises the `MIXED_TYPES`
warning.

**Preview rows**

The first **10** rows are shown in a horizontally scrollable table with the raw cell values.
Values are rendered as text; missing cells render as `—` and are visually marked.

### Warnings

| Code                 | Trigger                                                          | Suggested action shown to the user                       |
| -------------------- | ---------------------------------------------------------------- | ------------------------------------------------------- |
| `CONSTANT_COLUMN`    | The column has exactly one distinct value.                        | "Remove this column from the feature set."              |
| `HIGH_MISSING_RATE`  | Missing rate ≥ 30%.                                               | "Impute or drop this column."                           |
| `ALL_VALUES_MISSING` | Every value is missing.                                           | "This column carries no information."                   |
| `HIGH_CARDINALITY`   | Cardinality > 50% of the row count and > 50 distinct values.      | "Looks like an identifier — keep it out of the features."|
| `POSSIBLE_ID_COLUMN` | Distinct count equals the row count.                              | "Every row has a unique value — probably an ID."        |
| `MIXED_TYPES`        | Numbers and non-numbers in the same column.                       | "Convert this column to a single type before using it."  |
| `TEXT_COLUMN`        | Inferred `text` type.                                             | "Not usable as a numeric feature."                       |
| `SINGLE_VALUE_LABEL` | The dataset has one possible label value in every candidate column | "Add more labelled rows."                              |

Warnings are advisory. The upload still succeeds and the job still trains.

### Re-reading insights

Insights are computed once at upload and stored with the dataset. The panel can be reopened
later without re-parsing the CSV. Insights are therefore stable for a given `sha256`.

## API

| Method | Path                                  | Success | Purpose                                       |
| ------ | ------------------------------------- | ------- | --------------------------------------------- |
| GET    | `/api/v1/datasets/{datasetId}/insights` | 200   | Column profiles, warnings, and preview rows  |

Query parameters:

| Parameter  | Default | Range    | Meaning                                   |
| ---------- | ------- | -------- | ----------------------------------------- |
| `previewLimit`  | `10` | 1–100 | Number of preview rows returned       |
| `previewOffset` | `0` | ≥ 0     | Number of preview rows skipped        |

Response:

```json
{
  "datasetId": "uuid",
  "sha256": "hex",
  "filename": "iris.csv",
  "rowCount": 150,
  "columnCount": 5,
  "totalMissingCells": 3,
  "missingRate": 0.004,
  "columns": [
    {
      "name": "age",
      "inferredType": "numeric",
      "missingCount": 3,
      "missingRate": 0.02,
      "uniqueCount": 87,
      "numeric": { "min": 18.0, "max": 79.0, "mean": 41.2, "std": 12.9, "median": 40.5 },
      "topValues": [],
      "warnings": []
    },
    {
      "name": "city",
      "inferredType": "categorical",
      "missingCount": 0,
      "missingRate": 0.0,
      "uniqueCount": 12,
      "numeric": null,
      "topValues": [{ "value": "London", "count": 31 }],
      "warnings": []
    }
  ],
  "warnings": [{ "code": "POSSIBLE_ID_COLUMN", "column": "id", "message": "…" }],
  "preview": {
    "offset": 0,
    "limit": 10,
    "totalRows": 150,
    "columns": ["age", "city", "target"],
    "rows": [["38", "London", "setosa"]]
  }
}
```

Preview rows are arrays of strings, positionally aligned with `preview.columns`, so a cell that
held a number in the CSV is returned as its source text. `null` marks a missing cell.

## Edge cases

| Case                                       | Behaviour                                                        |
| ------------------------------------------ | ----------------------------------------------------------------- |
| Unknown `datasetId`                        | `404 DATASET_NOT_FOUND`                                            |
| Dataset with a single row                  | Insights are returned; `numeric.std` is `null` (undefined for n=1) |
| Dataset with a single column               | Insights are returned; no warning about feature count            |
| Every column is missing                    | Insights are returned; every column carries `ALL_VALUES_MISSING`  |
| A column with 10 000 distinct string values | Inferred `text`; `HIGH_CARDINALITY` warning                       |
| `previewOffset` beyond the row count       | `preview.rows` is empty; `totalRows` still reports the real count |
| `previewLimit` outside 1–100               | `400 VALIDATION_ERROR` (`details.field = "previewLimit"`)         |
| CSV re-uploaded with identical bytes        | Same `sha256`; insights are identical                              |
| CSV re-uploaded with different bytes        | New `datasetId`; new insights                                       |

## Acceptance conditions

- `GET /api/v1/datasets/{datasetId}/insights` returns `200` with a profile for **every** column in
  the header, including columns with missing values.
- A constant column raises `CONSTANT_COLUMN` and appears in `warnings`.
- A column whose distinct count equals the row count raises `POSSIBLE_ID_COLUMN`.
- The response contains at most `previewLimit` preview rows starting at `previewOffset`.
- The insights panel renders the column table, the warning list with suggested actions, and the
  preview table; the panel is keyboard navigable and the warning list is announced via
  `aria-live="polite"`.
- No warning blocks the user from starting a training job.

**See also:** [`FEATURE-01`](./FEATURE-01-Dataset-Upload.md),
[`FEATURE-02`](./FEATURE-02-Column-Mapping.md),
[`06-api-specification.md`](../06-api-specification.md).
