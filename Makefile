# blog.rs — development tasks. Run `make` for the list.

CARGO ?= cargo
WATCH_PATHS := main.rs build.rs Cargo.toml Cargo.lock src content styles assets macros
WATCH_ARGS := $(foreach path,$(WATCH_PATHS),-w $(path))

.DEFAULT_GOAL := help
.PHONY: help dev run build release test fmt fmt-check lint check ci clean up down

help:
	@echo "blog.rs — available targets:"
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) \
		| awk 'BEGIN { FS = ":.*## " }; { printf "  \033[36m%-10s\033[0m %s\n", $$1, $$2 }'

dev:
	@command -v cargo-watch >/dev/null 2>&1 \
		|| { echo "cargo-watch is missing: cargo install cargo-watch"; exit 1; }
	$(CARGO) watch -c $(WATCH_ARGS) -x run

run:
	$(CARGO) run

build:
	$(CARGO) build

release:
	$(CARGO) build --release

test:
	$(CARGO) test

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

check:
	$(CARGO) check --all-targets --all-features

lint:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

ci: fmt-check check lint test ## Everything the CI workflow runs

clean:
	$(CARGO) clean
	rm -rf build

up:
	docker compose up --build

down:
	docker compose down
