# Use bash for array/flow control support
set shell := ["bash", "-c"]
set dotenv-load := true         # Auto-load .env

# ============ Configuration Variables ============
BIN              := env_var_or_default("BIN", `cargo metadata --no-deps --format-version 1 2>/dev/null | jaq -r '.packages[] | select(.targets[]?.kind[]? == "bin") | .targets[] | select(.kind[] == "bin") | .name' 2>/dev/null | head -1 || echo "cli"`)
ESSENTIAL_TOOLS  := "cargo-nextest typos-cli jaq"
QUALITY_TOOLS    := "cargo-audit cargo-deny cargo-tarpaulin"
UTILITY_TOOLS    := "git-cliff cargo-binstall cargo-cache cargo-watch cargo-outdated tokei"
TOOLS            := ESSENTIAL_TOOLS + " " + QUALITY_TOOLS + " " + UTILITY_TOOLS

# ============ Smart Detection ============
WORKSPACE        := `grep -q '^[[:space:]]*\[workspace\]' Cargo.toml && echo 1 || echo 0`
WORKSPACE_FLAG   := `if grep -q '^[[:space:]]*\[workspace\]' Cargo.toml; then echo '--workspace'; fi`
HAS_ANY_LIB      := `find . -maxdepth 4 -path './target' -prune -o -name lib.rs -print -quit | grep -q . && echo 1 || echo 0`
NIGHTLY_AVAILABLE:= `rustup toolchain list 2>/dev/null | grep -q nightly && echo 1 || echo 0`
TEST_RUNNER      := `command -v cargo-nextest >/dev/null 2>&1 && echo 'nextest run' || echo 'test'`
ALL_TARGETS_FLAG := "--all-targets"
ALL_FEATURES_FLAG:= "--all-features"  # kept for backward compatibility
FEATURES_FLAG    := `if [ -n "$FEATURES" ]; then echo "--features $FEATURES"; else echo "${ALL_FEATURES_FLAG}"; fi`
DISPLAY_FEATURES_FLAG := `if [ -n "$FEATURES" ]; then echo "--features $FEATURES"; else echo "${ALL_FEATURES_FLAG}"; fi`

# Package enumeration (for info / list-packages)
PACKAGES := `cargo metadata --no-deps --format-version 1 2>/dev/null | jaq -r '.workspace_members[]' 2>/dev/null | awk -F'[/#]' '{print $(NF-1)}' | tr '\n' ' ' | sed 's/ $//' || true`

help:
    @echo "🦀 Rust Project Automation Script (Enhanced)"
    @echo "============================================"
    @echo "Mode: $([ {{WORKSPACE}} -eq 1 ] && echo Workspace || echo Single )  |  Bin: {{BIN}}"
    @echo "Features: $([ -n \"$FEATURES\" ] && echo "$FEATURES (custom)" || echo 'ALL (--all-features)')"
    @echo ""
    @echo "Core: dev ci health info env"
    @echo "Test: test test-unit test-doc test-watch test-integration coverage"
    @echo "Quality: format format-check lint lint-strict lint-docs typos security"
    @echo "Deps: deps outdated list-packages changelog"
    @echo "Build: check build build-release run"
    @echo "Misc: bench stats"
    @echo "Release: release-dry release-exec"
    @echo "(Aliases: f fc b br t tu td cov li mc fx bn st od rd re)"
    @echo "(Use: FEATURES=foo,bar just build)"

env:
    @echo "BIN={{BIN}}"
    @echo "WORKSPACE=$([ {{WORKSPACE}} -eq 1 ] && echo 1 || echo 0)"
    @echo "FEATURES_INPUT=${FEATURES:-<all>}"
    @echo "FEATURES_FLAG={{DISPLAY_FEATURES_FLAG}}"
    @echo "NIGHTLY=$([ {{NIGHTLY_AVAILABLE}} -eq 1 ] && echo yes || echo no)"


default: dev

