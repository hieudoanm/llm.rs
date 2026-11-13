# Jobs, metrics, and ONNX export

## Training jobs

Training runs asynchronously — the UI and API expose job status while the
backend works. Jobs are recorded in the local history together with their
datasets, metrics, and artefact paths.

## Metrics

- **Supervised, semi-supervised, offline RL** — hold-out evaluation (episodes or
  transitions for RL).
- **Unsupervised (clustering, anomaly detection)** — in-sample scores.

## ONNX export

Every trained model can be exported as a portable **ONNX** artefact, so models
trained in llm.rs can be loaded by any ONNX runtime — no llm.rs dependency at
inference time.
