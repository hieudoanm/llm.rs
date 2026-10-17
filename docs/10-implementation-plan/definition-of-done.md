# llm — Definition of Done

_Part of [10-implementation-plan.md](../10-implementation-plan.md)._

A phase is complete when all phase tasks are done, tests pass in CI, docs match behaviour, coverage thresholds hold for changed code, and deliverables are demonstrated on Compose.

MVP (end of Phase 8) additionally:

- [ ] `docker compose up --build` yields healthy frontend, backend, sqlite.
- [ ] Reviewer uploads sample CSV, maps X/Y, trains llm classifier, downloads ONNX.
- [ ] Regression sample path works.
- [ ] Invalid CSV surfaces a validation error in the UI.
- [ ] Failed export leaves `failed` without ONNX.
- [ ] `/healthz` and `/readyz` work as specified.

Semi-supervised extension (Phase 9) additionally:

- [ ] A partially labelled CSV trains and scores only held-out labelled rows.
- [ ] `labelColumn` is nullable in the schema with the task-type CHECK.
- [ ] `self_training_classifier` exports ONNX that matches the fitted wrapper.
- [ ] `INSUFFICIENT_LABELS` is returned before a job row is created.

Unsupervised extension (Phase 10) additionally:

- [ ] Clustering and anomaly jobs are accepted without a label column. Reinforcement-learning jobs are accepted without a label column but with `rlColumns`.
- [ ] Supplying a label to an unsupervised or reinforcement-learning task is rejected.
- [ ] KMeans/MiniBatchKMeans/GaussianMixture, IsolationForest, and `fitted_q_iteration` all export ONNX with `ai.onnx.ml` 3.
## Phase 11 reinforcement learning (additionally)

- [ ] An experience CSV trains with `fitted_q_iteration` and produces ONNX whose Q-values match llm.
- [ ] A label column on a reinforcement-learning job is rejected; missing or invalid `rlColumns` are rejected.
- [ ] FQI hyperparameters outside the documented ranges are rejected before a job row is created.
- [ ] Completed jobs return the RL metrics (including `actions`) and render the RL charts.