# Quick minimal check (without all-targets/all-features)
minimal-check:
    @echo "🧪 Quick minimal check (single target / default features)..."
    @if [ {{WORKSPACE}} -eq 1 ]; then \
        cargo check {{WORKSPACE_FLAG}}; \
    else \
        cargo check; \
    fi

# ============================================
# Core Workflows
# ============================================

# Daily development workflow - fast feedback
dev: format check test-unit lint
    @echo "✅ Development workflow complete"

# Strict linting with pedantic checks
lint-strict:
    @echo "🔎 Strict Clippy check (pedantic/nursery/cargo)..."
    @FLAGS="-D warnings -D clippy::pedantic -D clippy::nursery -D clippy::cargo -A clippy::multiple-crate-versions -A clippy::cargo-common-metadata"; \
    if [ {{WORKSPACE}} -eq 1 ]; then \
        cargo clippy {{WORKSPACE_FLAG}} --all-targets --all-features -- $FLAGS; \
    else \
        cargo clippy --all-targets --all-features -- $FLAGS; \
    fi

# CI/CD complete check pipeline
ci: format-check lint lint-dependencies typos security check test coverage build-release
    @echo "✅ CI/CD pipeline complete"

# Auto-fix with clippy
fix:
    @echo "🔧 Attempting auto-fix (clippy --fix)..."
    @set -e; MODE=0; \
    if cargo clippy {{WORKSPACE_FLAG}} --all-targets --fix --allow-dirty --allow-staged -- -D warnings 2>/dev/null; then MODE=1; fi; \
    echo "  → Formatting..."; just format; \
    echo "Complete (fix status: $${MODE})"

# ============================================
# Build Commands
# ============================================

# Quick compile check
check:
    @echo "🔍 Checking code compilation..."
    @if [ {{WORKSPACE}} -eq 1 ]; then \
    cargo check {{WORKSPACE_FLAG}} {{ALL_TARGETS_FLAG}} {{FEATURES_FLAG}}; \
    else \
    cargo check {{ALL_TARGETS_FLAG}} {{FEATURES_FLAG}}; \
    fi

# Debug mode build
build:
    @echo "🔨 Debug build..."
    @if [ {{WORKSPACE}} -eq 1 ]; then \
    cargo build {{WORKSPACE_FLAG}} {{FEATURES_FLAG}}; \
    else \
    cargo build {{FEATURES_FLAG}}; \
    fi

# Release mode build
build-release:
    @echo "🚀 Release build..."
    @if [ {{WORKSPACE}} -eq 1 ]; then \
    RUSTFLAGS="-C target-cpu=native" cargo build --release {{WORKSPACE_FLAG}} {{FEATURES_FLAG}}; \
    else \
    RUSTFLAGS="-C target-cpu=native" cargo build --release {{FEATURES_FLAG}}; \
    fi

# Watch source files and continuously check
watch: (_ensure-tool "cargo-watch")
    @echo "👀 Watching and continuously running cargo check (Ctrl+C to exit)"
    cargo watch -x check

# Integration tests
test-integration: (_ensure-tool "cargo-nextest")
    @echo "🧪 Integration tests (tests/)..."
    @if [ -d tests ]; then \
        if command -v cargo-nextest >/dev/null 2>&1; then \
            if [ {{WORKSPACE}} -eq 1 ]; then cargo nextest run {{WORKSPACE_FLAG}} --test '*'; else cargo nextest run --test '*'; fi; \
        else \
            if [ {{WORKSPACE}} -eq 1 ]; then cargo test {{WORKSPACE_FLAG}} --test '*'; else cargo test --test '*'; fi; \
        fi; \
    else \
        echo "(Skipped) No tests/ directory found"; \
    fi

# Benchmark tests
bench:
    @HAS_BENCHES=`test -d benches && find benches -name '*.rs' -type f | head -1 | grep -q . && echo 1 || echo 0`; \
    if [ $$HAS_BENCHES -eq 1 ]; then \
        echo "🏁 Running benchmark tests..."; \
        if [ {{WORKSPACE}} -eq 1 ]; then cargo bench {{WORKSPACE_FLAG}}; else cargo bench; fi; \
    else \
        echo "(Skipped) No benches/"; \
    fi

