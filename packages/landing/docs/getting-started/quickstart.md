# Quickstart

1. **Upload a CSV** — header row required; columns are inferred automatically.
2. **Map columns** — feature columns to **X**; for supervised and
   semi-supervised tasks, the target to **Y**. For offline reinforcement
   learning, declare state/action/reward/next-state or episode columns.
3. **Choose a task** — classification, regression, semi-supervised learning,
   clustering, anomaly detection, or reinforcement learning.
4. **Choose an algorithm** — pick from the curated catalogue for the task.
5. **Train** — jobs run asynchronously with visible status.
6. **Evaluate** — hold-out metrics for supervised/semi-supervised/offline RL;
   in-sample cluster or anomaly scores for unsupervised tasks.
7. **Export** — every job can download a portable **ONNX** artefact.

A local history of datasets, jobs, metrics, and artefact paths is kept
automatically.
