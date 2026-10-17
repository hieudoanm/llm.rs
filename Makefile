.PHONY: help up down start stop e2e e2e-demo backend-build backend rust-check

FRONTEND_DIR := packages/frontend
CRATES_DIR := crates
BACKEND_CRATE_DIR := $(CRATES_DIR)/mlrs-server
DEMO_DIR := demo
DEMO_VIDEO := $(DEMO_DIR)/demo.mp4
BACKEND_PORT := 8000
FRONTEND_PORT := 3000
BACKEND_LOG := /tmp/mlrs-backend.log
FRONTEND_LOG := /tmp/mlrs-frontend.log
BACKEND_PID := /tmp/mlrs-backend.pid
FRONTEND_PID := /tmp/mlrs-frontend.pid
# The pure-Rust backend persists to a local SQLite file by default.
DATABASE_URL ?= sqlite:///.mlrs-data/mlrs.db
# Stops host-mode services and whatever still holds their ports (make start /
# make stop only — never run it against the compose stack).
STOP_SERVICES = test -f $(BACKEND_PID) && xargs -r kill -9 < $(BACKEND_PID) 2>/dev/null || true; test -f $(FRONTEND_PID) && xargs -r kill -9 < $(FRONTEND_PID) 2>/dev/null || true; lsof -ti :$(BACKEND_PORT) | xargs -r kill -9 2>/dev/null || true; lsof -ti :$(FRONTEND_PORT) | xargs -r kill -9 2>/dev/null || true; rm -f $(BACKEND_PID) $(FRONTEND_PID)
# Force-frees the published ports (backend, frontend) so `make up` /
# `make start` never trip over a stale host process or container.
FREE_PORTS = for port in $(BACKEND_PORT) $(FRONTEND_PORT); do lsof -ti :$$port | xargs -r kill -9 2>/dev/null || true; done
# Runs docker compose with the make-level port overrides (a shell-env prefix
# beats .env, so `make start BACKEND_PORT=8001` republishes the backend too).
COMPOSE = docker compose

help: ## Show available make targets
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

up: ## Start the full stack with Docker Compose (backend + frontend)
	@echo "==> Freeing ports $(BACKEND_PORT) and $(FRONTEND_PORT)..."
	-@$(FREE_PORTS)
	@echo "==> Starting the stack with Docker Compose (backend :$(BACKEND_PORT), frontend :$(FRONTEND_PORT))..."
	@$(COMPOSE) up -d --build || { echo "ERROR: docker compose up failed."; echo "If host-mode services are holding :$(BACKEND_PORT)/:$(FRONTEND_PORT), run 'make stop' first."; exit 1; }
	@echo "==> Waiting for services to be ready..."
	@for i in $$(seq 1 30); do \
		if curl -sf http://127.0.0.1:$(BACKEND_PORT)/healthz > /dev/null 2>&1 && curl -sf http://127.0.0.1:$(FRONTEND_PORT) > /dev/null 2>&1; then \
			echo "==> Stack ready."; \
			echo "    Backend:  http://127.0.0.1:$(BACKEND_PORT) (health: /healthz)"; \
			echo "    Frontend: http://127.0.0.1:$(FRONTEND_PORT)"; \
			break; \
		fi; \
		if [ $$i -eq 30 ]; then echo "ERROR: services not ready"; docker compose ps; docker compose logs --tail=20 backend frontend; exit 1; fi; \
		sleep 2; \
	done

down: ## Stop the Docker Compose stack (containers and data volumes are kept)
	-@docker compose stop
	@echo "==> Stack stopped. 'make up' brings it back; 'docker compose down' removes it."

