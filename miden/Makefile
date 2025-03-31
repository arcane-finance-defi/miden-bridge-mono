.DEFAULT_GOAL := help

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

# -- variables --------------------------------------------------------------------------------------

WARNINGS=RUSTDOCFLAGS="-D warnings"
DEBUG_ASSERTIONS=RUSTFLAGS="-C debug-assertions"
ALL_FEATURES_BUT_ASYNC=--features testing
BUILD_NOTE_ERRORS=BUILD_NOTE_ERRORS=1

# -- linting --------------------------------------------------------------------------------------

.PHONY: clippy
clippy: ## Runs Clippy with configs
	cargo clippy --all-features --all-targets -- -D warnings


.PHONY: fix
fix: ## Runs Fix with configs
	cargo fix --allow-staged --allow-dirty --all-targets --all-features


.PHONY: format
format: ## Runs Format using nightly toolchain
	cargo +nightly fmt


.PHONY: format-check
format-check: ## Runs Format using nightly toolchain but only in check mode
	cargo +nightly fmt --check


.PHONY: lint
lint: format fix clippy ## Runs all linting tasks at once (Clippy, fixing, formatting)

# --- testing -------------------------------------------------------------------------------------

.PHONY: test-build
test-build: ## Build the test binary
	$(DEBUG_ASSERTIONS) cargo nextest run --cargo-profile test-release $(ALL_FEATURES_BUT_ASYNC) --no-run


.PHONY: test-default
test-default: ## Run default tests 
	$(DEBUG_ASSERTIONS) cargo nextest run --profile default --cargo-profile test-release $(ALL_FEATURES_BUT_ASYNC)


.PHONY: test
test: test-default ## Run all tests

# --- checking ------------------------------------------------------------------------------------

.PHONY: check
check: ## Check all targets and features for errors without code generation
	${BUILD_NOTE_ERRORS} cargo check --all-targets --all-features

# --- building ------------------------------------------------------------------------------------

.PHONY: build
build: ## By default we should build in release mode
	${BUILD_NOTE_ERRORS} cargo build --release


.PHONY: build-no-std
build-no-std: ## Build without the standard library
	${BUILD_NOTE_ERRORS} cargo build --no-default-features --target wasm32-unknown-unknown --lib
