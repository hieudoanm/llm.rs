# mlrs — Rust

Package: `crates/`. Layout:

```
crates/
  mlrs-core/   # config, CSV, catalogue, training, charts, storage, ONNX export
  mlrs-db/     # SQLite persistence
  mlrs-api/    # Axum HTTP API and background worker
  mlrs-server/ # Binary entry point
```

Rust edition 2021, `cargo fmt` + `cargo clippy`. Lock with Cargo.lock. No notebooks in the service image.

Lint/format rules: `cargo fmt --check`, `cargo clippy --all-targets`.