start: ## Run backend (:8000) and frontend (:3000) on the host
	@echo "==> Freeing ports $(BACKEND_PORT) and $(FRONTEND_PORT)..."
	-@$(FREE_PORTS)
	@echo "==> Stopping any host-mode services on ports $(BACKEND_PORT) and $(FRONTEND_PORT)..."
	@echo "==> Starting backend on port $(BACKEND_PORT)..."
	cd $(CRATES_DIR) && DATABASE_URL=$(DATABASE_URL) MLRS_DATA_DIR=/tmp/mlrs-data MLRS_HOST=127.0.0.1 MLRS_PORT=$(BACKEND_PORT) cargo run -p mlrs-server > $(BACKEND_LOG) 2>&1 & echo $$! > $(BACKEND_PID)
	@echo "==> Starting frontend on port $(FRONTEND_PORT)..."
	cd $(FRONTEND_DIR) && pnpm run dev > $(FRONTEND_LOG) 2>&1 & echo $$! > $(FRONTEND_PID)
	@echo "==> Waiting for services to be ready..."
	@for i in $$(seq 1 30); do \
		if curl -sf http://127.0.0.1:$(BACKEND_PORT)/healthz > /dev/null 2>&1 && curl -sf http://127.0.0.1:$(FRONTEND_PORT) > /dev/null 2>&1; then \
			echo "==> Services ready."; \
			echo "    Backend:  http://127.0.0.1:$(BACKEND_PORT)"; \
			echo "    Frontend: http://127.0.0.1:$(FRONTEND_PORT)"; \
			break; \
		fi; \
		if [ $$i -eq 30 ]; then echo "ERROR: services not ready"; echo "--- $(BACKEND_LOG) ---"; tail -20 $(BACKEND_LOG) 2>/dev/null; echo "--- $(FRONTEND_LOG) ---"; tail -20 $(FRONTEND_LOG) 2>/dev/null; $(STOP_SERVICES); exit 1; fi; \
		sleep 2; \
	done

stop: ## Stop the host-mode backend and frontend
	-@$(STOP_SERVICES)
	@echo "==> Host services stopped."

backend-build: ## Build the pure-Rust backend binary (crates/mlrs-server)
	cd $(CRATES_DIR) && cargo build -p mlrs-server

backend: ## Run the pure-Rust backend on the host (SQLite by default)
	cd $(CRATES_DIR) && DATABASE_URL=$(DATABASE_URL) MLRS_DATA_DIR=/tmp/mlrs-data MLRS_HOST=127.0.0.1 MLRS_PORT=$(BACKEND_PORT) cargo run -p mlrs-server

rust-check: ## Type-check the whole Rust workspace
	cd $(CRATES_DIR) && cargo check --workspace --all-targets

e2e: ## Run e2e tests only (requires backend on :8000 and frontend on :3000)
	cd $(FRONTEND_DIR) && pnpm exec playwright test

e2e-demo: ## Capture all demo scenarios and save as demo/demo.mp4 and demo/demo.gif
	@echo "==> Starting the stack with Docker Compose..."
	@$(COMPOSE) up -d --build
	@echo "==> Waiting for services to be ready..."
	@for i in $$(seq 1 30); do \
		if curl -sf http://127.0.0.1:$(BACKEND_PORT)/healthz > /dev/null 2>&1 && curl -sf http://127.0.0.1:$(FRONTEND_PORT) > /dev/null 2>&1; then \
			echo "==> Services ready."; break; \
		fi; \
		if [ $$i -eq 30 ]; then echo "ERROR: services not ready"; docker compose ps; docker compose logs --tail=20 backend frontend; exit 1; fi; \
		sleep 2; \
	done
	@echo "==> Recording demo video (single worker, so scenes stay in order)..."
	@rm -rf $(FRONTEND_DIR)/test-results
	@cd $(FRONTEND_DIR) && DEMO=1 pnpm exec playwright test e2e/sml.e2e.test.ts --reporter=list --workers=1 || { \
		status=$$?; echo "==> Recording failed"; docker compose stop backend frontend; exit $$status; \
	}
	@echo "==> Saving video to $(DEMO_VIDEO)..."
	@mkdir -p $(DEMO_DIR)
	@VIDEO_FILES=$$(cd $(FRONTEND_DIR) && find test-results -name "video.webm" -type f 2>/dev/null | sort -t- -k4,4n); \
	if [ -n "$$VIDEO_FILES" ]; then \
		echo "$$VIDEO_FILES" | sed 's|^|file '"'"'$(CURDIR)/$(FRONTEND_DIR)/|;s|$$|'"'"'|' > $(DEMO_DIR)/concat.txt; \
		if command -v ffmpeg > /dev/null 2>&1; then \
			ffmpeg -y -f concat -safe -i $(DEMO_DIR)/concat.txt -c:v libx264 -pix_fmt yuv420p -crf 23 -preset medium "$(DEMO_VIDEO)" && rm -f $(DEMO_DIR)/concat.txt; \
			echo "==> Converting to GIF..."; \
			ffmpeg -y -i "$(DEMO_VIDEO)" -vf "fps=10,scale=800:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1]paletteuse" $(DEMO_DIR)/demo.gif || echo "WARNING: GIF conversion failed"; \
		else \
			echo "WARNING: ffmpeg not found, cannot concatenate/convert"; \
		fi; \
	else \
		echo "WARNING: no video file found"; \
	fi
	@echo "==> Stopping services..."
	-@docker compose stop backend frontend

reup: down up
