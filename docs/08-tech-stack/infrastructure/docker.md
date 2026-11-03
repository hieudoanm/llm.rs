# llm — Docker

## Backend

Multi-stage: the builder compiles the Rust workspace with `cargo build --release`, the runtime stage is `debian:bookworm-slim` running as a non-root user. Entry point: `./llm-server --host 0.0.0.0 --port 8000`. Data lives under `LLM_DATA_DIR=/var/lib/llm`.

Build context is the **repository root** so the root `.dockerignore` applies:

```sh
docker build -f crates/llm-server/Dockerfile -t llm-backend .
```

## Frontend

Multi-stage `node:26.12-alpine` build (matching `.node-version`), runtime `next start --hostname 0.0.0.0 --port 3000`. `NEXT_PUBLIC_API_URL` is inlined at build time, so it is passed as a build arg:

```sh
docker build -f packages/frontend/Dockerfile \
  --build-arg NEXT_PUBLIC_API_URL=http://127.0.0.1:8000 \
  -t llm-frontend .
```

## Compose

Defined in the root [`docker-compose.yaml`](../../../docker-compose.yaml). Services: `backend` (builds `crates/llm-server/Dockerfile`, volume `llm_data`, healthcheck `GET /healthz`), `frontend` (builds `packages/frontend/Dockerfile`, depends on healthy backend). Ports `127.0.0.1:8000`, `127.0.0.1:3000`.
