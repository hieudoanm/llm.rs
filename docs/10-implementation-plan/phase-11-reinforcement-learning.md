# Phase 11: Reinforcement learning

Adds offline (batch) reinforcement learning as the sixth task type on top of the
Phase 9 nullable-label plumbing and the Phase 10 task-strategy work. No new API
endpoints and no new dependencies.

## Scope

1. **Experience mapping** — persist `jobs.rl_columns` (`actionColumn`,
   `rewardColumn`, optional `nextStateColumns`, `doneColumn`, `episodeColumn`)
   alongside `featureColumns`; add migration `0003` widening the `task_type`
   CHECK to include `reinforcement_learning` and adding the conditional
   `rl_columns` constraint.
2. **Catalogue** — add `fitted_q_iteration` under
   `taskType=reinforcement_learning` with `n_iterations`, `gamma`, `max_depth`,
   and `learning_rate` (`INVALID_HYPERPARAMETERS` outside 1..50, 0..1, 1..20,
   and (0, 1]).
3. **Validation** — reinforcement-learning jobs must omit `labelColumn` and
   supply `rlColumns` (`INVALID_COLUMN_MAP`); reward must be numeric
   (`INVALID_LABEL_TYPE`); require ≥ 10 transitions and ≥ 2 episodes when an
   episode column exists (`INVALID_HYPERPARAMETERS`).
4. **Training** — resolve transitions (explicit next state, or the next row
   within an episode), split by episode (or by transition when no episode
   column exists), and run FQI sweeps fitting the Q-function regressor on
   Bellman targets.
5. **Metrics and charts** — `meanReward`, `meanQ`, `tdErrorMean`,
   `tdErrorStd`, and `actions` (plus `episodeCount` and `meanEpisodeReturn`
   with an episode column); RL1–RL4 charts per
   [`charts.md` §3.7](../07-ui-ux-design/system/charts.md#37-reinforcement-learning-phase-11).
6. **Export** — convert the fitted preprocessor + Q-function regressor with
   `target_opset={"": ONNX_OPSET, "ai.onnx.ml": 3}`; the graph takes the state
   features followed by the one-hot action block; ONNX Q-values must match
   mlrs.
7. **UI** — add the Reinforcement learning task, the experience-column
   pickers, the FQI hyperparameter fields, metric explanations, job-history
   badges, and the sixth algorithms-page group; update the `02-new-job`,
   `04-jobs`, and `05-algorithms` prototypes and screenshots.
8. **Tests** — experience fixtures; API validation tests; FQI and metrics
   tests; ONNX parity for the Q-function; a Playwright path for the new task.

## Acceptance

- An experience CSV trains with `fitted_q_iteration` and produces ONNX whose
  Q-values match mlrs.
- `labelColumn` on a reinforcement-learning job returns
  `400 INVALID_COLUMN_MAP`, and so does a missing `rlColumns` (or one supplied
  to another task).
- Missing both `nextStateColumns` and `episodeColumn` returns
  `400 INVALID_COLUMN_MAP`.
- Out-of-range `n_iterations` / `gamma` / `max_depth` / `learning_rate`
  return `400 INVALID_HYPERPARAMETERS`.
- The job poll payload carries the documented RL metrics (including `actions`)
  and the RL1–RL4 chart data.
- Existing supervised, semi-supervised, and unsupervised flows remain green.
