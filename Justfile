# Lexicon development commands

set dotenv-load := true

# Default recipe: list all available recipes
default:
    @just --list
    
# Run all tests
test:
    cargo test --workspace

# Run tests for a specific crate
test-crate crate:
    cargo test -p lexicon-{{crate}}

# Build the workspace
build:
    cargo build --workspace

# Build in release mode
build-release:
    cargo build --release

# Run clippy on the entire workspace
lint:
    cargo clippy --workspace -- -D warnings

# Format all code
fmt:
    cargo fmt --all

# Check formatting without modifying
fmt-check:
    cargo fmt --all -- --check

# Run the CLI
run *args:
    cargo run -- {{args}}

# Run the TUI
tui:
    cargo run -- tui

# Run verification
verify:
    cargo run -- verify

# Initialize lexicon in a temp directory (for testing)
demo:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    echo "Demo directory: $dir"
    cd "$dir"
    cargo run --manifest-path {{justfile_directory()}}/Cargo.toml -- init

# Check everything (fmt, lint, test)
check: fmt-check lint test

# Watch for changes and run tests
watch:
    cargo watch -x 'test --workspace'

# Count lines of code
loc:
    tokei crates/

# Show workspace dependency tree
deps:
    cargo tree --workspace --depth 1

# Start the docs site in dev mode
docs-dev:
    cd docs && pnpm dev

# Build the docs site
docs-build:
    cd docs && pnpm build

# Preview the built docs site
docs-preview:
    cd docs && pnpm preview

# Clean build artifacts
clean:
    cargo clean