# Code statistics
stats:
    @echo "📊 Code statistics"; \
    rs_files=$(find . -name '*.rs' -not -path './target/*' | wc -l); \
    lines=$(find . -name '*.rs' -not -path './target/*' -exec cat {} + | wc -l); \
    echo "  Rust files: $$rs_files"; \
    echo "  Lines of code: $$lines"; \
    if command -v tokei >/dev/null 2>&1; then echo ""; tokei || true; else echo "(Tip) Install tokei for detailed statistics"; fi

# Check outdated dependencies
outdated: (_ensure-tool "cargo-outdated")
    @echo "📦 Checking dependency updates (cargo-outdated)..."
    @if [ {{WORKSPACE}} -eq 1 ]; then \
        cargo outdated {{WORKSPACE_FLAG}} || true; \
    else \
        cargo outdated || true; \
    fi

# Show dependency tree
deps:
    @echo "🌳 Dependency tree..."
    @if [ {{WORKSPACE}} -eq 1 ]; then \
        cargo tree {{WORKSPACE_FLAG}}; \
    else \
        cargo tree; \
    fi

# ============================================
# Test Commands
# ============================================

# Run all tests
test: test-unit test-doc
    @echo "✅ All tests complete"

# Run unit tests
test-unit: (_ensure-tool "cargo-nextest")
    @echo "🧪 Running unit tests..."
    @if command -v cargo-nextest >/dev/null 2>&1; then \
        if [ {{WORKSPACE}} -eq 1 ]; then \
            cargo nextest run {{ALL_TARGETS_FLAG}} {{FEATURES_FLAG}} {{WORKSPACE_FLAG}}; \
        else \
            cargo nextest run {{ALL_TARGETS_FLAG}} {{FEATURES_FLAG}}; \
        fi; \
    else \
        if [ {{WORKSPACE}} -eq 1 ]; then \
            cargo test {{WORKSPACE_FLAG}}; \
        else \
            cargo test --lib --bins; \
        fi; \
    fi

# Run documentation tests
test-doc:
    @echo "📚 Running documentation tests..."
    @if [ {{HAS_ANY_LIB}} -eq 0 ]; then \
        echo "(Skipped) No lib.rs found (no doc test targets)"; \
    else \
        if [ {{WORKSPACE}} -eq 1 ]; then \
            cargo test --doc {{WORKSPACE_FLAG}}; \
        else \
            cargo test --doc; \
        fi; \
    fi

# Coverage report (only when cargo-tarpaulin is available)
coverage: (_ensure-tool "cargo-tarpaulin")
    @echo "📊 Generating test coverage..."
    @if command -v cargo-tarpaulin >/dev/null 2>&1; then \
        if [ {{WORKSPACE}} -eq 1 ]; then \
            cargo tarpaulin {{WORKSPACE_FLAG}} {{FEATURES_FLAG}} --timeout 120 --out Html --out Lcov --out Xml --output-dir coverage; \
        else \
            cargo tarpaulin {{FEATURES_FLAG}} --timeout 120 --out Html --out Lcov --out Xml --output-dir coverage; \
        fi; \
        echo "  → Output directory: coverage/"; \
    else \
        echo "(Skipped) cargo-tarpaulin not installed"; \
    fi

# Watch and continuously run tests
test-watch: (_ensure-tool "cargo-watch") (_ensure-tool "cargo-nextest")
    @echo "🔄 Watching and continuously running tests..."
    @if command -v cargo-nextest >/dev/null 2>&1; then \
        if [ {{WORKSPACE}} -eq 1 ]; then \
            cargo watch -x "nextest run --all-targets {{FEATURES_FLAG}} --workspace"; \
        else \
            cargo watch -x "nextest run --all-targets {{FEATURES_FLAG}}"; \
        fi; \
    else \
        if [ {{WORKSPACE}} -eq 1 ]; then \
            cargo watch -x 'test --workspace'; \
        else \
            cargo watch -x 'test --lib --bins'; \
        fi; \
    fi

