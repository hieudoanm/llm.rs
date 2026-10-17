# FEATURE-13: Job Discovery

## Priority

P1 (FR-120..FR-127)

## Description

A local user accumulates jobs. Without filters, the history table becomes a wall of rows that is
useless after the tenth run. This feature adds three orthogonal ways to find a job again —
**filter**, **sort**, and **search** — plus free-form **tags** and an optional human-readable
**name** so a job can be labelled at creation time and found later.

All of it is local, single-user, and implemented as query parameters on the existing
`GET /api/v1/jobs` endpoint plus one metadata endpoint. No new index or search service is
introduced.

## User-visible behaviour

### Filter

The jobs page has a filter bar above the table. Each control maps to one query parameter.

| Control            | Query parameter | Values                                                     | Behaviour                                                                    |
| ------------------ | --------------- | ---------------------------------------------------------- | ---------------------------------------------------------------------------- |
| Status             | `status`        | `queued`, `validating`, `training`, `evaluating`, `exporting`, `retrying`, `completed`, `failed` | Multi-select; an empty selection means "any".            |
| Task type          | `taskType`      | the six documented task types                              | Multi-select; an empty selection means "any".                               |
| Algorithm          | `algorithmId`   | catalogue ids                                               | Multi-select; an empty selection means "any".                               |
| Tag                | `tag`           | tag slugs                                                   | Multi-select; **all** selected tags must be present (AND semantics).        |
| Created between    | `createdFrom`, `createdTo` | ISO-8601 timestamps                                   | `createdFrom` inclusive, `createdTo` exclusive. Only one bound is required. |
| Text               | `q`             | free text                                                   | Matches job name, tag slugs, or algorithm id (case-insensitive substring). |

Filters compose with AND: `status=failed&taskType=classification&tag=baseline` returns failed
classification jobs tagged `baseline`.

An active filter set is shown as removable chips above the table, with a **Clear all** action.
The filter state is reflected in the URL query string so a filtered view can be bookmarked or
shared.

The URL is authoritative, but a bare `/jobs` visit restores the **last-used** filter set from
`localStorage` (key `mlrs-jobs-filters`); clearing every filter removes the stored entry. The restore
happens once, after mount, so a shared link always wins over the local memory.

The history table is **virtualised**: only the rows in view are mounted, so a history with hundreds
of jobs does not pay layout cost on every poll. Native table semantics (`<tbody>`, column
relationships) are preserved for assistive technology.

### Sort

The table header columns for **Created**, **Status**, and **Algorithm** are sortable; clicking a
header toggles ascending/descending. The chosen sort is the `sort` query parameter.

| `sort` value     | Meaning                                     | Default order |
| ---------------- | ------------------------------------------- | ------------- |
| `createdAt`      | Job creation time                           | `desc`        |
| `updatedAt`      | Last status transition                      | `desc`        |
| `status`        | Lifecycle order (not alphabetical)          | `asc`         |
| `algorithmId`    | Algorithm id, alphabetical                  | `asc`         |

The value is `field` or `field:asc` / `field:desc`. An unknown field returns
`400 VALIDATION_ERROR` with `details.field` set.

Sorting by `status` uses the **lifecycle order** (`queued` → `validating` → `training` →
`evaluating` → `exporting` → `retrying` → `completed` → `failed`), not alphabetical order.

Sorting is stable: rows with an equal sort key are ordered by `id DESC`.

### Search

The `q` parameter is a single free-text box in the filter bar. It matches, case-insensitively:

- the job **name**,
- any **tag** slug,
- the **algorithm id**.

It does **not** match column names, hyperparameters, or metrics — those are not useful search
keys for "which run was this?".

### Tags and name

A job carries:

- `name` — an optional human-readable label, max **120** characters, trimmed, stored as `null`
  when blank.
- `tags` — up to **10** tags, each 1–32 characters, lowercase-normalised, de-duplicated
  case-insensitively.

Both are editable from the job detail page and from the jobs table row menu via
`PATCH /api/v1/jobs/{jobId}`. Only metadata is editable — the training configuration of a job is
immutable once submitted.

The tag input offers the tags already used across the workspace as suggestions, plus free text.
Typing a new tag and pressing Enter creates it.

Tag rules:

