# API reference

The backend is a Rust HTTP service backed by SQLite (`.llm-data/llm.db` by
default). Datasets, training jobs, metrics, and ONNX artefacts are exposed
through REST endpoints consumed by the web and desktop frontends.

## Configuration

`.env` (copy from `.env.example`) parameterises the app:

| Variable            | Purpose                               |
| ------------------- | ------------------------------------- |
| `LLM_DATA_DIR`      | Data directory (default `.llm-data/`) |
| `LLM_MAX_UPLOAD_MB` | Maximum CSV upload size               |
| `LLM_LOG_LEVEL`     | Log verbosity                         |

The full endpoint specification lives in the repository's
[`docs/06-api-specification`](https://github.com/hieudoanm/llm.rs/tree/master/docs/06-api-specification)
(OpenAPI YAML under `backend/`).
