# FEATURE-06: Job History

## Priority

P0 list/get (FR-060..FR-062); P1 delete (FR-063)

## Description

Users review previous local jobs, reopen a job's metrics and downloads, and (P1) delete a job including files.

## User-visible behaviour

- Jobs table: status badge, algorithm, task, timestamps.
- Task labels for all five task types; clustering and anomaly-detection rows show no label column.
- Click row → job status page.
- Empty state with CTA to create a job.
- Optional delete with confirm dialog.

## Acceptance conditions

- Newest first, stable pagination.
- Delete removes DB rows and files; subsequent GET is `404`.

## Discovery additions (Phase 12)

The history list also supports filtering, sorting, free-text search, and per-job tags. These are
normative in [`FEATURE-13`](./FEATURE-13-Job-Discovery.md); the side-by-side view is
[`FEATURE-14`](./FEATURE-14-Job-Comparison.md), and the multi-job artefact bundle is
[`FEATURE-15`](./FEATURE-15-Bulk-Artefact-Download.md).

- `GET /api/v1/jobs` accepts `status`, `taskType`, `algorithmId`, `tag`, `q`, `createdFrom`,
  `createdTo`, and `sort` in addition to `limit` and `cursor`.
- `PATCH /api/v1/jobs/{jobId}` updates the job's optional `name` and `tags`. The training
  configuration of a job stays immutable.
