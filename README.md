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

Bump version with git tagging:
```bash
versioneer patch --tag                    # Bump and create tag: {repository_name}-v1.2.4
versioneer minor --tag --tag-format "v{version}"   # Bump and create tag: v1.3.0
versioneer major --tag --tag-format "release-{major}.{minor}.{patch}"  # Custom format
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

Create git tag for current version:
```bash
versioneer tag                             # Create tag: {repository_name}-v{current_version}
versioneer tag --tag-format "v{version}"   # Create tag: v{current_version}
versioneer tag --tag-format "{major}.{minor}.{patch}-release"  # Create tag: 1.2.3-release
```

### Workflow

1. **Initialize your project** with a VERSION file containing your starting version (e.g., `1.0.0`)
2. **Ensure you have** at least one of: `Cargo.toml`, `pyproject.toml`, or `package.json` with a version field
3. **Run `versioneer sync`** to synchronize all files to the VERSION file content
4. **Use version bump commands** (`major`, `minor`, `patch`) to increment versions
5. **All files are updated automatically** and kept in sync

### Version Validation

Before performing version bumps, versioneer verifies that all version files are synchronized. If versions differ between files, the tool will:

1. Display a clear error message showing which files have mismatched versions
2. Suggest running `versioneer sync` to resolve the mismatch
3. Exit with an error code to prevent accidental version bumps

This ensures that your version files never get out of sync accidentally.

## Git Tagging

Versioneer can automatically create git tags when bumping versions, making it easy to track releases in your repository.

### Tag Format Placeholders

You can customize the tag format using these placeholders:

- `{repository_name}` - Name of the repository (from git remote or directory name)
- `{version}` - Full semantic version (e.g., "1.2.3")
- `{major}` - Major version number
- `{minor}` - Minor version number
- `{patch}` - Patch version number

### Default Tag Format

If no custom format is specified, tags use the format: `{repository_name}-v{version}`

Examples:
- `versioneer-v1.2.3`
- `my-project-v2.0.0`

### Custom Tag Formats

```bash
# Simple version tag
versioneer patch --tag --tag-format "v{version}"
# Result: v1.2.4

# Release format with individual components
versioneer minor --tag --tag-format "release-{major}.{minor}.{patch}"
# Result: release-1.3.0

# Project-specific format
versioneer major --tag --tag-format "{repository_name}-release-{major}.{minor}"
# Result: my-project-release-2.0
```

### Standalone Tagging

Create a tag for the current version without bumping:

```bash
versioneer tag                           # Use default format
versioneer tag --tag-format "v{version}" # Use custom format
```

## Automated Release Management

**Enterprise Release Process** - Versioneer integrates with automated release workflows to prevent version/tag synchronization issues:

### Multi-Layer Validation System

Versioneer is designed to work with a comprehensive validation system:

1. **Layer 1: Git Hooks Validation** - Pre-push hooks verify version synchronization
2. **Layer 2: GitHub Actions Validation** - CI validates tag versions match Cargo.toml
3. **Layer 3: Automated Release Script** - Complete automated release workflow
4. **Layer 4: Quality Gates** - Tests, lints, audits before every release

### Integration with Release Scripts

For projects using automated release management, use the provided release script instead of individual versioneer commands:

```bash
# Automated release (recommended for production projects)
./scripts/release.sh patch   # Runs versioneer patch + quality checks + git operations
./scripts/release.sh minor   # Runs versioneer minor + quality checks + git operations
./scripts/release.sh major   # Runs versioneer major + quality checks + git operations

# Individual versioneer commands (for manual workflows)
versioneer patch             # Just version bumping
versioneer sync              # Just synchronization
versioneer verify            # Just verification
versioneer tag               # Just git tagging
```

### Preventing Common Release Problems

Versioneer solves these critical issues:
- **Version mismatches**: GitHub releases with tag `v1.0.9` containing binary version `1.0.8`
- **Install failures**: Checksum verification failures due to incorrect URL construction
- **Tag inconsistencies**: Manual git tags not matching actual code versions
- **Process errors**: Human mistakes in manual version management

### Release Workflow Integration

When integrated with automated release workflows, versioneer ensures:
- ✅ **Atomic operations**: Version bumps and git tags created together
- ✅ **Validation gates**: Pre-push hooks prevent inconsistent versions
- ✅ **Quality enforcement**: Tests, lints, audits run before every release
- ✅ **Binary verification**: Built binaries report expected versions
- ✅ **Rollback safety**: Failed releases don't leave repository in inconsistent state

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

This project is released under the CC0 1.0 Universal (CC0 1.0) Public Domain Dedication. See [LICENSE](LICENSE) for details.