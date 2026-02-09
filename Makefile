# Makefile for knowledge-index
# Run CI checks locally using Docker to match GitHub Actions environment

.PHONY: help ci ci-quick ci-msrv ci-format ci-clippy ci-test ci-test-verbose ci-doc ci-publish-check build release clean

# Default target
help:
	@echo "knowledge-index development commands"
	@echo ""
	@echo "CI Commands (Docker-based, matches GitHub Actions):"
	@echo "  make ci              - Run full CI pipeline (format, clippy, build, test, doc)"
	@echo "  make ci-quick        - Run quick CI checks (format, clippy only)"
	@echo "  make ci-msrv         - Check minimum supported Rust version (1.88)"
	@echo "  make ci-format       - Check code formatting"
	@echo "  make ci-clippy       - Run clippy lints"
	@echo "  make ci-test         - Run all tests"
	@echo "  make ci-test-verbose - Run all tests with output (for debugging)"
	@echo "  make ci-doc          - Build documentation with warnings as errors"
	@echo "  make ci-publish-check - Dry-run crates.io publish"
	@echo ""
	@echo "Local Commands (uses local Rust toolchain):"
	@echo "  make build           - Build debug version"
	@echo "  make release         - Build release version"
	@echo "  make test            - Run tests"
	@echo "  make fmt             - Format code"
	@echo "  make lint            - Run clippy"
	@echo "  make clean           - Clean build artifacts"

# =============================================================================
# CI Commands - Use Docker to match GitHub Actions environment
# =============================================================================

# Full CI pipeline (matches GitHub Actions ci.yml)
ci:
	@echo "🔄 Running full CI pipeline in Docker..."
	docker run --rm -v $(PWD):/app -w /app rust:latest sh -c "\
		rustup component add clippy rustfmt && \
		echo '=== Step 1/6: Check formatting ===' && \
		cargo fmt --all --check && \
		echo '✅ Formatting OK' && \
		echo '' && \
		echo '=== Step 2/6: Clippy lints ===' && \
		cargo clippy --all-targets --all-features -- -D warnings && \
		echo '✅ Clippy OK' && \
		echo '' && \
		echo '=== Step 3/6: Build release ===' && \
		cargo build --release && \
		echo '✅ Build OK' && \
		echo '' && \
		echo '=== Step 4/6: Run tests ===' && \
		cargo test --all-features && \
		echo '✅ Tests OK' && \
		echo '' && \
		echo '=== Step 5/6: Documentation ===' && \
		RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features && \
		echo '✅ Documentation OK' && \
		echo '' && \
		echo '=== Step 6/6: Publish dry-run ===' && \
		cargo publish --dry-run --allow-dirty && \
		echo '✅ Publish check OK' && \
		echo '' && \
		echo '========================================' && \
		echo '✅ ALL CI STEPS PASSED!' && \
		echo '========================================' \
	"

# Quick CI checks (format + clippy only, faster)
ci-quick:
	@echo "🔄 Running quick CI checks in Docker..."
	docker run --rm -v $(PWD):/app -w /app rust:latest sh -c "\
		rustup component add clippy rustfmt && \
		echo '=== Check formatting ===' && \
		cargo fmt --all --check && \
		echo '✅ Formatting OK' && \
		echo '' && \
		echo '=== Clippy lints ===' && \
		cargo clippy --all-targets --all-features -- -D warnings && \
		echo '✅ Clippy OK' \
	"

# MSRV check (Minimum Supported Rust Version)
ci-msrv:
	@echo "🔄 Checking MSRV (Rust 1.88) in Docker..."
	docker run --rm -v $(PWD):/app -w /app rust:1.88.0 cargo check --all-features
	@echo "✅ MSRV check passed"

# Format check only
ci-format:
	@echo "🔄 Checking formatting in Docker..."
	docker run --rm -v $(PWD):/app -w /app rust:latest sh -c "\
		rustup component add rustfmt && \
		cargo fmt --all --check \
	"
	@echo "✅ Formatting OK"

# Clippy only
ci-clippy:
	@echo "🔄 Running clippy in Docker..."
	docker run --rm -v $(PWD):/app -w /app rust:latest sh -c "\
		rustup component add clippy && \
		cargo clippy --all-targets --all-features -- -D warnings \
	"
	@echo "✅ Clippy OK"

# Tests only
ci-test:
	@echo "🔄 Running tests in Docker..."
	docker run --rm -v $(PWD):/app -w /app rust:latest cargo test --all-features
	@echo "✅ Tests OK"

# Tests with verbose output (for debugging)
ci-test-verbose:
	@echo "🔄 Running tests in Docker (verbose)..."
	docker run --rm -v $(PWD):/app -w /app rust:latest cargo test --all-features -- --nocapture
	@echo "✅ Tests OK"

# Documentation build
ci-doc:
	@echo "🔄 Building documentation in Docker..."
	docker run --rm -v $(PWD):/app -w /app rust:latest sh -c "\
		RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features \
	"
	@echo "✅ Documentation OK"

# Publish dry-run
ci-publish-check:
	@echo "🔄 Running publish dry-run in Docker..."
	docker run --rm -v $(PWD):/app -w /app rust:latest cargo publish --dry-run --allow-dirty
	@echo "✅ Publish check OK"

# =============================================================================
# Local Commands - Use local Rust toolchain (faster, but may differ from CI)
# =============================================================================

# Build debug
build:
	cargo build

# Build release
release:
	cargo build --release

# Run tests
test:
	cargo test --all-features

# Format code
fmt:
	cargo fmt --all

# Run clippy
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Clean build artifacts
clean:
	cargo clean
