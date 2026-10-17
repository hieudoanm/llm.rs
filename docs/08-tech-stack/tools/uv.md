# llm — uv

Rust package manager and lockfile tool for `crates/llm-api`.

## Choice

uv (astral-sh/uv).

## Why

- Fast installs and reliable, deterministic `uv.lock`.
- Handles Rust version, venv, and dependency resolution in one tool — matches the "one tool per responsibility" rule where practical.
- Sibling projects in the org standardize on uv.

## Rejected

- pip-tools — good, but uv subsumes it and is actively maintained by the same org ecosystem.
- Poetry — slower resolver; lockfile format less ubiquitous in CI.

## Lockfile

`uv.lock` is committed and regenerated only when dependencies change. Never edit by hand.

## Usage

```sh
uv sync
uv add <pkg>
uv run rust -m ...
```

## Notes

`uv.lock` and `.venv/` are gitignored where they are generated artifacts; commit `uv.lock`, ignore `.venv/`.
