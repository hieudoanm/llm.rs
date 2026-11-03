# FEATURE-11: Reinforcement Learning

## Priority

P0 (FR-110..FR-115). Planned in Phase 11.

## Description

Offline (batch) reinforcement learning turns a logged experience CSV into a fitted Q-function:

- The dataset is a **transition log**: one row per observed transition with state features (**X**), an action, a reward, and either explicit next-state columns or an episode column that orders transitions.
- The single P0 algorithm, **`fitted_q_iteration`**, runs Fitted Q-Iteration (FQI): each sweep fits a llm regressor that maps `(state, action)` → Q-value against Bellman targets built from the previous sweep's Q-function.
- The fitted Q-function (preprocessor + regressor) is exported to ONNX like every other task. A consumer derives the policy by evaluating Q for each recorded action and taking the argmax.

No label column is used, and no new dependency is introduced: FQI is fitted on the llm estimators already in the stack.

## User-visible behaviour

- Task select adds **Reinforcement learning**, grouped under a new **Offline reinforcement learning** section alongside the existing supervision groups.
- The label select is hidden for the task, with the helper "No label column is used."
- The mapping step gains experience-column pickers: **Action**, **Reward**, **Next state** (one column per state feature, in state order), **Done**, and **Episode** (the last two optional).
- The model step shows FQI fields: iterations (1–50, default 10), discount γ (0–1, default 0.9), tree depth (1–20, default 3), learning rate (0 < value ≤ 1, default 0.1).
- The job status screen renders the reinforcement-learning metrics with direction hints and, on `completed`, the RL charts (see [`charts.md`](../07-ui-ux-design/system/charts.md#37-reinforcement-learning-phase-11)).
- Job history shows the task badge and no label column.
- The `/algorithms` page gains a **Reinforcement learning** group containing `fitted_q_iteration`.

## Inputs

| Field           | Type     | Required | Constraints                                           |
| --------------- | -------- | -------- | ----------------------------------------------------- |
| datasetId       | UUID     | Yes      | Must exist                                            |
| featureColumns  | string[] | Yes      | State features; subset of header, ≥ 1                 |
| labelColumn     | —        | No       | Must be omitted or `null`                             |
| taskType        | enum     | Yes      | `reinforcement_learning`                              |
| algorithmId     | string   | Yes      | Catalogue allow-list for the task                     |
| rlColumns       | object   | Yes      | Experience columns, see below                         |
| hyperparameters | object   | No       | `n_iterations`, `gamma`, `max_depth`, `learning_rate` |
| splitRatio      | number   | No       | 0 < value < 1, default 0.2                            |
| randomSeed      | integer  | No       | Default 42                                            |

### `rlColumns`

| Field            | Type     | Required    | Constraints                                                                           |
| ---------------- | -------- | ----------- | ------------------------------------------------------------------------------------- |
| actionColumn     | string   | Yes         | Discrete; 2–50 distinct values; not a state feature                                   |
| rewardColumn     | string   | Yes         | Numeric; not a state feature                                                          |
| nextStateColumns | string[] | Conditional | Same length and order as `featureColumns`; required unless `episodeColumn` is present |
| doneColumn       | string   | No          | Booleans or `0`/`1`; not a state feature                                              |
| episodeColumn    | string   | No          | Episode id; orders transitions and drives the episode split                           |

### Algorithm

| algorithmId          | Estimator                              | Hyperparameters                                                                      | Default         |
| -------------------- | -------------------------------------- | ------------------------------------------------------------------------------------ | --------------- |
| `fitted_q_iteration` | `GradientBoostingRegressor` Q-function | `n_iterations` (1..50), `gamma` (0..1), `max_depth` (1..20), `learning_rate` ((0,1]) | 10, 0.9, 3, 0.1 |

`random_state` is fixed to 42 (or the job seed) so the split, the sweeps, and the export are reproducible.

## Behaviour

1. Validate that `labelColumn` is absent and that `rlColumns` is present; validate every experience column against the header, against overlap with `featureColumns`, and against its type rules (discrete action set of 2–50 values, numeric reward, boolean/`0`/`1` done flag, `nextStateColumns` mirroring `featureColumns`). Require `nextStateColumns` or `episodeColumn`, at least 10 transitions, and at least two episodes when `episodeColumn` is present.
2. Resolve each row into a transition `(s, a, r, s', done, episode)`:
   - the next state comes from `nextStateColumns` when present, otherwise from the next row of the same episode (CSV order); the last row of an episode is terminal;
   - `done` is true when the `doneColumn` is truthy or the transition ends its episode.
3. Split: whole episodes go to the hold-out side (default 0.2 of episodes, seed 42) when `episodeColumn` is present; otherwise a random hold-out of transitions. The preprocessor is fitted on the training side only.
4. Run FQI: initialise `Q₀(s, a) = 0`; for each of `n_iterations` sweeps fit the Q-function regressor on the training transitions with Bellman targets `y = r + gamma × (1 − done) × maxₐ′ Qₖ₋₁(s′, a′)`. The state block goes through the shared preprocessor, the action is one-hot encoded, and the two are concatenated before the regressor.
5. Evaluate on the hold-out side with the TD error `δ = y − Qₖ(s, a)`; record the documented metrics and chart data.
6. Export the fitted preprocessor + Q-function regressor to ONNX with `ai.onnx.ml` set to 3, record the action set in `metrics.actions`, and complete the job.

## Export contract

- ONNX inputs: the state features in `featureColumns` order, followed by a one-hot action block of length `len(actions)` in `actions` order.
- ONNX output: one scalar — the Q-value for the supplied `(state, action)` pair.
- Policy: `π(s) = argmaxᵢ Q(s, onehot(actionᵢ))`. Evaluating a state costs `len(actions)` inference calls; the argmax is a documented consumer-side rule and is not part of the graph.
- `pkl` / `joblib` contain the same fitted pipeline.

## Metrics

| Metric              | Task                                         | Meaning                                                                                                              | Better |
| ------------------- | -------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- | ------ |
| `meanReward`        | reinforcement_learning                       | Mean immediate reward across evaluation transitions; a data baseline, not a model score.                             | —      |
| `meanQ`             | reinforcement_learning                       | Mean fitted Q(s, a) over evaluation transitions.                                                                     | —      |
| `tdErrorMean`       | reinforcement_learning                       | Mean absolute Bellman residual on evaluation transitions.                                                            | Lower  |
| `tdErrorStd`        | reinforcement_learning                       | Standard deviation of signed TD errors on evaluation transitions.                                                    | Lower  |
| `episodeCount`      | reinforcement_learning (with episode column) | Number of episodes in the log (evaluation side of the split).                                                        | —      |
| `meanEpisodeReturn` | reinforcement_learning (with episode column) | Mean undiscounted reward sum per episode in the log; a reference for the fitted values.                              | —      |
| `actions`           | reinforcement_learning                       | Sorted distinct action values (numeric order for numeric columns, lexicographic otherwise); the one-hot input order. | —      |

## Edge cases

- A supplied label column → `400 INVALID_COLUMN_MAP`.
- `rlColumns` missing for a reinforcement-learning job, or supplied for any other task → `400 INVALID_COLUMN_MAP`.
- Unknown experience column, or an experience column also selected as a state feature → `400 INVALID_COLUMN_MAP`.
- Neither `nextStateColumns` nor `episodeColumn` → `400 INVALID_COLUMN_MAP` because `s′` cannot be resolved.
- Fewer than 2 or more than 50 distinct action values → `400 INVALID_COLUMN_MAP`.
- Non-numeric reward or non-boolean/`0`/`1` done flag → `400 INVALID_LABEL_TYPE` / `400 INVALID_COLUMN_MAP`.
- Fewer than 10 transitions, or fewer than two episodes when `episodeColumn` is present → `400 INVALID_HYPERPARAMETERS`.
- `n_iterations`, `gamma`, `max_depth`, or `learning_rate` outside its documented range → `400 INVALID_HYPERPARAMETERS`.
- Last row of an episode → terminal transition; the Bellman target reduces to `r` and never crosses an episode boundary.
- Degenerate log (identical rewards, single action after validation) → metrics still return; charts hide empty series per [`charts.md` §7](../07-ui-ux-design/system/charts.md#7-empty-and-degenerate-states).
- ONNX export must set `ai.onnx.ml` to 3; the default v4 operators are not supported by the pinned ONNX Runtime.

## Acceptance conditions

- An experience CSV completes for `fitted_q_iteration` with the documented metrics and an ONNX artefact.
- Reinforcement-learning jobs expose `labelColumn = null` and persist `rlColumns` on the job.
- ONNX Q-values match the fitted llm regressor on evaluation inputs (state block + one-hot action).
- `metrics.actions` lists the action set in one-hot order, and the documented argmax rule reproduces the greedy policy.
- Validation errors (label supplied, missing experience columns, unresolvable next state, out-of-range hyperparameters) return the documented 400 codes with no job row.
- The UI hides the label control, exposes the experience-column pickers, validates ranges before submission, and renders the RL metrics and charts.
