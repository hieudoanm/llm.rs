# FEATURE-01: Dataset Upload

## Priority

P0 (FR-001..FR-009)

## Description

Users upload a CSV with a header row. The backend stores the bytes on the artefact volume, hashes the content, infers columns, and returns a `datasetId` used by later mapping and training steps.

## User-visible behaviour

- Drag-and-drop or file picker accepting `.csv`.
- Immediate client-side checks (extension, empty file, size).
- Loading state while the file is uploaded.
- Preview table of column names and inferred dtypes plus row count.
- Error alerts for validation, size, and network failures.

## Inputs

| Field | Type   | Location | Required | Constraints                           |
| ----- | ------ | -------- | -------- | ------------------------------------- |
| file  | binary | Form     | Yes      | `text/csv` or `.csv`, ≤ 10 MiB, UTF-8 |

## Outputs

| Field     | Type    | Description                                          |
| --------- | ------- | ---------------------------------------------------- |
| datasetId | UUID    | Generated identifier                                 |
| sha256    | string  | Hex digest of file bytes                             |
| filename  | string  | Original filename metadata                           |
| byteSize  | integer | Size in bytes                                        |
| columns   | array   | `{ name, inferredDtype, nullCount, sampleValues[] }` |
| rowCount  | integer | Count of data rows                                   |
| createdAt | string  | ISO 8601 UTC                                         |

## Behaviour

1. Reject missing file, empty body, or size over limit.
2. Check the declared content type and the extension, then **sniff the bytes**: a payload whose
   header matches a known binary format (ZIP, gzip, PNG, PDF, ELF) is rejected with
   `UNSUPPORTED_FILE_TYPE` even when it is named `.csv`.
3. Decode as UTF-8 CSV with `csv.Sniffer` / pandas; require ≥ 1 header and ≥ 1 data row. A byte
   stream that is not valid UTF-8, or that cannot be parsed as delimited text, is rejected with
   `MALFORMED_CSV`.
4. Require a non-empty header with no empty and no duplicate column names; otherwise
   `VALIDATION_ERROR` with `details.field = "columns"`.
5. Write to `{DATA_DIR}/datasets/{datasetId}.csv`.
6. Insert `datasets` row, and the `dataset_insights` profile row
   ([`FEATURE-16`](./FEATURE-16-Dataset-Insights.md)).
7. Return `201`.

## Edge cases

- Duplicate content (same SHA-256): still create a new `datasetId` (simpler local UX); hash is recorded for reproducibility.
- Binary or HTML disguised as CSV → `UNSUPPORTED_FILE_TYPE`.
- Non-UTF-8 bytes → `MALFORMED_CSV`.
- Header-only file → `VALIDATION_ERROR`.
- Duplicate or empty header names → `VALIDATION_ERROR`.

## Acceptance conditions

- Valid sample CSV yields `201` and a preview.
- Oversize file yields `413 UPLOAD_TOO_LARGE`.
- A ZIP payload renamed to `.csv` yields `400 UNSUPPORTED_FILE_TYPE`.
- Stored path is not derived from the user filename.
