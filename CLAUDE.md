# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Essential Commands

### Version Management
- `versioneer patch` - Bump patch version (1.2.3 → 1.2.4)
- `versioneer minor` - Bump minor version (1.2.3 → 1.3.0)
- `versioneer major` - Bump major version (1.2.3 → 2.0.0)
- `versioneer show` - Show current version
- `versioneer sync` - Synchronize all version files
- `versioneer verify` - Verify all versions are synchronized
- `versioneer tag` - Create git tag for current version
- `versioneer reset [VERSION]` - Reset version to specific value or 0.0.0

### Utility Commands
- `versioneer completions <shell>` - Generate shell completions (bash/zsh/fish)
- `versioneer doctor` - Health check and update notifications
- `versioneer update` - Self-update to latest version
- `versioneer status` - Show version and build system status
- `versioneer version` - Show versioneer version

### Development Commands
- `cargo build` - Build the project
- `cargo test` - Run all tests
- `cargo clippy --all-targets -- -D warnings` - Lint code
- `cargo fmt` - Format code

## Architecture

**versioneer** is a CLI tool for synchronizing VERSION files with build system version declarations (Cargo.toml, pyproject.toml, package.json).

### Key Features
- Atomic version bumping across all build systems
- Git tag creation with customizable formats
- Version synchronization verification
- Supports Cargo (Rust), Python (pyproject.toml), and Node.js (package.json)

### Tool Behavior
- **Exit Codes**: 0 (success), 1 (error)
- **Subcommands**: Uses clap derive pattern with subcommands
- **TTY Detection**: Colorful output for terminals, plain text for pipes
- **Git Integration**: Optional git tagging with customizable formats

## Version Management Workflow

1. Edit code and commit changes
2. Run `versioneer patch|minor|major` to bump version
3. Optionally run `versioneer tag` to create git tag
4. Push commits and tags to trigger releases

For automated releases, use `./scripts/release.sh` which handles the complete workflow with quality gates.
