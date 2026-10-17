# Phase 9: Semi-supervised learning

Week 7. Add self-training on partially labelled CSVs without changing the
existing supervised contract.

## Scope

1. **Data model** — make `jobs.label_column` nullable; add the five-value
   `task_type` CHECK and the `jobs_label_column_matches_task` CHECK. Because
   `init_db()` does not alter existing tables, add the first Alembic migration
   (revision 0001) that makes `label_column` nullable and adds both CHECKs, and
   update `schema.sql`. The migration must upgrade an existing local volume in
   place; a volume reset is only a fallback for disposable data.
2. **Catalogue** — add `self_training_classifier` to `ALGORITHMS` with its
   default hyperparameters and `exports`; extend the catalogue integrity test.
3. **Validation** — parse the label column at create time; treat empty,
   whitespace-only, `NaN`, and `null` cells as unlabelled; enforce FR-082
   (two classes, 10 labelled rows, 2 per class) and return
   `400 INSUFFICIENT_LABELS` before inserting a job.
4. **Training** — split labelled rows, append unlabelled rows with `-1`, fit the
   preprocessor on all training rows, fit `SelfTrainingClassifier`, evaluate on
   the labelled hold-out, and record `labelledRows`, `unlabelledRows`,
   `iterations`.
5. **Export** — ONNX = fitted preprocessor + `SelfTrainingClassifier.estimator_`;
   `pkl`/`joblib` = fitted wrapper. Verify predictions match.
6. **API** — `taskType` enum, nullable `labelColumn`, new error code, metrics
   documentation, OpenAPI updates.
7. **UI** — task select, conditional label control, blank-label helper,
   labelled/unlabelled counts, threshold/max-iteration fields, metrics card, job
   history badge, algorithms page group.
8. **Prototypes and screenshots** — update the `02-new-job` task select and
   label helper, the `01-home` hero sub-line, and `04-jobs` badges, then
   regenerate light/dark images.
9. **Tests** — semi-supervised fixture and insufficient-label fixture; API test
   for 400; worker test for labelled-only scoring; ONNX parity test; Playwright
   happy path.

## Acceptance

- A compliant partially labelled CSV completes with classification metrics and
  ONNX.
- Unlabelled rows never appear in evaluation.
- An invalid label set returns `400 INSUFFICIENT_LABELS` and creates no job.
- Existing classification/regression tests and jobs are unchanged.
- ONNX predictions equal the fitted wrapper on the full feature matrix.
