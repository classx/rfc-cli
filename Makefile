.PHONY: build release test clean help

# Default target
help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## Build debug binary
	cargo build

release: ## Build release binary
	cargo build --release

test: ## Run all tests
	cargo test

clean: ## Remove build artifacts
	cargo clean

install: release ## Install binary to ~/.cargo
