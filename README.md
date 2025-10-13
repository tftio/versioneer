# Versioneer

A tool to synchronize VERSION files with build system version declarations, supporting Cargo.toml (Rust), pyproject.toml (Python), and package.json (Node.js/TypeScript) projects.

## Features

- Semantic versioning support with `major`, `minor`, and `patch` bumps
- Automatic synchronization between VERSION file and build system files
- Supports Cargo.toml, pyproject.toml, and package.json
- Version mismatch detection with helpful error messages
- Git tagging with customizable tag formats
- Cross-platform compatibility

## Installation

### Quick Install (Recommended)

Install the latest release directly from GitHub:

```bash
curl -fsSL https://raw.githubusercontent.com/workhelix/versioneer/main/install.sh | sh
```

Or with a custom install directory:

```bash
INSTALL_DIR=/usr/local/bin curl -fsSL https://raw.githubusercontent.com/workhelix/versioneer/main/install.sh | sh
```

The install script will:
- Auto-detect your OS and architecture
- Download the latest release
- Verify checksums (when available)
- Install to `$HOME/.local/bin` by default
- Prompt before replacing existing installations
- Guide you on adding the directory to your PATH

### Alternative Install Methods

**From Source (requires Rust toolchain):**

```bash
git clone https://github.com/workhelix/versioneer.git
cd versioneer
cargo build --release
install -m 0755 target/release/versioneer ~/.local/bin/
```

**From Releases:**

1. Visit [Releases](https://github.com/workhelix/versioneer/releases)
2. Download the appropriate `versioneer-{target}.zip` for your platform
3. Extract and copy the binary to a directory in your PATH

**Using GitHub CLI:**

```bash
# Create the local bin directory if it doesn't exist
mkdir -p ~/.local/bin

# Download and extract the binary for your platform (latest version)
# macOS Apple Silicon (ARM64)
gh release download --repo workhelix/versioneer --pattern "versioneer-aarch64-apple-darwin.zip" -O - | funzip > ~/.local/bin/versioneer

# macOS Intel (x64)
gh release download --repo workhelix/versioneer --pattern "versioneer-x86_64-apple-darwin.zip" -O - | funzip > ~/.local/bin/versioneer

# Linux x64
gh release download --repo workhelix/versioneer --pattern "versioneer-x86_64-unknown-linux-gnu.zip" -O - | funzip > ~/.local/bin/versioneer

# Linux ARM64
gh release download --repo workhelix/versioneer --pattern "versioneer-aarch64-unknown-linux-gnu.zip" -O - | funzip > ~/.local/bin/versioneer

chmod +x ~/.local/bin/versioneer
```

### Supported Platforms

- **Linux**: x86_64, aarch64
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **Windows**: x86_64

### PATH Setup

Make sure `~/.local/bin` is in your `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Add this to your shell configuration file (`.bashrc`, `.zshrc`, etc.) to make it permanent.

## Usage

### Basic Commands

Show current status (default when run with no arguments):
```bash
versioneer
```

Bump version:
```bash
versioneer patch   # 1.2.3 -> 1.2.4
versioneer minor   # 1.2.3 -> 1.3.0
versioneer major   # 1.2.3 -> 2.0.0
```

Show current version:
```bash
versioneer show
```

Synchronize all version files to match VERSION file:
```bash
versioneer sync
```

Show detailed status:
```bash
versioneer status
```

Verify all version files are synchronized:
```bash
versioneer verify
```

### Workflow

1. **Initialize your project** with a VERSION file containing your starting version (e.g., `1.0.0`)
2. **Ensure you have** at least one of: `Cargo.toml`, `pyproject.toml`, or `package.json` with a version field
3. **Run `versioneer sync`** to synchronize all files to the VERSION file content
4. **Use version bump commands** (`major`, `minor`, `patch`) to increment versions
5. **All files are updated automatically** and kept in sync
6. **Use your build system's release tools** to create git tags and publish releases

### Version Validation

Before performing version bumps, versioneer verifies that all version files are synchronized. If versions differ between files, the tool will:

1. Display a clear error message showing which files have mismatched versions
2. Suggest running `versioneer sync` to resolve the mismatch
3. Exit with an error code to prevent accidental version bumps

This ensures that your version files never get out of sync accidentally.

## Automated Release Management

**Enterprise Release Process** - Versioneer integrates with automated release workflows to prevent version synchronization issues:

### Multi-Layer Validation System

Versioneer is designed to work with a comprehensive validation system:

1. **Layer 1: Version Synchronization** - Ensures VERSION file matches all build system files
2. **Layer 2: Git Hooks Validation** - Pre-push hooks verify version synchronization
3. **Layer 3: GitHub Actions Validation** - CI validates versions before building releases
4. **Layer 4: Quality Gates** - Tests, lints, audits before every release

### Integration with Release Scripts

For projects using automated release management, integrate versioneer with your release scripts:

```bash
# Example automated release workflow
versioneer patch             # Bump version across all files
git add VERSION Cargo.toml   # Stage version changes
git commit -m "bump version" # Commit version bump
git tag v$(cat VERSION)      # Create git tag
git push && git push --tags  # Publish to trigger CI/CD

# Or use task runners (e.g., just, make)
just release patch           # Automated workflow with quality gates
```

### Preventing Common Release Problems

Versioneer solves these critical issues:
- **Version mismatches**: Ensures VERSION, Cargo.toml, pyproject.toml, and package.json always match
- **Build failures**: Pre-bump verification prevents partial updates
- **Install failures**: Consistent versions across all build systems
- **Process errors**: Atomic updates prevent human mistakes in manual version management

### Release Workflow Integration

When integrated with automated release workflows, versioneer ensures:
- ✅ **Atomic operations**: All version files updated together or none at all
- ✅ **Validation gates**: Pre-bump verification prevents inconsistent versions
- ✅ **Quality enforcement**: Integration with CI/CD quality gates
- ✅ **Binary verification**: Built binaries report expected versions
- ✅ **Rollback safety**: Failed bumps don't leave repository in inconsistent state

## Supported File Formats

### VERSION File

A simple text file containing the semantic version:
```
1.2.3
```

### Cargo.toml

Rust project configuration with version in the `[package]` section:
```toml
[package]
name = "my-project"
version = "1.2.3"
edition = "2021"
```

### pyproject.toml

Python project configuration with version in the `[project]` section:
```toml
[project]
name = "my-project"
version = "1.2.3"
description = "My project"
```

### package.json

Node.js/TypeScript project configuration with version as a top-level field:
```json
{
  "name": "my-project",
  "version": "1.2.3",
  "description": "My project",
  "main": "index.js"
}
```

## Requirements

- A VERSION file in the project root
- At least one supported build system file (Cargo.toml, pyproject.toml, or package.json)
- Valid semantic version format (MAJOR.MINOR.PATCH)

## Error Handling

The tool provides clear error messages for common issues:
- Missing VERSION file
- Missing build system files
- Invalid version formats
- Version mismatches between files
- File read/write permissions

## License

This project is released under the MIT License. See [LICENSE](LICENSE) for details.