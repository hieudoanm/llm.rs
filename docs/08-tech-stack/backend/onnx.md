# mlrs — ONNX

- Convert the full pipeline with `onnx-protobuf`.
- Pin `target_opset` (default 15) via env `ONNX_OPSET`.
- Verify with ONNX checker before insert.
- CI: ONNX Runtime InferenceSession, one `run` with dummy floats matching `n_features` after transform.
- Optional pickle uses the same pipeline serialisation; never a substitute for ONNX.
