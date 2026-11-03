.PHONY: help cli cli-build cli-run desktop desktop-dev desktop-build test rust-check rust-test

CRATES_DIR := crates
DESKTOP_DIR := packages/desktop
CLI_CRATE := llm-cli
DESKTOP_CRATE := llm-desktop

help: ## Show available make targets
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

cli: ## Build the CLI binary (release)
	cd $(CRATES_DIR) && cargo build --release -p $(CLI_CRATE)

cli-run: ## Run the CLI with --help
	cd $(CRATES_DIR) && cargo run -p $(CLI_CRATE) -- --help

cli-test: ## Run CLI-related tests
	cd $(CRATES_DIR) && cargo test -p $(CLI_CRATE)

desktop-dev: ## Run desktop app in development (Next.js + Tauri concurrently)
	cd $(DESKTOP_DIR) && pnpm dev

desktop-build: ## Build desktop app for production
	cd $(DESKTOP_DIR) && pnpm build && pnpm build:tauri

desktop-test: ## Run desktop frontend tests
	cd $(DESKTOP_DIR) && pnpm test

test: ## Run all tests (Rust + desktop)
	cd $(CRATES_DIR) && cargo test --workspace
	cd $(DESKTOP_DIR) && pnpm test

rust-check: ## Type-check the whole Rust workspace
	cd $(CRATES_DIR) && cargo check --workspace --all-targets

rust-test: ## Run all Rust tests
	cd $(CRATES_DIR) && cargo test --workspace

release: ## Build CLI binary and desktop app for release
	$(MAKE) cli
	$(MAKE) desktop-build
