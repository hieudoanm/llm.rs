# llm — Test Fixtures

Store under `crates/llm-api/tests/fixtures/`:

| File                               | Task                        | Notes                                                             |
| ---------------------------------- | --------------------------- | ----------------------------------------------------------------- |
| `binary_class.csv`                 | classification              | 2 features, 2 labels, ≥ 40 rows                                   |
| `regression.csv`                   | regression                  | numeric Y                                                         |
| `semi_supervised.csv`              | semi_supervised             | 2 classes, ≥ 40 labelled rows, ≥ 10 blank label cells, 2 features |
| `semi_supervised_insufficient.csv` | invalid for semi_supervised | fewer than 10 labelled rows                                       |
| `clustering.csv`                   | clustering                  | 3 well-separated blobs, no label, ≥ 40 rows                       |
| `anomaly.csv`                      | anomaly_detection           | 2 features, a small injected outlier group, no label              |
| `experience.csv`                   | reinforcement_learning      | 2–3 state features, action, reward, next-state or episode, optional done |
| `empty.csv`                        | invalid                     | empty                                                             |
| `header_only.csv`                  | invalid                     | header, no rows                                                   |
| `non_numeric_y.csv`                | invalid for regression      | strings in Y                                                      |

Deterministic row order. UTF-8. No PII. Unlabelled rows use an empty label
cell (not a sentinel token) so the test exercises the real CSV parsing path.
