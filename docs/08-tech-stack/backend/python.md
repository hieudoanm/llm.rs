# llm — Rust

Package: `crates/`. Layout:

```
crates/
  llm-core/   # config, CSV, catalogue, training, charts, storage, ONNX export
  llm-db/     # SQLite persistence
  llm-api/    # Axum HTTP API and background worker
  llm-server/ # Binary entry point
```

Rust edition 2021, `cargo fmt` + `cargo clippy`. Lock with Cargo.lock. No notebooks in the service image.

Lint/format rules: `cargo fmt --check`, `cargo clippy --all-targets`.
