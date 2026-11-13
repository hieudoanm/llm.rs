# Tasks and algorithms

llm.rs organises training around **tasks**; each task has a curated catalogue of
algorithms.

| Task                   | Label column                | Evaluation                     |
| ---------------------- | --------------------------- | ------------------------------ |
| Classification         | `Y` required                | Hold-out metrics               |
| Regression             | `Y` required                | Hold-out metrics               |
| Semi-supervised        | `Y` partially labelled      | Hold-out metrics               |
| Clustering             | none                        | In-sample cluster scores       |
| Anomaly detection      | none                        | In-sample anomaly scores       |
| Reinforcement learning | state/action/reward columns | Episode or transition hold-out |

Column mapping is explicit: features → `X`, target → `Y` (when the task requires
it), or the RL experience columns for offline RL.