# ============================================
# Code Quality
# ============================================

# Format code (smart nightly detection)
format:
    @echo "✨ Formatting code..."
    @if [ {{NIGHTLY_AVAILABLE}} -eq 1 ]; then \
        if rustup component list --toolchain nightly | grep -q 'rustfmt.*(installed)'; then cargo +nightly fmt --all; \
        else echo "  → Installing nightly rustfmt..."; rustup component add rustfmt --toolchain nightly && cargo +nightly fmt --all; fi; \
    else cargo fmt --all; fi

# Check code format (don't modify)
format-check:
    @echo "🔍 Checking code format..."
    @if rustup toolchain list | grep -q nightly; then \
        cargo +nightly fmt --all -- --check; \
    else \
        cargo fmt --all -- --check; \
    fi

# Spell check
typos: (_ensure-tool "typos-cli")
    @echo "📝 Spell checking..."
    typos

# Clippy code check
lint:
    @echo "🧹 Running Clippy check..."
    @if [ {{WORKSPACE}} -eq 1 ]; then \
        cargo clippy {{WORKSPACE_FLAG}} --all-targets --tests --benches -- -D warnings; \
    else \
        cargo clippy --all-targets --tests --benches -- -D warnings; \
    fi

# Check documentation
lint-docs:
    @echo "📚 Documentation compilation check..."
    @if rustup toolchain list | grep -q nightly; then \
        if [ {{HAS_ANY_LIB}} -eq 0 ]; then \
            echo "(Skipped) No lib targets"; \
        else \
            if [ {{WORKSPACE}} -eq 1 ]; then \
                cargo +nightly rustdoc {{WORKSPACE_FLAG}} -- -Zunstable-options --check -Dwarnings; \
            else \
                cargo +nightly rustdoc --lib -- -Zunstable-options --check -Dwarnings; \
            fi; \
        fi; \
    else \
        echo "  → Nightly toolchain required for documentation check"; \
    fi

# Dependency check
lint-dependencies: (_ensure-tool "cargo-deny")
    @echo "🔍 Dependency compliance check..."
    @if [ {{WORKSPACE}} -eq 1 ]; then \
        cargo deny check licenses bans sources advisories; \
    else \
        cargo deny check licenses bans sources advisories; \
    fi

# Security audit (using cargo-audit)
security: (_ensure-tool "cargo-audit")
    @echo "🔒 Running security audit (cargo-audit)..."
    cargo audit || echo "⚠️  Audit failed or risks exist (exit code preserved)"

# ============================================
# Environment Management
# ============================================

# Setup development environment
setup:
    @echo "🔧 Installing development tools..."
    @for tool in {{TOOLS}}; do \
        just _install-tool "$tool"; \
    done
    @echo "✅ Development environment setup complete"

# Clean build artifacts
clean:
    @echo "🧹 Cleaning build artifacts..."
    cargo clean

# Clean only cache (don't touch target)
clean-cache: (_ensure-tool "cargo-cache")
    @echo "🧹 Cleaning Cargo cache..."
    cargo cache --autoclean || true

# Generate changelog
changelog: (_ensure-tool "git-cliff")
    @echo "📝 Generating changelog..."
    git cliff -o CHANGELOG.md

# Generate documentation
docs:
    @echo "📚 Generating documentation..."
    cargo doc --no-deps

