# llm.rs documentation

**llm.rs** (Local Learning Machine) is a local, full-stack training studio for
tabular data — and a Rust ML workspace. Upload a CSV, map columns to **X** and
**Y**, choose a task and algorithm, train asynchronously, and download a
portable **ONNX** model. Everything runs on your own machine.

## Sections

- **Getting started** — run the stack with Docker Compose or build the CLI.
- **Concepts** — tasks, algorithms, jobs, and the ONNX export.
- **API reference** — the backend HTTP API.
- **Guides** — the desktop app and common workflows.

## Interfaces

| Interface   | What it is                                                  |
| ----------- | ----------------------------------------------------------- |
| Web app     | Next.js frontend + Rust backend + SQLite via Docker Compose |
| Desktop app | Next.js + Tauri desktop UI (`packages/desktop`)             |
| CLI         | `llm` binary (`crates/llm-cli`)                             |
