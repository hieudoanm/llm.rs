---
title: Getting Started
description: Install llm.rs, build the CLI, and run the desktop app
order: 1
author: llm.rs
tags: [getting-started, install, cli, desktop]
---

# Getting started

llm.rs has two surfaces: a desktop application and a command-line interface. Both use
the same Rust training core and export the same ONNX artefact.

## Prerequisites

- [Rust](https://www.rust-lang.org/) with a stable toolchain.
- [Node.js 26+](https://nodejs.org/) and [pnpm](https://pnpm.io/) for the desktop app.

## Option 1 — CLI

Build the standalone binary:

```sh
cd crates
cargo build --release -p llm-cli
./target/release/llm --help
```

Register a dataset, train a classifier, and export the model:

```sh
# Register a CSV and note the dataset id
llm dataset data/demo.csv

# Train a logistic-regression classifier
llm train \
  --dataset-id <dataset-id> \
  --task classification \
  --algorithm logistic_regression \
  --features x1,x2 \
  --label target

# Inspect the job and copy the derived ONNX artefact
llm job <job-id>
llm export <job-id> ./model.onnx
```

`llm train` writes the ONNX artefact as part of the job. Use `llm algorithms` to list
the allow-listed algorithms and their default hyperparameters.

## Option 2 — Desktop app

The desktop app pairs a Next.js UI with a Tauri shell:

```sh
cd packages/desktop
pnpm install

# Run Next.js and Tauri together
pnpm dev

# Build the production app
pnpm build && pnpm build:tauri
```

Prebuilt bundles are published as release artifacts: macOS `.dmg` and `.app.zip`,
Linux `.rpm`, `.deb`, and `.AppImage`, and Windows `.msi` and `.exe`.

## Local data

By default the app persists state under `.llm-data/`. Set `LLM_DATA_DIR` to use a
different directory and `LLM_MAX_UPLOAD_MB` to change the CSV upload limit.

## Next steps

- [Training workflow](/docs/03-workflow/)
- [Algorithms](/docs/04-algorithms/)
- [CLI reference](/docs/02-cli/)