# Generate and open documentation
docs-open: docs
    @echo "🌐 Opening documentation..."
    @if command -v xdg-open >/dev/null 2>&1; then \
        xdg-open target/doc/{{BIN}}/index.html >/dev/null 2>&1 || \
        xdg-open target/doc/index.html >/dev/null 2>&1 || \
        echo "  → Cannot auto-open browser, please manually visit target/doc/"; \
    elif command -v open >/dev/null 2>&1; then \
        open target/doc/{{BIN}}/index.html || \
        open target/doc/index.html || \
        echo "  → Cannot auto-open browser, please manually visit target/doc/"; \
    else \
        echo "  → Documentation generated at target/doc/"; \
    fi

# Run binary
run:
    @echo "▶️ Running binary: {{BIN}}"
    cargo run --bin {{BIN}}

# Display package information
info:
    @echo "📋 Project information" \
    && echo "Mode: $([ {{WORKSPACE}} -eq 1 ] && echo Workspace || echo Single )" \
    && echo "Binary: {{BIN}}" \
    && echo -n "Packages: " \
    && cargo metadata --no-deps --format-version 1 2>/dev/null | jaq -r '.workspace_members[]' 2>/dev/null | awk -F'[/#]' '{print $(NF-1)}' | tr '\n' ' ' | sed 's/ $//' \
    && echo "" \
    && echo "Nightly available: $([ {{NIGHTLY_AVAILABLE}} -eq 1 ] && echo Yes || echo No)" \
    && echo "Test runner: {{TEST_RUNNER}}"

list-packages:
    @echo "📦 Package list:" \
    && (echo "${PACKAGES}" | tr ' ' '\n' | sed 's/^/  - /' || echo "(none)")

# Simple health check
health:
    @echo "🏥 Health check..."; issues=0; \
    echo "  ▶ Compile check"; cargo check {{WORKSPACE_FLAG}} --quiet || issues=$$((issues+1)); \
    echo "  ▶ Unit tests"; cargo test {{WORKSPACE_FLAG}} --lib --quiet || true; \
    echo "  ▶ Format check"; cargo fmt --all -- --check >/dev/null 2>&1 || issues=$$((issues+1)); \
    echo "  ▶ Clippy (quick)"; cargo clippy {{WORKSPACE_FLAG}} --all-targets -- -D warnings >/dev/null 2>&1 || issues=$$((issues+1)); \
    if [ $$issues -eq 0 ]; then echo "✅ Health: OK"; else echo "⚠️  Found $$issues issues"; fi

# Preview release (dry run)
release-dry level="patch": (_ensure-tool "cargo-release")
    @echo "🏷 Preview release: {{level}}"
    cargo release {{level}} --dry-run {{WORKSPACE_FLAG}}

# Execute release
release-exec level="patch": (_ensure-tool "cargo-release")
    @echo "🏷 Execute release: {{level}}"
    cargo release {{level}} --execute {{WORKSPACE_FLAG}}

# ============================================
# Internal Tool Functions
# ============================================

# Generic tool ensure function
_ensure-tool tool:
    @bin={{tool}}; \
    # Handle *-cli tool names \
    if echo "$bin" | grep -q -- '-cli$'; then \
        bin=${bin%-cli}; \
    fi; \
    if ! command -v "$bin" >/dev/null 2>&1; then \
        just _install-tool {{tool}}; \
    fi

# Install tool (prefer binstall)
_install-tool tool:
    @echo "📦 Installing {{tool}}..."
    @if command -v cargo-binstall >/dev/null 2>&1; then \
        cargo binstall {{tool}} --no-confirm --log-level warn || cargo install {{tool}}; \
    else \
        cargo install {{tool}}; \
    fi

###############################################
# Convenient Aliases (English/legacy compatibility)
###############################################
alias f := format
alias fc := format-check
alias b := build
alias br := build-release
alias t := test
alias tu := test-unit
alias td := test-doc
alias cov := coverage
alias ls-pkg := list-packages
alias sec := security
alias h := health
alias i := info
alias rd := release-dry
alias re := release-exec
alias li := lint-strict
alias mc := minimal-check
alias fx := fix
alias ti := test-integration
alias bn := bench
alias st := stats
alias od := outdated
alias dch := lint-docs