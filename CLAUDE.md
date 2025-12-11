# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Essential Commands

### Version Management

**Standard Mode (single project)**:
- `versioneer patch` - Bump patch version (1.2.3 → 1.2.4)
- `versioneer minor` - Bump minor version (1.2.3 → 1.3.0)
- `versioneer major` - Bump major version (1.2.3 → 2.0.0)
- `versioneer sync` - Synchronize all version files
- `versioneer reset [VERSION]` - Reset version to specific value or 0.0.0

**Cascade Mode (monorepo with multiple manifests)**:
- `versioneer patch --cascade` - Bump patch and update all discovered manifests recursively
- `versioneer minor --cascade` - Bump minor and update all discovered manifests recursively
- `versioneer major --cascade` - Bump major and update all discovered manifests recursively
- `versioneer sync --cascade` - Sync all discovered manifests to VERSION file
- `versioneer reset 1.2.3 --cascade` - Reset version and update all discovered manifests

**Preview and Automation Flags**:
- `--dry-run` - Preview changes without writing files (requires --cascade)
- `--quiet` / `-q` - Suppress output (only show errors), useful for scripts

**Verification and Status**:
- `versioneer show` - Show current version
- `versioneer verify` - Verify all versions are synchronized
- `versioneer status` - Show version and build system status
- `versioneer tag` - Create git tag for current version

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
- **Cascade mode** for monorepos with multiple projects
- Recursive manifest discovery with .gitignore support
- Dry-run preview for safe operations
- Quiet mode for scripting and automation
- Git tag creation with customizable formats
- Version synchronization verification
- Supports Cargo (Rust), Python (pyproject.toml), and Node.js (package.json)

### Tool Behavior
- **Exit Codes**: 0 (success), 1 (error)
- **Subcommands**: Uses clap derive pattern with subcommands
- **TTY Detection**: Colorful output for terminals, plain text for pipes
- **Git Integration**: Optional git tagging with customizable formats

### Cascade Mode

Cascade mode enables version management across monorepos with multiple projects:

**Structure Requirements**:
- **One VERSION file** at the root (source of truth)
- **Multiple manifest files** in subdirectories (Cargo.toml, pyproject.toml, package.json)
- **No nested VERSION files** allowed (would create ambiguity)

**How It Works**:
1. Recursively discovers all manifest files in the directory tree
2. Respects .gitignore patterns (requires .git directory)
3. Updates all discovered manifests atomically
4. Full rollback on any error (in-memory staging)

**Safety Features**:
- Rejects nested VERSION files (only one at root allowed)
- Rejects symlinks (prevents confusion)
- Atomic operations with full rollback
- Dry-run mode for safe previews

**Example Usage**:
```bash
# Preview changes
versioneer patch --cascade --dry-run

# Apply changes
versioneer patch --cascade

# Quiet mode for scripts
versioneer patch --cascade --quiet
```

**When to Use Cascade**:
- Monorepo with multiple projects sharing a version
- Keeping subproject manifests synchronized with root VERSION
- Batch version updates across many manifest files

**When NOT to Use Cascade**:
- Multi-project monorepo with independent versions (run versioneer from each project directory)
- Single project (use standard mode without --cascade)

## Version Management Workflow

1. Edit code and commit changes
2. Run `versioneer patch|minor|major` to bump version
3. Optionally run `versioneer tag` to create git tag
4. Push commits and tags to trigger releases

For automated releases, use `./scripts/release.sh` which handles the complete workflow with quality gates.
