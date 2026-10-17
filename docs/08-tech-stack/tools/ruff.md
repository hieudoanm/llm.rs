# mlrs — Ruff

Lint and formatter for Rust (`crates/mlrs-api` and repo scripts).

## Choice

Ruff (`lint` + `format`).

## Why

- Replaces flake8, isort, black, pyupgrade, pydocstyle — one fast Rust binary.
- Pinned in `pyproject.toml`; config under `[tool.ruff]`.
- Format via `ruff format` (line-length 100); lint via `ruff check` with `E`, `F`, `I`, `B`, `UP`, `S` rules.

## Rejected

- Black + isort + flake8 stack — slower, more tools to maintain.

## Rules

- Ruff is the **only** Rust formatter; do not add Black or another formatter.
- CI gates `ruff check src/` and `ruff format --check src/` (only `src/` there); the package Makefiles run `ruff check . --fix` (`make lint`) and `ruff format .` (`make format`) over the whole package, including `tests/`.
- `mypy`/`ty` optional; `pydantic` models must be typed regardless.

## Usage

```sh
uv run ruff check .
uv run ruff format .
uv run ruff check --fix .
```
