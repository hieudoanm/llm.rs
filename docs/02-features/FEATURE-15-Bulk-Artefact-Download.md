# FEATURE-15: Bulk Artefact Download

## Priority

P1 (FR-140..FR-144)

## Description

A completed job can hold up to three artefacts (`.onnx` is always present; `.pkl` and `.joblib`
are optional) plus a `metrics.json` sidecar. Downloading them one click at a time is tedious,
and downloading several jobs' artefacts to share a result set is worse. This feature adds a
**ZIP bundle** endpoint: the backend streams a single archive containing every artefact (and the
metrics sidecar) for one job, or for several jobs at once.

The archive is built on the fly and streamed; nothing is written to the volume.

## User-visible behaviour

### Single job

On the job detail page, the downloads card gains a **Download all** button next to the
per-artefact links. It downloads `{jobId}.zip` containing:

```text
{jobId}/
├── {jobId}.onnx
├── {jobId}.pkl          # only when exportPkl was requested
├── {jobId}.joblib       # only when exportJoblib was requested
├── metrics.json         # only when the job is completed and has metrics
└── manifest.json        # always
```

`manifest.json` records what the bundle contains and how it was produced:

```json
{
  "jobId": "uuid",
  "name": "baseline",
  "tags": ["baseline"],
  "taskType": "classification",
  "algorithmId": "logistic_regression",
  "status": "completed",
  "createdAt": "2026-10-21T10:00:00Z",
  "libraryVersions": { "scikit-learn": "1.7.2" },
  "files": [
    { "path": "abc.onnx", "format": "onnx", "byteSize": 4096 },
    { "path": "metrics.json", "format": "metrics", "byteSize": 128 }
  ]
}
```

### Multiple jobs

On the jobs page, the selection bar (see [`FEATURE-13`](./FEATURE-13-Job-Discovery.md) and
[`FEATURE-14`](./FEATURE-14-Job-Comparison.md)) gains **Download all (N)** when at least one
selected job has artefacts. The archive is organised per job:

```text
mlrs-jobs.zip
├── <jobId-a>/
│   ├── <jobId-a>.onnx
│   ├── metrics.json
│   └── manifest.json
└── <jobId-b>/
    ├── <jobId-b>.onnx
    └── manifest.json
```

Jobs without artefacts are skipped and listed in a top-level `skipped.json`:

```json
{ "skipped": [{ "jobId": "uuid", "reason": "NO_ARTEFACTS" }] }
```

`reason` is `NO_ARTEFACTS` (job has no artefact rows) or `NOT_COMPLETED` (the job is still
running or failed).

## API

| Method | Path                          | Success | Purpose                                   |
| ------ | ----------------------------- | ------- | ----------------------------------------- |
| GET    | `/api/v1/jobs/{jobId}/artefacts.zip` | 200 | Bundle for one job                    |
| GET    | `/api/v1/jobs/artefacts.zip`  | 200     | Bundle for 1–10 jobs (`ids` query param) |

Both endpoints respond with:

- `Content-Type: application/zip`
- `Content-Disposition: attachment; filename="<name>.zip"`
- `Content-Length` when the archive is fully buffered (see the note below)

`GET /api/v1/jobs/artefacts.zip` takes `ids` — comma-separated UUIDs, **1 to 10** entries.

The archive is written to a temporary file and then streamed, so `Content-Length` is known and
the response is not chunked. The temporary file is deleted after the response is sent.

## Edge cases

| Case                                    | Behaviour                                                          |
| --------------------------------------- | ------------------------------------------------------------------ |
| Job does not exist                      | `404 JOB_NOT_FOUND`                                                 |
| Job exists but has no artefact rows     | `404 ARTEFACT_NOT_FOUND` (`details.jobId`)                         |
| Job is still running                    | `404 ARTEFACT_NOT_FOUND` (`details.jobId`) — artefacts do not exist yet |
| Multi-job bundle where **some** jobs have no artefacts | Those jobs are skipped and listed in `skipped.json`; the request still returns `200` |
| Multi-job bundle where **no** job has artefacts | `404 ARTEFACT_NOT_FOUND` with `details.jobIds` listing them |
| More than 10 ids                        | `400 VALIDATION_ERROR` (`details.field = "ids"`, `details.max = 10`) |
| Malformed UUID in `ids`                 | `400 VALIDATION_ERROR` (`details.field = "ids"`)                     |
| An artefact file is missing on disk     | The bundle is still produced; the missing file is omitted from `manifest.json` and the response header `X-mlrs-Missing-Files` lists the paths |
| `metrics.json` is absent (job not completed) | The sidecar is omitted; the bundle still contains the artefacts |

## Acceptance conditions

- `GET /api/v1/jobs/{jobId}/artefacts.zip` for a completed job with ONNX only returns `200`,
  `application/zip`, and an archive containing exactly `<jobId>.onnx`, `metrics.json`, and
  `manifest.json`.
- The same request for a job created with `exportPkl` and `exportJoblib` returns an archive
  containing the ONNX, PKL, and joblib files plus the two JSON sidecars.
- The extracted ONNX file byte-matches the artefact served by
  `GET /api/v1/jobs/{jobId}/artefacts/{artefactId}`.
- `GET /api/v1/jobs/artefacts.zip?ids=<a>,<b>` returns one directory per job with artefacts, and
  `skipped.json` naming the jobs that were omitted.
- A bundle request never writes to the artefact volume.
- The download buttons are disabled, with an explanatory tooltip, while the bundle is being
  generated, and the browser receives exactly one file.

**See also:** [`FEATURE-05`](./FEATURE-05-Model-Export.md),
[`FEATURE-12`](./FEATURE-12-Metrics-Charts-Download.md),
[`06-api-specification.md`](../06-api-specification.md).