- Tags are normalised to lowercase, trimmed, and stripped of surrounding whitespace. `Baseline`
  and `baseline` are the same tag.
- A tag may contain letters, digits, `-`, `_`, `.`, and spaces; other characters are rejected
  with `400 VALIDATION_ERROR` and `details.field = "tags"`.
- Duplicate tags in one request are collapsed, not an error.
- Adding more than 10 tags returns `400 VALIDATION_ERROR` and `details.limit = 10`.
- Removing a tag does not affect the job's training outcome.

## API

| Method | Path                       | Success | Purpose                                              |
| ------ | -------------------------- | ------- | ---------------------------------------------------- |
| GET    | `/api/v1/jobs`             | 200     | List with `status`, `taskType`, `algorithmId`, `tag`, `q`, `createdFrom`, `createdTo`, `sort` |
| PATCH  | `/api/v1/jobs/{jobId}`     | 200     | Update `name` and/or `tags`                         |

`GET /api/v1/jobs` accepts the existing `limit` and `cursor` parameters plus `status`,
`taskType`, `algorithmId`, `tag`, `q`, `createdFrom`, `createdTo`, and `sort`.

The cursor stays opaque: it encodes the sort key, the last row's id, and a fingerprint of the
active filter and sort set, so a cursor issued for one filter cannot silently paginate another.
A stale cursor is rejected with `400 VALIDATION_ERROR` (`details.field = "cursor"`) and the
client restarts from the first page.

`PATCH /api/v1/jobs/{jobId}` accepts `{ "name": "string | null", "tags": ["string"] }` and
returns the full job payload. Sending a field that is not `name` or `tags` returns
`400 VALIDATION_ERROR`; extra fields are rejected.

## Edge cases

| Case                                   | Behaviour                                                          |
| -------------------------------------- | ------------------------------------------------------------------ |
| Unknown `jobId` in `PATCH`             | `404 JOB_NOT_FOUND`                                                 |
| Filter matches nothing                 | `200` with an empty `jobs` array and the empty state                |
| `cursor` + filters combined            | The cursor encodes a fingerprint of the active filter and sort set. A cursor issued for a different set is rejected with `400 VALIDATION_ERROR` (`details.field = "cursor"`); the client restarts pagination from the first page |
| `status=failed&status=queued`          | `400 VALIDATION_ERROR` — repeated query parameters are rejected; use the comma-separated form `status=failed,queued` |
| `createdTo` before `createdFrom`       | `400 VALIDATION_ERROR` (`details.field = "createdTo"`)             |
| `createdFrom` in the future            | Accepted; matches nothing                                            |
| `sort=createdAt:sideways`              | `400 VALIDATION_ERROR` (`details.field = "sort"`)                    |
| `q` longer than 200 characters         | `400 VALIDATION_ERROR` (`details.field = "q"`)                       |
| `limit` outside 1..100                 | `400 VALIDATION_ERROR` (existing rule)                              |
| Job deleted while its row is displayed | `404 JOB_NOT_FOUND` on the next `PATCH`; the list refresh drops it  |

## Acceptance conditions

- `GET /api/v1/jobs?status=failed&taskType=classification` returns only failed classification
  jobs, newest first, with unchanged pagination semantics.
- `GET /api/v1/jobs?sort=algorithmId:asc` returns jobs ordered by algorithm id with the
  documented stable tie-break.
- `GET /api/v1/jobs?tag=baseline,release` returns only jobs carrying **both** tags.
- `GET /api/v1/jobs?q=logistic` returns jobs whose algorithm id contains `logistic`.
- `PATCH /api/v1/jobs/{jobId}` with `{ "tags": ["Baseline", "baseline", "release"] }` stores
  `["baseline", "release"]` and returns `200`.
- `PATCH /api/v1/jobs/{jobId}` with an empty `name` stores `null`.
- Filtering never mutates job data and never changes a job's status.
- The jobs page shows removable filter chips, sortable column headers, and a working empty state
  that distinguishes "no jobs yet" from "no jobs match these filters".

**See also:** [`FEATURE-06`](./FEATURE-06-Job-History.md),
[`06-api-specification.md`](../06-api-specification.md),
[`05-data-model.md`](../05-data-model.md).
