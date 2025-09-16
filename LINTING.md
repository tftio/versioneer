# How to Lint a Rust Project: Comprehensive Guide

A complete guide to setting up aggressive, professional-grade linting for Rust projects using modern tooling and best practices.

## Table of Contents

- [Core Philosophy](#core-philosophy)
- [Quick Start](#quick-start)
- [Essential Tools](#essential-tools)
- [Configuration](#configuration)
- [Git Hooks Setup](#git-hooks-setup)
- [Troubleshooting](#troubleshooting)
- [Advanced Configuration](#advanced-configuration)

## Core Philosophy

Aggressive linting catches issues before they become problems:
- **Fail fast**: Catch issues at commit time, not in production
- **Consistent quality**: Enforce coding standards automatically
- **Security first**: Scan for vulnerabilities continuously
- **No exceptions**: All code must pass all checks

## Quick Start

### 1. Install Core Tools

```bash
# TOML linting and formatting
cargo install taplo-cli

# Security and dependency management
cargo install cargo-audit
cargo install --locked cargo-deny
cargo install cargo-machete

# Git hooks manager
cargo install peter-hook  # or download from releases
```

### 2. Initialize Configuration

```bash
# Initialize dependency checking
cargo deny init

# Create hooks configuration (see below)
touch hooks.toml

# Install git hooks
peter-hook install
```

## Essential Tools

### TOML Linting

**Taplo** - The definitive TOML toolkit:
```bash
# Install
cargo install taplo-cli

# Usage
taplo format        # Format TOML files
taplo format --check    # Check formatting without modifying
taplo check         # Validate TOML syntax and schemas
```

### Rust Code Quality

**Clippy** with aggressive settings:
```bash
# Maximum strictness
cargo clippy --all-targets --all-features -- \
  -D warnings \
  -D clippy::all \
  -W clippy::pedantic \
  -W clippy::nursery \
  -W clippy::cargo
```

**rustfmt** for consistent formatting:
```bash
cargo fmt --check   # Verify formatting
cargo fmt          # Apply formatting
```

### Security & Dependencies

**cargo-audit** for vulnerability scanning:
```bash
cargo audit         # Check for known vulnerabilities
cargo audit fix     # Auto-fix where possible
```

**cargo-deny** for comprehensive dependency management:
```bash
cargo deny check    # Check licenses, advisories, and duplicates
cargo deny init     # Create configuration file
```

**cargo-machete** for unused dependencies:
```bash
cargo machete       # Find unused dependencies
cargo machete --fix # Remove unused dependencies (use with care)
```

### Documentation

Built-in documentation linting:
```bash
# Strict documentation checking
RUSTDOCFLAGS="-D warnings -D rustdoc::broken_intra_doc_links" \
  cargo doc --no-deps --document-private-items --all-features
```

## Configuration

### Cargo.toml Linting Configuration

Add workspace-level linting configuration:

```toml
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[lints.clippy]
# Deny common issues
enum_glob_use = "deny"
multiple_crate_versions = "warn"
wildcard_imports = "deny"

# Pedantic lints
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
cargo = { level = "warn", priority = -1 }
```

### cargo-deny Configuration (deny.toml)

```toml
[advisories]
# Fail on any security advisory
vulnerability = "deny"
unmaintained = "warn"

[licenses]
# Strictly control allowed licenses
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-3.0",
    "Unicode-DFS-2016",
]
confidence-threshold = 0.8

[bans]
# Deny multiple versions of same crate
multiple-versions = "deny"
wildcards = "deny"

[sources]
# Only allow crates.io
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

### rustfmt Configuration (rustfmt.toml)

```toml
edition = "2021"
max_width = 100
hard_tabs = false
tab_spaces = 4

# Stable features
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

## Git Hooks Setup

### Using peter-hook (Recommended)

Create `hooks.toml`:

```toml
# TOML linting hooks
[hooks.toml-format]
command = "taplo format --check"
modifies_repository = false
files = ["**/*.toml"]
description = "Check TOML file formatting"

[hooks.toml-lint]
command = "taplo check"
modifies_repository = false
files = ["**/*.toml"]
depends_on = ["toml-format"]
description = "Lint TOML files"

# Rust formatting and compilation
[hooks.rust-format]
command = "cargo fmt -- --check"
modifies_repository = false
files = ["**/*.rs", "Cargo.toml"]
description = "Check Rust code formatting"

[hooks.cargo-check]
command = "cargo check --all-targets --all-features"
modifies_repository = false
files = ["**/*.rs", "Cargo.toml"]
depends_on = ["rust-format"]
description = "Check compilation"

# Aggressive clippy linting
[hooks.clippy-aggressive]
command = "cargo clippy --all-targets --all-features -- -D warnings -D clippy::all -W clippy::pedantic -W clippy::nursery -W clippy::cargo"
modifies_repository = false
files = ["**/*.rs", "Cargo.toml"]
depends_on = ["cargo-check"]
description = "Aggressive clippy linting"

# Security auditing
[hooks.cargo-audit]
command = "cargo audit"
modifies_repository = false
run_always = true
description = "Security vulnerability scan"

[hooks.cargo-deny]
command = "cargo deny check"
modifies_repository = false
files = ["Cargo.toml", "Cargo.lock", "deny.toml"]
depends_on = ["cargo-audit"]
description = "Dependency compliance check"

[hooks.unused-deps]
command = "cargo machete"
modifies_repository = false
files = ["Cargo.toml"]
description = "Check for unused dependencies"

# Documentation and testing
[hooks.doc-check]
command = "cargo doc --no-deps --document-private-items --all-features"
modifies_repository = false
files = ["**/*.rs"]
env = { RUSTDOCFLAGS = "-D warnings -D rustdoc::broken_intra_doc_links" }
description = "Documentation check"

[hooks.test-suite]
command = "cargo test --all-features"
modifies_repository = false
files = ["**/*.rs", "Cargo.toml"]
depends_on = ["clippy-aggressive"]
description = "Run test suite"

# Pre-commit group - comprehensive checks
[groups.pre-commit]
includes = [
    "toml-format",
    "toml-lint",
    "rust-format",
    "cargo-check",
    "clippy-aggressive",
    "cargo-audit",
    "cargo-deny",
    "unused-deps",
    "doc-check",
    "test-suite"
]
execution = "sequential"
description = "Comprehensive pre-commit checks"

# Commit message validation
[hooks.commit-msg-length]
command = "sh -c 'head -n1 \"$1\" | wc -c | awk \"{if (\\$1 > 72) {print \\\"Commit message too long\\\"; exit 1}}\"' --"
modifies_repository = false
description = "Check commit message length"

[groups.commit-msg]
includes = ["commit-msg-length"]
execution = "sequential"
description = "Commit message validation"
```

Install hooks:
```bash
peter-hook install
```

### Alternative: pre-commit Framework

`.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/ComPWA/taplo-pre-commit
    rev: v0.9.3
    hooks:
      - id: taplo-format
      - id: taplo-lint

  - repo: https://github.com/doublify/pre-commit-rust
    rev: v1.0
    hooks:
      - id: fmt
        args: ["--", "--check"]
      - id: cargo-check
        args: ["--workspace"]
      - id: clippy
        args: ["--all-targets", "--all-features", "--", "-D", "warnings"]

  - repo: local
    hooks:
      - id: cargo-audit
        name: cargo-audit
        entry: cargo audit
        language: system
        pass_filenames: false

      - id: cargo-deny
        name: cargo-deny
        entry: cargo deny check
        language: system
        pass_filenames: false
```

## Troubleshooting

### Common Issues

**"Multiple versions of crate detected"**
```bash
# Find duplicate dependencies
cargo tree --duplicates

# Update dependencies
cargo update

# Use cargo-deny skip for unavoidable duplicates
```

**"License not allowed"**
- Review `deny.toml` license allowlist
- Check if dependency license changed
- Add exception for specific crates if justified

**"Unmaintained crate advisory"**
- Find maintained alternatives
- Pin to last known-good version temporarily
- Consider forking if critical

### Security Advisory Resolution

```bash
# View detailed advisory info
cargo audit --format json

# Check for fixes
cargo update

# Ignore specific advisories (use sparingly)
# Add to deny.toml:
# [advisories]
# ignore = ["RUSTSEC-2024-XXXX"]
```

## Advanced Configuration

### CI/CD Integration

**GitHub Actions example:**
```yaml
name: Lint
on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install linting tools
        run: |
          cargo install taplo-cli cargo-audit cargo-deny cargo-machete

      - name: TOML checks
        run: |
          taplo format --check
          taplo check

      - name: Rust checks
        run: |
          cargo fmt --check
          cargo clippy --all-targets --all-features -- -D warnings

      - name: Security audit
        run: cargo audit

      - name: Dependency compliance
        run: cargo deny check

      - name: Documentation
        run: cargo doc --no-deps --document-private-items
        env:
          RUSTDOCFLAGS: -D warnings
```

### Performance Optimization

**Parallel execution where safe:**
```toml
# In hooks.toml
[groups.pre-commit-parallel]
includes = ["cargo-audit", "unused-deps"]
execution = "parallel"  # These don't interfere
```

**Incremental checks:**
```bash
# Only check changed files
cargo clippy --all-targets -- -D warnings $(git diff --name-only HEAD~1 | grep -E '\.(rs)$')
```

### Project-Specific Customization

**Workspace-specific overrides:**
```toml
# Cargo.toml - per-package lints
[package.lints]
workspace = true

# Override specific lint for this package
[package.lints.clippy]
too_many_arguments = "allow"
```

## Best Practices

1. **Start strict**: It's easier to relax rules than enforce them later
2. **Document exceptions**: Always justify lint exceptions in comments
3. **Regular updates**: Keep linting tools updated
4. **Team alignment**: Ensure all contributors use the same tools
5. **CI enforcement**: Never rely solely on local hooks

## Tool Installation Script

Save as `install-lint-tools.sh`:
```bash
#!/bin/bash
set -e

echo "Installing Rust linting tools..."

# Core tools
cargo install taplo-cli
cargo install cargo-audit
cargo install --locked cargo-deny
cargo install cargo-machete

# Optional advanced tools
cargo install cargo-semver-checks
cargo install cargo-geiger

# Git hooks manager
if ! command -v peter-hook &> /dev/null; then
    echo "Installing peter-hook..."
    cargo install peter-hook
fi

echo "✅ All linting tools installed successfully!"
echo "Next steps:"
echo "1. Run 'cargo deny init' to create deny.toml"
echo "2. Create hooks.toml configuration"
echo "3. Run 'peter-hook install' to set up git hooks"
```

---

This comprehensive linting setup ensures code quality, security, and consistency across your Rust projects. The aggressive configuration catches issues early and maintains professional standards throughout development.