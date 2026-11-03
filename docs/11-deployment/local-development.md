# Local Development

## Prerequisites

- Docker and Docker Compose v2
- Node.js 26+ (frontend host tooling)
- Rust toolchain (backend host tooling)
- pnpm, cargo
- Git

## Repository structure

```text
llm/
├── crates/
│   ├── llm-core/   # Pure-Rust core
│   ├── llm-db/     # SQLite persistence
│   ├── llm-api/    # Axum HTTP API and worker
│   └── llm-server/ # Binary entry point
├── packages/
│   └── frontend/    # Next.js
├── docker-compose.yaml
├── docs/
└── README.md
```

## Docker Compose services

Defined in [`docker-compose.yaml`](../../docker-compose.yaml). Both images build
from the **repository root** context.

### backend

- Build: `crates/llm-server/Dockerfile`
- Port: `127.0.0.1:8000`
- Volume: `llm_data:/var/lib/llm`
- Env: `DATABASE_URL`, `LLM_DATA_DIR`, `LLM_MAX_UPLOAD_MB` (upload cap in MiB, default 10), plus
  `LLM_LOG_LEVEL`, `LLM_CORS_ORIGINS`, `LLM_CORS_ALLOW_CREDENTIALS`, `LLM_ENABLE_INTERACTIVE_DOCS`,
  `LLM_MAX_CONCURRENT_JOBS`, `LLM_JOB_TIMEOUT_SECONDS`, `LLM_JOB_MAX_ATTEMPTS`,
  `LLM_JOB_RETRY_BASE_SECONDS`, `LLM_WORKER_STALE_SECONDS` (see below)
- Healthcheck: `GET /healthz`

Every variable the backend reads is listed in
[`environment-variables.md`](./environment-variables.md), which is the single source of truth.

### frontend

- Build: `packages/frontend/Dockerfile`
- Port: `127.0.0.1:3000`
- Env: `NEXT_PUBLIC_API_URL=http://127.0.0.1:8000` (baked in at build time),
  `NEXT_PUBLIC_API_DOCS_URL` (defaults to `${NEXT_PUBLIC_API_URL}/docs` when empty)
- Depends on: backend healthy

## Workflow

```bash
cp .env.example .env
docker compose up --build   # or: make up
```

Open `http://127.0.0.1:3000`. API docs: `http://127.0.0.1:8000/docs` (Swagger UI) and
`http://127.0.0.1:8000/redoc` (ReDoc); the raw schema is at
`http://127.0.0.1:8000/openapi.json`. The committed copy of the schema lives in
[`06-api-specification/backend/openapi.yaml`](../06-api-specification/backend/openapi.yaml).

## CLI

The `llm` CLI is a standalone offline tool (no backend required). Install the
latest release binary:

```bash
curl -fsSL https://github.com/hieudoanm/llm-client/releases/download/latest/llm \
  -o "$HOME/.local/bin/llm"
chmod +x "$HOME/.local/bin/llm"
```

Full CLI documentation lives in
[`11-deployment/cli.md`](./cli.md); the CLI source is in `crates/llm-server/`.

Make shortcuts (from the repository root): `make up` brings up the whole
stack with Docker Compose and `make down` shuts it down. Host mode (optional):
`make start` runs the API and frontend on the host — the Makefile passes all `LLM_*`
variables from the environment (`.env`).

## Tests

```bash
make e2e        # Playwright scenarios (backend + frontend must already be up)
make e2e-demo   # same scenarios recorded to demo/demo.mp4 (starts/stops services)
```

## Data

Volumes persist jobs across restarts. Reset:

```bash
# Reset the entire stack including SQLite data and artefacts
docker compose down -v && docker compose up --build -d
```

Or use the Makefile:

```bash
make down  # stops and removes containers; add -v to wipe volumes
```

`docker compose down -v` deletes artefacts and the SQLite database.
