//! Versioneer - A tool to synchronize VERSION files with build system version declarations
//!
//! This library provides functionality to read, parse, and update version information
//! across different file formats including VERSION files, Cargo.toml, and pyproject.toml.

pub mod output;

use anyhow::{Context, Result};
use semver::Version;
use std::fs;
use std::path::Path;

/// Represents different types of build system files that can contain version information
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildSystem {
    /// Cargo.toml file for Rust projects
    Cargo,
    /// pyproject.toml file for Python projects
    PyProject,
    /// package.json file for Node.js/TypeScript projects
    PackageJson,
}

/// Represents a version bump type following semantic versioning
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BumpType {
    /// Increment major version, reset minor and patch to 0
    Major,
    /// Increment minor version, reset patch to 0
    Minor,
    /// Increment patch version
    Patch,
}

/// Result of a dry-run operation showing what would change
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DryRunResult {
    /// The new version that would be applied
    pub new_version: Version,
    /// List of files that would be updated
    pub files_to_update: Vec<std::path::PathBuf>,
}

/// Default version filename
pub const DEFAULT_VERSION_FILE: &str = "VERSION";

/// Core version management functionality
pub struct VersionManager {
    /// The current working directory path
    pub base_path: std::path::PathBuf,
    /// The version filename (e.g. "VERSION" or "version.txt")
    pub version_file: String,
}

impl VersionManager {
    /// Create a new `VersionManager` for the given directory with the default version filename
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            version_file: DEFAULT_VERSION_FILE.to_string(),
        }
    }

    /// Create a new `VersionManager` with a custom version filename
    pub fn with_version_file<P: AsRef<Path>>(base_path: P, version_file: &str) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            version_file: version_file.to_string(),
        }
    }

    /// Read the current version from the VERSION file
    ///
    /// # Errors
    ///
    /// Returns an error if the VERSION file cannot be read or contains an invalid version format.
    pub fn read_version_file(&self) -> Result<Version> {
        let version_path = self.base_path.join(&self.version_file);
        let content = fs::read_to_string(&version_path).with_context(|| {
            format!("Failed to read VERSION file at {}", version_path.display())
        })?;

        // Strip inline comments (e.g. "1.2.3 # x-release-please-version")
        let version_str = content.trim().split('#').next().unwrap_or("").trim();
        Version::parse(version_str)
            .with_context(|| format!("Invalid version format in VERSION file: {version_str}"))
    }

    /// Write a version to the VERSION file
    ///
    /// # Errors
    ///
    /// Returns an error if the VERSION file cannot be written to.
    pub fn write_version_file(&self, version: &Version) -> Result<()> {
        let version_path = self.base_path.join(&self.version_file);

        // Preserve inline comments (e.g. "# x-release-please-version")
        let content = if version_path.exists() {
            let existing = fs::read_to_string(&version_path).unwrap_or_default();
            let trimmed = existing.trim();
            trimmed.find('#').map_or_else(
                || format!("{version}\n"),
                |hash_pos| {
                    let comment = trimmed[hash_pos..].trim();
                    format!("{version} {comment}\n")
                },
            )
        } else {
            format!("{version}\n")
        };

        fs::write(&version_path, content)
            .with_context(|| format!("Failed to write VERSION file at {}", version_path.display()))
    }

    /// Detect which build system files are present
    #[must_use]
    pub fn detect_build_systems(&self) -> Vec<BuildSystem> {
        let mut systems = Vec::new();

        if self.base_path.join("Cargo.toml").exists() {
            systems.push(BuildSystem::Cargo);
        }

        if self.base_path.join("pyproject.toml").exists() {
            systems.push(BuildSystem::PyProject);
        }

        if self.base_path.join("package.json").exists() {
            systems.push(BuildSystem::PackageJson);
        }

        systems
    }

    /// Read version from a specific build system file
    ///
    /// # Errors
    ///
    /// Returns an error if the build system file cannot be read or parsed.
    pub fn read_build_system_version(&self, system: &BuildSystem) -> Result<Version> {
        match system {
            BuildSystem::Cargo => self.read_cargo_version(),
            BuildSystem::PyProject => self.read_pyproject_version(),
            BuildSystem::PackageJson => self.read_package_json_version(),
        }
    }

    /// Update version in a specific build system file
    ///
    /// # Errors
    ///
    /// Returns an error if the build system file cannot be read, parsed, or written.
    pub fn update_build_system_version(
        &self,
        system: &BuildSystem,
        version: &Version,
    ) -> Result<()> {
        match system {
            BuildSystem::Cargo => self.update_cargo_version(version),
            BuildSystem::PyProject => self.update_pyproject_version(version),
            BuildSystem::PackageJson => self.update_package_json_version(version),
        }
    }

    /// Bump version according to semantic versioning rules
    ///
    /// # Errors
    ///
    /// Returns an error if version files are not synchronized or cannot be updated.
    pub fn bump_version(&self, bump_type: BumpType) -> Result<()> {
        // Ensure all versions are in sync before bumping
        self.verify_versions_in_sync()?;

        let current_version = self.read_version_file()?;
        let new_version = match bump_type {
            BumpType::Major => Version::new(current_version.major + 1, 0, 0),
            BumpType::Minor => Version::new(current_version.major, current_version.minor + 1, 0),
            BumpType::Patch => Version::new(
                current_version.major,
                current_version.minor,
                current_version.patch + 1,
            ),
        };

        // Update VERSION file
        self.write_version_file(&new_version)?;

        // Update all detected build system files
        let build_systems = self.detect_build_systems();
        for system in &build_systems {
            self.update_build_system_version(system, &new_version)
                .with_context(|| format!("Failed to update {system:?} version"))?;
        }

        Ok(())
    }

    /// Reset the version to a specific version string
    ///
    /// # Errors
    ///
    /// Returns an error if the version string is invalid or if file operations fail.
    pub fn reset_version(&self, version_str: &str) -> Result<()> {
        // Parse the provided version string
        let new_version = Version::parse(version_str)
            .with_context(|| format!("Invalid semantic version format: '{version_str}'"))?;

        // Update VERSION file
        self.write_version_file(&new_version)?;

        // Update all detected build system files
        let build_systems = self.detect_build_systems();
        for system in &build_systems {
            self.update_build_system_version(system, &new_version)
                .with_context(|| format!("Failed to update {system:?} version"))?;
        }

        Ok(())
    }

    /// Verify that all version files are synchronized
    ///
    /// # Errors
    ///
    /// Returns an error if version files are not synchronized or cannot be read.
    pub fn verify_versions_in_sync(&self) -> Result<()> {
        let version_file_version = self.read_version_file()?;
        let build_systems = self.detect_build_systems();

        let mut mismatched = Vec::new();

        for system in &build_systems {
            match self.read_build_system_version(system) {
                Ok(system_version) => {
                    if system_version != version_file_version {
                        mismatched.push(format!(
                            "{system:?} has version {system_version} but VERSION file has {version_file_version}"
                        ));
                    }
                }
                Err(e) => {
                    mismatched.push(format!("Failed to read {system:?} version: {e}"));
                }
            }
        }

        if !mismatched.is_empty() {
            anyhow::bail!(
                "Version files are not synchronized:\n{}\n\nRun 'versioneer sync' to synchronize all version files.",
                mismatched.join("\n")
            );
        }

        Ok(())
    }

    /// Synchronize all version files to match the VERSION file
    ///
    /// # Errors
    ///
    /// Returns an error if version files cannot be read or updated.
    pub fn sync_versions(&self) -> Result<()> {
        let version = self.read_version_file()?;
        let build_systems = self.detect_build_systems();

        for system in &build_systems {
            self.update_build_system_version(system, &version)
                .with_context(|| format!("Failed to sync {system:?} version"))?;
        }

        Ok(())
    }

    /// Bump version with cascade dry-run (preview what would change)
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails or version reading fails.
    pub fn bump_cascade_dry_run(&self, bump_type: BumpType) -> Result<DryRunResult> {
        // Step 1: Discover all manifests
        let manifests = self.discover_manifests()?;

        // Step 2: Read current version and calculate new version
        let current_version = self.read_version_file()?;
        let new_version = match bump_type {
            BumpType::Major => Version::new(current_version.major + 1, 0, 0),
            BumpType::Minor => Version::new(current_version.major, current_version.minor + 1, 0),
            BumpType::Patch => Version::new(
                current_version.major,
                current_version.minor,
                current_version.patch + 1,
            ),
        };

        // Step 3: Collect all files that would be updated
        let mut files_to_update = vec![self.base_path.join(&self.version_file)];
        for (path, _) in manifests {
            files_to_update.push(path);
        }

        Ok(DryRunResult {
            new_version,
            files_to_update,
        })
    }

    /// Bump version with cascade (update all discovered manifests)
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails, versions are out of sync, or updates fail.
    /// On error, all changes are rolled back.
    pub fn bump_cascade(&self, bump_type: BumpType) -> Result<()> {
        use std::collections::HashMap;

        // Step 1: Discover all manifests
        let manifests = self.discover_manifests()?;

        // Step 2: Read current version and calculate new version
        let current_version = self.read_version_file()?;
        let new_version = match bump_type {
            BumpType::Major => Version::new(current_version.major + 1, 0, 0),
            BumpType::Minor => Version::new(current_version.major, current_version.minor + 1, 0),
            BumpType::Patch => Version::new(
                current_version.major,
                current_version.minor,
                current_version.patch + 1,
            ),
        };

        // Step 3: Read all files into memory for potential rollback
        let mut original_contents: HashMap<std::path::PathBuf, String> = HashMap::new();

        // Read version file
        let version_path = self.base_path.join(&self.version_file);
        original_contents.insert(version_path.clone(), fs::read_to_string(&version_path)?);

        // Read all manifests
        for (path, _) in &manifests {
            original_contents.insert(path.clone(), fs::read_to_string(path)?);
        }

        // Step 4: Perform updates with rollback on error
        let update_result = (|| -> Result<()> {
            // Update VERSION file
            self.write_version_file(&new_version)?;

            // Update all manifests
            for (path, system) in &manifests {
                // Create a temporary VersionManager for this manifest's directory
                let manifest_dir = path.parent().context("Manifest has no parent directory")?;
                let temp_manager = Self::new(manifest_dir);
                temp_manager
                    .update_build_system_version(system, &new_version)
                    .with_context(|| {
                        format!("Failed to update {:?} at {}", system, path.display())
                    })?;
            }

            Ok(())
        })();

        // Step 5: Rollback on error
        if let Err(e) = update_result {
            // Restore all original contents
            for (path, content) in original_contents {
                let _ = fs::write(&path, content); // Best effort rollback
            }
            return Err(e);
        }

        Ok(())
    }

    /// Preview sync operation with cascade (dry-run mode)
    ///
    /// # Errors
    ///
    /// Returns an error if discovery or version reading fails.
    pub fn sync_cascade_dry_run(&self) -> Result<DryRunResult> {
        // Step 1: Discover all manifests
        let manifests = self.discover_manifests()?;

        // Step 2: Read current version
        let version = self.read_version_file()?;

        // Step 3: Collect all files that would be updated
        let mut files_to_update = Vec::new();
        for (path, _) in manifests {
            files_to_update.push(path);
        }

        Ok(DryRunResult {
            new_version: version,
            files_to_update,
        })
    }

    /// Sync all manifests with cascade (update all to match VERSION file)
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails or updates fail.
    /// On error, all changes are rolled back.
    pub fn sync_cascade(&self) -> Result<()> {
        use std::collections::HashMap;

        // Step 1: Discover all manifests
        let manifests = self.discover_manifests()?;

        // Step 2: Read current version
        let version = self.read_version_file()?;

        // Step 3: Read all manifest files into memory for potential rollback
        let mut original_contents: HashMap<std::path::PathBuf, String> = HashMap::new();

        for (path, _) in &manifests {
            original_contents.insert(path.clone(), fs::read_to_string(path)?);
        }

        // Step 4: Perform updates with rollback on error
        let update_result = (|| -> Result<()> {
            for (path, system) in &manifests {
                let manifest_dir = path.parent().context("Manifest has no parent directory")?;
                let temp_manager = Self::new(manifest_dir);
                temp_manager
                    .update_build_system_version(system, &version)
                    .with_context(|| {
                        format!("Failed to sync {:?} at {}", system, path.display())
                    })?;
            }
            Ok(())
        })();

        // Step 5: Rollback on error
        if let Err(e) = update_result {
            for (path, content) in original_contents {
                let _ = fs::write(&path, content);
            }
            return Err(e);
        }

        Ok(())
    }

    /// Preview reset operation with cascade (dry-run mode)
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails or version is invalid.
    pub fn reset_cascade_dry_run(&self, version_str: &str) -> Result<DryRunResult> {
        // Step 1: Parse and validate version
        let new_version = Version::parse(version_str)
            .with_context(|| format!("Invalid semantic version format: '{version_str}'"))?;

        // Step 2: Discover all manifests
        let manifests = self.discover_manifests()?;

        // Step 3: Collect all files that would be updated
        let mut files_to_update = vec![self.base_path.join(&self.version_file)];
        for (path, _) in manifests {
            files_to_update.push(path);
        }

        Ok(DryRunResult {
            new_version,
            files_to_update,
        })
    }

    /// Reset version with cascade (reset all discovered manifests)
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails, version is invalid, or updates fail.
    /// On error, all changes are rolled back.
    pub fn reset_cascade(&self, version_str: &str) -> Result<()> {
        use std::collections::HashMap;

        // Step 1: Parse and validate version
        let new_version = Version::parse(version_str)
            .with_context(|| format!("Invalid semantic version format: '{version_str}'"))?;

        // Step 2: Discover all manifests
        let manifests = self.discover_manifests()?;

        // Step 3: Read all files into memory for potential rollback
        let mut original_contents: HashMap<std::path::PathBuf, String> = HashMap::new();

        let version_path = self.base_path.join(&self.version_file);
        original_contents.insert(version_path.clone(), fs::read_to_string(&version_path)?);

        for (path, _) in &manifests {
            original_contents.insert(path.clone(), fs::read_to_string(path)?);
        }

        // Step 4: Perform updates with rollback on error
        let update_result = (|| -> Result<()> {
            self.write_version_file(&new_version)?;

            for (path, system) in &manifests {
                let manifest_dir = path.parent().context("Manifest has no parent directory")?;
                let temp_manager = Self::new(manifest_dir);
                temp_manager
                    .update_build_system_version(system, &new_version)
                    .with_context(|| {
                        format!("Failed to reset {:?} at {}", system, path.display())
                    })?;
            }
            Ok(())
        })();

        // Step 5: Rollback on error
        if let Err(e) = update_result {
            for (path, content) in original_contents {
                let _ = fs::write(&path, content);
            }
            return Err(e);
        }

        Ok(())
    }

    /// Discover all manifest files recursively in subdirectories
    ///
    /// Respects .gitignore patterns. Errors if nested VERSION files are found.
    ///
    /// # Errors
    ///
    /// Returns an error if directory traversal fails or nested VERSION files are found.
    pub fn discover_manifests(&self) -> Result<Vec<(std::path::PathBuf, BuildSystem)>> {
        use ignore::WalkBuilder;

        let mut manifests = Vec::new();

        // Use ignore crate to respect .gitignore
        let walker = WalkBuilder::new(&self.base_path)
            .hidden(false)
            .git_ignore(true)
            .build();

        for entry in walker {
            let entry = entry?;
            let path = entry.path();

            // Check for symlinks (error condition)
            let metadata = fs::symlink_metadata(path)?;
            if metadata.is_symlink() {
                anyhow::bail!(
                    "Symlink found at {}. Symlinks are not supported in cascade mode.",
                    path.display()
                );
            }

            if path.is_file() {
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy();

                    // Check for nested version files
                    if filename_str == self.version_file {
                        // Version file in base_path is OK, but not in subdirectories
                        if path.parent() != Some(&self.base_path) {
                            anyhow::bail!(
                                "Nested {} file found at {}. Only one {} file is allowed at the root directory.",
                                self.version_file,
                                path.display(),
                                self.version_file,
                            );
                        }
                    } else if filename_str == "Cargo.toml" {
                        manifests.push((path.to_path_buf(), BuildSystem::Cargo));
                    } else if filename_str == "pyproject.toml" {
                        manifests.push((path.to_path_buf(), BuildSystem::PyProject));
                    } else if filename_str == "package.json" {
                        manifests.push((path.to_path_buf(), BuildSystem::PackageJson));
                    }
                }
            }
        }

        Ok(manifests)
    }

    /// Count existing RC tags for a given base version.
    ///
    /// Queries git for tags matching `v{major}.{minor}.{patch}-rc.*`
    /// and returns the highest RC number found, or 0 if none exist.
    fn count_rc_tags(&self, base_version: &Version) -> Result<u64> {
        let pattern = format!(
            "v{}.{}.{}-rc.*",
            base_version.major, base_version.minor, base_version.patch
        );

        let output = std::process::Command::new("git")
            .args(["tag", "-l", &pattern])
            .current_dir(&self.base_path)
            .output()
            .context("Failed to run git. Is git installed and is this a git repository?")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git tag failed: {stderr}");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut max_rc: u64 = 0;

        for line in stdout.lines() {
            let line = line.trim();
            if let Some(rc_part) = line.rsplit("-rc.").next() {
                if let Ok(n) = rc_part.parse::<u64>() {
                    max_rc = max_rc.max(n);
                }
            }
        }

        Ok(max_rc)
    }

    /// Compute the next RC version from the VERSION file and git tags.
    ///
    /// Reads VERSION (must be a clean M.m.p without pre-release suffix),
    /// counts existing RC tags, and returns the next RC version.
    ///
    /// # Errors
    ///
    /// Returns an error if the VERSION file cannot be read, contains a pre-release
    /// suffix, or if git tag querying fails.
    pub fn next_rc_version(&self) -> Result<Version> {
        let base_version = self.read_version_file()?;

        if !base_version.pre.is_empty() {
            anyhow::bail!(
                "VERSION must be a clean M.m.p version without pre-release suffix, got: {base_version}"
            );
        }

        let max_rc = self.count_rc_tags(&base_version)?;
        let mut rc_version = base_version;
        rc_version.pre = semver::Prerelease::new(&format!("rc.{}", max_rc + 1))
            .context("Failed to construct pre-release version")?;

        Ok(rc_version)
    }

    /// Read version from Cargo.toml
    fn read_cargo_version(&self) -> Result<Version> {
        let cargo_path = self.base_path.join("Cargo.toml");
        let content = fs::read_to_string(&cargo_path)
            .with_context(|| format!("Failed to read Cargo.toml at {}", cargo_path.display()))?;

        let cargo_toml: toml::Value =
            toml::from_str(&content).with_context(|| "Failed to parse Cargo.toml")?;

        let version_str = cargo_toml
            .get("package")
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
            .context("No version found in Cargo.toml [package] section")?;

        Version::parse(version_str)
            .with_context(|| format!("Invalid version format in Cargo.toml: {version_str}"))
    }

    /// Update version in Cargo.toml
    fn update_cargo_version(&self, version: &Version) -> Result<()> {
        let cargo_path = self.base_path.join("Cargo.toml");
        let content = fs::read_to_string(&cargo_path)
            .with_context(|| format!("Failed to read Cargo.toml at {}", cargo_path.display()))?;

        let updated_content = Self::update_toml_version(&content, version, "package")?;

        fs::write(&cargo_path, updated_content)
            .with_context(|| format!("Failed to write Cargo.toml at {}", cargo_path.display()))
    }

    /// Read version from pyproject.toml
    fn read_pyproject_version(&self) -> Result<Version> {
        let pyproject_path = self.base_path.join("pyproject.toml");
        let content = fs::read_to_string(&pyproject_path).with_context(|| {
            format!(
                "Failed to read pyproject.toml at {}",
                pyproject_path.display()
            )
        })?;

        let pyproject_toml: toml::Value =
            toml::from_str(&content).with_context(|| "Failed to parse pyproject.toml")?;

        let version_str = pyproject_toml
            .get("project")
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
            .context("No version found in pyproject.toml [project] section")?;

        Version::parse(version_str)
            .with_context(|| format!("Invalid version format in pyproject.toml: {version_str}"))
    }

    /// Update version in pyproject.toml
    fn update_pyproject_version(&self, version: &Version) -> Result<()> {
        let pyproject_path = self.base_path.join("pyproject.toml");
        let content = fs::read_to_string(&pyproject_path).with_context(|| {
            format!(
                "Failed to read pyproject.toml at {}",
                pyproject_path.display()
            )
        })?;

        let updated_content = Self::update_toml_version(&content, version, "project")?;

        fs::write(&pyproject_path, updated_content).with_context(|| {
            format!(
                "Failed to write pyproject.toml at {}",
                pyproject_path.display()
            )
        })
    }

    /// Read version from package.json
    fn read_package_json_version(&self) -> Result<Version> {
        let package_json_path = self.base_path.join("package.json");
        let content = fs::read_to_string(&package_json_path).with_context(|| {
            format!(
                "Failed to read package.json at {}",
                package_json_path.display()
            )
        })?;

        let json: serde_json::Value =
            serde_json::from_str(&content).with_context(|| "Failed to parse package.json")?;

        let version_str = json
            .get("version")
            .and_then(|v| v.as_str())
            .context("No version found in package.json")?;

        Version::parse(version_str)
            .with_context(|| format!("Invalid version format in package.json: {version_str}"))
    }

    /// Update version in package.json
    fn update_package_json_version(&self, version: &Version) -> Result<()> {
        let package_json_path = self.base_path.join("package.json");
        let content = fs::read_to_string(&package_json_path).with_context(|| {
            format!(
                "Failed to read package.json at {}",
                package_json_path.display()
            )
        })?;

        let mut json: serde_json::Value =
            serde_json::from_str(&content).with_context(|| "Failed to parse package.json")?;

        // Update the version field
        if let Some(obj) = json.as_object_mut() {
            obj.insert(
                "version".to_string(),
                serde_json::Value::String(version.to_string()),
            );
        } else {
            anyhow::bail!("package.json root is not a JSON object");
        }

        // Serialize with pretty printing (2-space indent, standard for Node.js)
        let updated_content = serde_json::to_string_pretty(&json)
            .with_context(|| "Failed to serialize package.json")?;

        // Add trailing newline (Node.js convention)
        let updated_content = format!("{updated_content}\n");

        fs::write(&package_json_path, updated_content).with_context(|| {
            format!(
                "Failed to write package.json at {}",
                package_json_path.display()
            )
        })
    }

    /// Helper to update version in TOML content
    fn update_toml_version(content: &str, version: &Version, section: &str) -> Result<String> {
        use regex::Regex;

        // More flexible regex that handles multiline TOML sections with better whitespace handling
        let pattern = format!(r#"(?s)(\[{section}\][^\[]*?version\s*=\s*")[^"]*(")"#);
        let re = Regex::new(&pattern).context("Failed to create regex for version replacement")?;

        // Check if the regex can find a match at all
        if !re.is_match(content) {
            anyhow::bail!("No version field found in [{section}] section");
        }

        let result = re.replace(content, format!("${{1}}{version}${{2}}"));

        Ok(result.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_files(dir: &Path, version: &str) -> Result<()> {
        // Create VERSION file
        fs::write(dir.join("VERSION"), version)?;

        // Create Cargo.toml
        let cargo_content = format!(
            r#"[package]
name = "test"
version = "{version}"
edition = "2021"

[dependencies]
"#
        );
        fs::write(dir.join("Cargo.toml"), cargo_content)?;

        // Create pyproject.toml
        let pyproject_content = format!(
            r#"[project]
name = "test"
version = "{version}"
description = "Test project"

[build-system]
requires = ["setuptools"]
build-backend = "setuptools.build_meta"
"#
        );
        fs::write(dir.join("pyproject.toml"), pyproject_content)?;

        Ok(())
    }

    #[test]
    fn test_read_version_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_files(temp_dir.path(), "1.2.3")?;

        let manager = VersionManager::new(temp_dir.path());
        let version = manager.read_version_file()?;

        assert_eq!(version, Version::new(1, 2, 3));
        Ok(())
    }

    #[test]
    fn test_detect_build_systems() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_files(temp_dir.path(), "1.0.0")?;

        let manager = VersionManager::new(temp_dir.path());
        let systems = manager.detect_build_systems();

        assert!(systems.contains(&BuildSystem::Cargo));
        assert!(systems.contains(&BuildSystem::PyProject));
        Ok(())
    }

    #[test]
    fn test_bump_major() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_files(temp_dir.path(), "1.2.3")?;

        let manager = VersionManager::new(temp_dir.path());
        manager.bump_version(BumpType::Major)?;

        let version = manager.read_version_file()?;
        assert_eq!(version, Version::new(2, 0, 0));

        let cargo_version = manager.read_cargo_version()?;
        assert_eq!(cargo_version, Version::new(2, 0, 0));

        let pyproject_version = manager.read_pyproject_version()?;
        assert_eq!(pyproject_version, Version::new(2, 0, 0));

        Ok(())
    }

    #[test]
    fn test_bump_minor() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_files(temp_dir.path(), "1.2.3")?;

        let manager = VersionManager::new(temp_dir.path());
        manager.bump_version(BumpType::Minor)?;

        let version = manager.read_version_file()?;
        assert_eq!(version, Version::new(1, 3, 0));
        Ok(())
    }

    #[test]
    fn test_bump_patch() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_files(temp_dir.path(), "1.2.3")?;

        let manager = VersionManager::new(temp_dir.path());
        manager.bump_version(BumpType::Patch)?;

        let version = manager.read_version_file()?;
        assert_eq!(version, Version::new(1, 2, 4));
        Ok(())
    }

    #[test]
    fn test_reset_version_to_default() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_files(temp_dir.path(), "1.2.3")?;

        let manager = VersionManager::new(temp_dir.path());
        manager.reset_version("0.0.0")?;

        let version = manager.read_version_file()?;
        assert_eq!(version, Version::new(0, 0, 0));

        let cargo_version = manager.read_cargo_version()?;
        assert_eq!(cargo_version, Version::new(0, 0, 0));

        let pyproject_version = manager.read_pyproject_version()?;
        assert_eq!(pyproject_version, Version::new(0, 0, 0));

        Ok(())
    }

    #[test]
    fn test_reset_version_to_specific_version() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_files(temp_dir.path(), "1.2.3")?;

        let manager = VersionManager::new(temp_dir.path());
        manager.reset_version("3.5.7")?;

        let version = manager.read_version_file()?;
        assert_eq!(version, Version::new(3, 5, 7));

        let cargo_version = manager.read_cargo_version()?;
        assert_eq!(cargo_version, Version::new(3, 5, 7));

        let pyproject_version = manager.read_pyproject_version()?;
        assert_eq!(pyproject_version, Version::new(3, 5, 7));

        Ok(())
    }

    #[test]
    fn test_reset_version_with_prerelease() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_files(temp_dir.path(), "1.0.0")?;

        let manager = VersionManager::new(temp_dir.path());
        manager.reset_version("2.0.0-alpha.1")?;

        let version = manager.read_version_file()?;
        assert_eq!(version.major, 2);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
        assert_eq!(version.pre.as_str(), "alpha.1");

        Ok(())
    }

    #[test]
    fn test_reset_version_invalid_format() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_files(temp_dir.path(), "1.0.0")?;

        let manager = VersionManager::new(temp_dir.path());
        let result = manager.reset_version("invalid-version");

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid semantic version format")
        );

        // Verify original version is unchanged
        let version = manager.read_version_file()?;
        assert_eq!(version, Version::new(1, 0, 0));

        Ok(())
    }

    #[test]
    fn test_reset_version_empty_string() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_files(temp_dir.path(), "1.0.0")?;

        let manager = VersionManager::new(temp_dir.path());
        let result = manager.reset_version("");

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid semantic version format")
        );

        Ok(())
    }

    fn create_package_json(dir: &Path, version: &str, with_dependencies: bool) -> Result<()> {
        let package_json_content = if with_dependencies {
            format!(
                r#"{{
  "name": "test-package",
  "version": "{version}",
  "description": "A test package",
  "main": "index.js",
  "scripts": {{
    "test": "jest",
    "build": "tsc"
  }},
  "dependencies": {{
    "express": "^4.18.0"
  }},
  "devDependencies": {{
    "typescript": "^5.0.0"
  }}
}}
"#
            )
        } else {
            format!(
                r#"{{
  "name": "test-package",
  "version": "{version}"
}}
"#
            )
        };
        fs::write(dir.join("package.json"), package_json_content)?;
        Ok(())
    }

    #[test]
    fn test_detect_package_json() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("VERSION"), "1.0.0")?;
        create_package_json(temp_dir.path(), "1.0.0", false)?;

        let manager = VersionManager::new(temp_dir.path());
        let systems = manager.detect_build_systems();

        assert!(systems.contains(&BuildSystem::PackageJson));
        Ok(())
    }

    #[test]
    fn test_read_package_json_version() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_package_json(temp_dir.path(), "2.3.4", false)?;

        let manager = VersionManager::new(temp_dir.path());
        let version = manager.read_package_json_version()?;

        assert_eq!(version, Version::new(2, 3, 4));
        Ok(())
    }

    #[test]
    fn test_read_package_json_version_with_dependencies() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_package_json(temp_dir.path(), "1.5.0", true)?;

        let manager = VersionManager::new(temp_dir.path());
        let version = manager.read_package_json_version()?;

        assert_eq!(version, Version::new(1, 5, 0));
        Ok(())
    }

    #[test]
    fn test_update_package_json_version() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_package_json(temp_dir.path(), "1.0.0", false)?;

        let manager = VersionManager::new(temp_dir.path());
        let new_version = Version::new(2, 0, 0);
        manager.update_package_json_version(&new_version)?;

        let version = manager.read_package_json_version()?;
        assert_eq!(version, Version::new(2, 0, 0));
        Ok(())
    }

    #[test]
    fn test_update_package_json_preserves_other_fields() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_package_json(temp_dir.path(), "1.0.0", true)?;

        let manager = VersionManager::new(temp_dir.path());
        let new_version = Version::new(3, 2, 1);
        manager.update_package_json_version(&new_version)?;

        // Read the file and verify other fields are preserved
        let content = fs::read_to_string(temp_dir.path().join("package.json"))?;
        let json: serde_json::Value = serde_json::from_str(&content)?;

        assert_eq!(json["version"], "3.2.1");
        assert_eq!(json["name"], "test-package");
        assert_eq!(json["description"], "A test package");
        assert!(json["dependencies"].is_object());
        assert!(json["devDependencies"].is_object());
        Ok(())
    }

    #[test]
    fn test_bump_version_with_package_json() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("VERSION"), "1.2.3")?;
        create_package_json(temp_dir.path(), "1.2.3", true)?;

        let manager = VersionManager::new(temp_dir.path());
        manager.bump_version(BumpType::Minor)?;

        let version = manager.read_version_file()?;
        assert_eq!(version, Version::new(1, 3, 0));

        let package_json_version = manager.read_package_json_version()?;
        assert_eq!(package_json_version, Version::new(1, 3, 0));

        Ok(())
    }

    #[test]
    fn test_detect_all_build_systems() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_files(temp_dir.path(), "1.0.0")?;
        create_package_json(temp_dir.path(), "1.0.0", false)?;

        let manager = VersionManager::new(temp_dir.path());
        let systems = manager.detect_build_systems();

        assert_eq!(systems.len(), 3);
        assert!(systems.contains(&BuildSystem::Cargo));
        assert!(systems.contains(&BuildSystem::PyProject));
        assert!(systems.contains(&BuildSystem::PackageJson));
        Ok(())
    }

    #[test]
    fn test_sync_versions_with_package_json() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("VERSION"), "2.0.0")?;
        create_package_json(temp_dir.path(), "1.0.0", true)?;

        let manager = VersionManager::new(temp_dir.path());
        manager.sync_versions()?;

        let package_json_version = manager.read_package_json_version()?;
        assert_eq!(package_json_version, Version::new(2, 0, 0));
        Ok(())
    }

    #[test]
    fn test_verify_versions_with_package_json_mismatch() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("VERSION"), "2.0.0")?;
        create_package_json(temp_dir.path(), "1.0.0", false)?;

        let manager = VersionManager::new(temp_dir.path());
        let result = manager.verify_versions_in_sync();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Version files are not synchronized")
        );
        Ok(())
    }

    #[test]
    fn test_package_json_with_prerelease() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_package_json(temp_dir.path(), "1.0.0-beta.2", false)?;

        let manager = VersionManager::new(temp_dir.path());
        let version = manager.read_package_json_version()?;

        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
        assert_eq!(version.pre.as_str(), "beta.2");
        Ok(())
    }

    #[test]
    fn test_package_json_missing_version_field() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let package_json_content = r#"{"name": "test-package"}"#;
        fs::write(temp_dir.path().join("package.json"), package_json_content)?;

        let manager = VersionManager::new(temp_dir.path());
        let result = manager.read_package_json_version();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No version found in package.json")
        );
        Ok(())
    }

    #[test]
    fn test_pyproject_toml_missing_version_field() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let pyproject_content = r#"[project]
name = "test"
"#;
        fs::write(temp_dir.path().join("pyproject.toml"), pyproject_content)?;

        let manager = VersionManager::new(temp_dir.path());
        let result = manager.read_pyproject_version();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No version found in pyproject.toml")
        );
        Ok(())
    }

    #[test]
    fn test_cargo_toml_missing_version_field() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cargo_content = r#"[package]
name = "test"
"#;
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_content)?;

        let manager = VersionManager::new(temp_dir.path());
        let result = manager.read_cargo_version();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No version found in Cargo.toml")
        );
        Ok(())
    }

    #[test]
    fn test_package_json_invalid_json() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("package.json"), "not valid json {{")?;

        let manager = VersionManager::new(temp_dir.path());
        let result = manager.read_package_json_version();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to parse package.json")
        );
        Ok(())
    }

    #[test]
    fn test_pyproject_toml_invalid_toml() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("pyproject.toml"), "invalid toml [[[")?;

        let manager = VersionManager::new(temp_dir.path());
        let result = manager.read_pyproject_version();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to parse pyproject.toml")
        );
        Ok(())
    }

    #[test]
    fn test_cargo_toml_invalid_toml() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("Cargo.toml"), "invalid toml [[[")?;

        let manager = VersionManager::new(temp_dir.path());
        let result = manager.read_cargo_version();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to parse Cargo.toml")
        );
        Ok(())
    }

    #[test]
    fn test_version_file_invalid_semver() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("VERSION"), "not-a-version")?;

        let manager = VersionManager::new(temp_dir.path());
        let result = manager.read_version_file();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid version format")
        );
        Ok(())
    }

    #[test]
    fn test_version_file_with_whitespace() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("VERSION"), "  1.2.3  \n")?;

        let manager = VersionManager::new(temp_dir.path());
        let version = manager.read_version_file()?;

        assert_eq!(version, Version::new(1, 2, 3));
        Ok(())
    }

    #[test]
    fn test_bump_version_with_out_of_sync_error() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_files(temp_dir.path(), "1.0.0")?;

        // Manually modify Cargo.toml to be out of sync
        let cargo_content = r#"[package]
name = "test"
version = "2.0.0"
"#;
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_content)?;

        let manager = VersionManager::new(temp_dir.path());
        let result = manager.bump_version(BumpType::Patch);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Version files are not synchronized")
        );
        Ok(())
    }

    #[test]
    fn test_sync_versions_with_all_three_build_systems() -> Result<()> {
        let temp_dir = TempDir::new()?;
        // Create files with 1.0.0 first
        create_test_files(temp_dir.path(), "1.0.0")?;
        create_package_json(temp_dir.path(), "1.0.0", false)?;

        // Then update VERSION file to 5.0.0
        fs::write(temp_dir.path().join("VERSION"), "5.0.0")?;

        let manager = VersionManager::new(temp_dir.path());
        manager.sync_versions()?;

        // Verify all versions are now 5.0.0
        assert_eq!(manager.read_cargo_version()?, Version::new(5, 0, 0));
        assert_eq!(manager.read_pyproject_version()?, Version::new(5, 0, 0));
        assert_eq!(manager.read_package_json_version()?, Version::new(5, 0, 0));
        Ok(())
    }

    #[test]
    fn test_verify_versions_with_all_systems_in_sync() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_files(temp_dir.path(), "3.2.1")?;
        create_package_json(temp_dir.path(), "3.2.1", false)?;

        let manager = VersionManager::new(temp_dir.path());
        let result = manager.verify_versions_in_sync();

        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_semver_with_build_metadata() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_files(temp_dir.path(), "1.0.0")?;

        let manager = VersionManager::new(temp_dir.path());
        manager.reset_version("1.0.0+build.123")?;

        let version = manager.read_version_file()?;
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
        assert_eq!(version.build.as_str(), "build.123");
        Ok(())
    }

    #[test]
    fn test_package_json_not_an_object() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("package.json"), "[]")?;

        let manager = VersionManager::new(temp_dir.path());
        let result = manager.read_package_json_version();

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_update_package_json_not_an_object() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("package.json"), "[]")?;

        let manager = VersionManager::new(temp_dir.path());
        let result = manager.update_package_json_version(&Version::new(1, 0, 0));

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("package.json root is not a JSON object")
        );
        Ok(())
    }

    #[test]
    fn test_toml_version_update_no_version_field() {
        let content = "[package]\nname = \"test\"\n";
        let result =
            VersionManager::update_toml_version(content, &Version::new(1, 0, 0), "package");

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No version field found")
        );
    }

    #[test]
    fn test_cargo_toml_with_workspace() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cargo_content = r#"[workspace]
members = ["member1", "member2"]

[package]
name = "test"
version = "1.2.3"
"#;
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_content)?;

        let manager = VersionManager::new(temp_dir.path());
        let version = manager.read_cargo_version()?;

        assert_eq!(version, Version::new(1, 2, 3));
        Ok(())
    }

    #[test]
    fn test_discover_manifests_in_subdirectories() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create subdirectories with manifest files
        fs::create_dir(temp_dir.path().join("rust-tool"))?;
        fs::write(
            temp_dir.path().join("rust-tool/Cargo.toml"),
            "[package]\nname = \"rust-tool\"\nversion = \"1.0.0\"\n",
        )?;

        fs::create_dir(temp_dir.path().join("python-client"))?;
        fs::write(
            temp_dir.path().join("python-client/pyproject.toml"),
            "[project]\nname = \"python-client\"\nversion = \"1.0.0\"\n",
        )?;

        fs::create_dir(temp_dir.path().join("js-sdk"))?;
        fs::write(
            temp_dir.path().join("js-sdk/package.json"),
            r#"{"name": "js-sdk", "version": "1.0.0"}"#,
        )?;

        let manager = VersionManager::new(temp_dir.path());
        let manifests = manager.discover_manifests()?;

        // Should find all three manifest files
        assert_eq!(manifests.len(), 3);

        // Verify each type is found
        let cargo_found = manifests.iter().any(|(_, sys)| *sys == BuildSystem::Cargo);
        let pyproject_found = manifests
            .iter()
            .any(|(_, sys)| *sys == BuildSystem::PyProject);
        let packagejson_found = manifests
            .iter()
            .any(|(_, sys)| *sys == BuildSystem::PackageJson);

        assert!(cargo_found, "Should find Cargo.toml");
        assert!(pyproject_found, "Should find pyproject.toml");
        assert!(packagejson_found, "Should find package.json");

        Ok(())
    }

    #[test]
    fn test_discover_manifests_respects_gitignore() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Initialize git repo (required for ignore crate to respect .gitignore)
        fs::create_dir(temp_dir.path().join(".git"))?;

        // Create .gitignore
        fs::write(
            temp_dir.path().join(".gitignore"),
            "target/\nnode_modules/\n.venv/\n",
        )?;

        // Create manifests in ignored directories
        fs::create_dir(temp_dir.path().join("target"))?;
        fs::write(
            temp_dir.path().join("target/Cargo.toml"),
            "[package]\nname = \"ignored\"\nversion = \"1.0.0\"\n",
        )?;

        fs::create_dir(temp_dir.path().join("node_modules"))?;
        fs::write(
            temp_dir.path().join("node_modules/package.json"),
            r#"{"name": "ignored", "version": "1.0.0"}"#,
        )?;

        // Create manifests in non-ignored directory
        fs::create_dir(temp_dir.path().join("src"))?;
        fs::write(
            temp_dir.path().join("src/Cargo.toml"),
            "[package]\nname = \"not-ignored\"\nversion = \"1.0.0\"\n",
        )?;

        let manager = VersionManager::new(temp_dir.path());
        let manifests = manager.discover_manifests()?;

        // Should only find the non-ignored manifest
        assert_eq!(
            manifests.len(),
            1,
            "Should only find 1 manifest (not in ignored dirs)"
        );

        let found_path = &manifests[0].0;
        assert!(
            found_path.ends_with("src/Cargo.toml"),
            "Should find src/Cargo.toml, got: {}",
            found_path.display()
        );

        Ok(())
    }

    #[test]
    fn test_discover_manifests_rejects_nested_version_files() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create top-level VERSION
        fs::write(temp_dir.path().join("VERSION"), "1.0.0")?;

        // Create nested VERSION file (should be error)
        fs::create_dir(temp_dir.path().join("subproject"))?;
        fs::write(temp_dir.path().join("subproject/VERSION"), "2.0.0")?;

        let manager = VersionManager::new(temp_dir.path());
        let result = manager.discover_manifests();

        assert!(result.is_err(), "Should error on nested VERSION file");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("VERSION") || err_msg.contains("nested"),
            "Error should mention nested VERSION file, got: {err_msg}"
        );

        Ok(())
    }

    #[test]
    #[cfg(unix)]
    fn test_discover_manifests_rejects_symlinks() -> Result<()> {
        use std::os::unix::fs::symlink;

        let temp_dir = TempDir::new()?;

        // Create a real directory with manifest
        fs::create_dir(temp_dir.path().join("real-dir"))?;
        fs::write(
            temp_dir.path().join("real-dir/Cargo.toml"),
            "[package]\nname = \"real\"\nversion = \"1.0.0\"\n",
        )?;

        // Create a symlink to the directory
        let symlink_path = temp_dir.path().join("link-dir");
        symlink(temp_dir.path().join("real-dir"), &symlink_path)?;

        let manager = VersionManager::new(temp_dir.path());
        let result = manager.discover_manifests();

        assert!(result.is_err(), "Should error on symlink");
        let err_msg = result.unwrap_err().to_string().to_lowercase();
        assert!(
            err_msg.contains("symlink") || err_msg.contains("symbolic"),
            "Error should mention symlink, got: {err_msg}"
        );

        Ok(())
    }

    #[test]
    fn test_bump_cascade_updates_all_manifests() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create top-level VERSION
        fs::write(temp_dir.path().join("VERSION"), "1.0.0")?;

        // Create subdirectories with different manifest types
        fs::create_dir(temp_dir.path().join("rust-tool"))?;
        fs::write(
            temp_dir.path().join("rust-tool/Cargo.toml"),
            "[package]\nname = \"rust-tool\"\nversion = \"1.0.0\"\n",
        )?;

        fs::create_dir(temp_dir.path().join("python-client"))?;
        fs::write(
            temp_dir.path().join("python-client/pyproject.toml"),
            "[project]\nname = \"python-client\"\nversion = \"1.0.0\"\n",
        )?;

        fs::create_dir(temp_dir.path().join("js-sdk"))?;
        fs::write(
            temp_dir.path().join("js-sdk/package.json"),
            r#"{"name": "js-sdk", "version": "1.0.0"}"#,
        )?;

        let manager = VersionManager::new(temp_dir.path());
        manager.bump_cascade(BumpType::Minor)?;

        // Verify VERSION was bumped
        let version = manager.read_version_file()?;
        assert_eq!(version, Version::new(1, 1, 0));

        // Verify all manifests were updated
        let cargo_content = fs::read_to_string(temp_dir.path().join("rust-tool/Cargo.toml"))?;
        assert!(cargo_content.contains("version = \"1.1.0\""));

        let pyproject_content =
            fs::read_to_string(temp_dir.path().join("python-client/pyproject.toml"))?;
        assert!(pyproject_content.contains("version = \"1.1.0\""));

        let package_content = fs::read_to_string(temp_dir.path().join("js-sdk/package.json"))?;
        assert!(package_content.contains("\"version\": \"1.1.0\""));

        Ok(())
    }

    // Helper to initialize a git repo in a temp directory with an initial commit
    fn init_git_repo(dir: &Path) -> Result<()> {
        use std::process::Command;
        let run = |args: &[&str]| -> Result<()> {
            let output = Command::new("git").args(args).current_dir(dir).output()?;
            if !output.status.success() {
                anyhow::bail!(
                    "git {:?} failed: {}",
                    args,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Ok(())
        };

        run(&["init"])?;
        run(&["config", "user.email", "test@test.com"])?;
        run(&["config", "user.name", "Test"])?;
        run(&["commit", "--allow-empty", "-m", "init"])?;
        Ok(())
    }

    #[test]
    fn test_count_rc_tags_no_git() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("VERSION"), "1.0.0")?;

        let manager = VersionManager::new(temp_dir.path());
        let version = Version::new(1, 0, 0);
        let result = manager.count_rc_tags(&version);

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_count_rc_tags_no_tags() -> Result<()> {
        let temp_dir = TempDir::new()?;
        init_git_repo(temp_dir.path())?;
        fs::write(temp_dir.path().join("VERSION"), "1.0.0")?;

        let manager = VersionManager::new(temp_dir.path());
        let version = Version::new(1, 0, 0);
        let count = manager.count_rc_tags(&version)?;

        assert_eq!(count, 0);
        Ok(())
    }

    #[test]
    fn test_count_rc_tags_with_existing() -> Result<()> {
        let temp_dir = TempDir::new()?;
        init_git_repo(temp_dir.path())?;

        // Create RC tags
        std::process::Command::new("git")
            .args(["tag", "v1.0.0-rc.1"])
            .current_dir(temp_dir.path())
            .output()?;
        std::process::Command::new("git")
            .args(["tag", "v1.0.0-rc.2"])
            .current_dir(temp_dir.path())
            .output()?;

        let manager = VersionManager::new(temp_dir.path());
        let version = Version::new(1, 0, 0);
        let count = manager.count_rc_tags(&version)?;

        assert_eq!(count, 2);
        Ok(())
    }

    #[test]
    fn test_count_rc_tags_ignores_other_versions() -> Result<()> {
        let temp_dir = TempDir::new()?;
        init_git_repo(temp_dir.path())?;

        std::process::Command::new("git")
            .args(["tag", "v2.0.0-rc.1"])
            .current_dir(temp_dir.path())
            .output()?;
        std::process::Command::new("git")
            .args(["tag", "v1.0.0-rc.1"])
            .current_dir(temp_dir.path())
            .output()?;

        let manager = VersionManager::new(temp_dir.path());
        let version = Version::new(1, 0, 0);
        let count = manager.count_rc_tags(&version)?;

        assert_eq!(count, 1);
        Ok(())
    }

    #[test]
    fn test_next_rc_version_first() -> Result<()> {
        let temp_dir = TempDir::new()?;
        init_git_repo(temp_dir.path())?;
        fs::write(temp_dir.path().join("VERSION"), "1.0.0")?;

        let manager = VersionManager::new(temp_dir.path());
        let rc = manager.next_rc_version()?;

        assert_eq!(rc.to_string(), "1.0.0-rc.1");
        Ok(())
    }

    #[test]
    fn test_next_rc_version_increment() -> Result<()> {
        let temp_dir = TempDir::new()?;
        init_git_repo(temp_dir.path())?;
        fs::write(temp_dir.path().join("VERSION"), "1.0.0")?;

        std::process::Command::new("git")
            .args(["tag", "v1.0.0-rc.1"])
            .current_dir(temp_dir.path())
            .output()?;
        std::process::Command::new("git")
            .args(["tag", "v1.0.0-rc.2"])
            .current_dir(temp_dir.path())
            .output()?;

        let manager = VersionManager::new(temp_dir.path());
        let rc = manager.next_rc_version()?;

        assert_eq!(rc.to_string(), "1.0.0-rc.3");
        Ok(())
    }

    #[test]
    fn test_next_rc_version_rejects_prerelease() -> Result<()> {
        let temp_dir = TempDir::new()?;
        init_git_repo(temp_dir.path())?;
        fs::write(temp_dir.path().join("VERSION"), "1.0.0-beta.1")?;

        let manager = VersionManager::new(temp_dir.path());
        let result = manager.next_rc_version();

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("clean M.m.p"),
            "Error should mention clean version requirement, got: {err_msg}"
        );
        Ok(())
    }

    #[test]
    fn test_read_version_file_with_inline_comment() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(
            temp_dir.path().join("VERSION"),
            "1.2.3 # x-release-please-version\n",
        )?;

        let manager = VersionManager::new(temp_dir.path());
        let version = manager.read_version_file()?;

        assert_eq!(version, Version::new(1, 2, 3));
        Ok(())
    }

    #[test]
    fn test_read_version_file_with_comment_no_space() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("VERSION"), "2.0.0#some-comment\n")?;

        let manager = VersionManager::new(temp_dir.path());
        let version = manager.read_version_file()?;

        assert_eq!(version, Version::new(2, 0, 0));
        Ok(())
    }

    #[test]
    fn test_write_version_file_preserves_comment() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(
            temp_dir.path().join("VERSION"),
            "1.0.0 # x-release-please-version\n",
        )?;

        let manager = VersionManager::new(temp_dir.path());
        manager.write_version_file(&Version::new(2, 3, 4))?;

        let content = fs::read_to_string(temp_dir.path().join("VERSION"))?;
        assert_eq!(content, "2.3.4 # x-release-please-version\n");
        Ok(())
    }

    #[test]
    fn test_write_version_file_no_comment() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("VERSION"), "1.0.0\n")?;

        let manager = VersionManager::new(temp_dir.path());
        manager.write_version_file(&Version::new(3, 0, 0))?;

        let content = fs::read_to_string(temp_dir.path().join("VERSION"))?;
        assert_eq!(content, "3.0.0\n");
        Ok(())
    }

    #[test]
    fn test_write_version_file_new_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        // No VERSION file exists

        let manager = VersionManager::new(temp_dir.path());
        manager.write_version_file(&Version::new(1, 0, 0))?;

        let content = fs::read_to_string(temp_dir.path().join("VERSION"))?;
        assert_eq!(content, "1.0.0\n");
        Ok(())
    }

    #[test]
    fn test_bump_roundtrip_with_comment() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(
            temp_dir.path().join("VERSION"),
            "1.0.0 # x-release-please-version\n",
        )?;

        let manager = VersionManager::new(temp_dir.path());

        // Read should strip comment
        let version = manager.read_version_file()?;
        assert_eq!(version, Version::new(1, 0, 0));

        // Write should preserve comment
        let bumped = Version::new(1, 1, 0);
        manager.write_version_file(&bumped)?;

        let content = fs::read_to_string(temp_dir.path().join("VERSION"))?;
        assert_eq!(content, "1.1.0 # x-release-please-version\n");

        // Read again should work
        let version = manager.read_version_file()?;
        assert_eq!(version, Version::new(1, 1, 0));
        Ok(())
    }

    #[test]
    fn test_with_version_file_reads_custom_filename() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("version.txt"), "3.2.1\n")?;

        let manager = VersionManager::with_version_file(temp_dir.path(), "version.txt");
        let version = manager.read_version_file()?;

        assert_eq!(version, Version::new(3, 2, 1));
        Ok(())
    }

    #[test]
    fn test_with_version_file_writes_custom_filename() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("version.txt"), "1.0.0\n")?;

        let manager = VersionManager::with_version_file(temp_dir.path(), "version.txt");
        manager.write_version_file(&Version::new(2, 0, 0))?;

        let content = fs::read_to_string(temp_dir.path().join("version.txt"))?;
        assert_eq!(content, "2.0.0\n");

        // Ensure no "VERSION" file was created
        assert!(!temp_dir.path().join("VERSION").exists());
        Ok(())
    }

    #[test]
    fn test_with_version_file_bump_syncs_all() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("version.txt"), "1.0.0\n")?;
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"1.0.0\"\n",
        )?;
        fs::write(
            temp_dir.path().join("pyproject.toml"),
            "[project]\nname = \"test\"\nversion = \"1.0.0\"\n",
        )?;

        let manager = VersionManager::with_version_file(temp_dir.path(), "version.txt");
        manager.bump_version(BumpType::Minor)?;

        assert_eq!(manager.read_version_file()?, Version::new(1, 1, 0));
        assert_eq!(manager.read_cargo_version()?, Version::new(1, 1, 0));
        assert_eq!(manager.read_pyproject_version()?, Version::new(1, 1, 0));
        Ok(())
    }

    #[test]
    fn test_with_version_file_sync() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("version.txt"), "5.0.0\n")?;
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"1.0.0\"\n",
        )?;

        let manager = VersionManager::with_version_file(temp_dir.path(), "version.txt");
        manager.sync_versions()?;

        assert_eq!(manager.read_cargo_version()?, Version::new(5, 0, 0));
        Ok(())
    }

    #[test]
    fn test_with_version_file_verify_in_sync() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("version.txt"), "2.0.0\n")?;
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"2.0.0\"\n",
        )?;

        let manager = VersionManager::with_version_file(temp_dir.path(), "version.txt");
        assert!(manager.verify_versions_in_sync().is_ok());
        Ok(())
    }

    #[test]
    fn test_with_version_file_verify_out_of_sync() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("version.txt"), "2.0.0\n")?;
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"1.0.0\"\n",
        )?;

        let manager = VersionManager::with_version_file(temp_dir.path(), "version.txt");
        assert!(manager.verify_versions_in_sync().is_err());
        Ok(())
    }

    #[test]
    fn test_with_version_file_comment_preservation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(
            temp_dir.path().join("version.txt"),
            "1.0.0 # x-release-please-version\n",
        )?;

        let manager = VersionManager::with_version_file(temp_dir.path(), "version.txt");

        // Read strips comment
        let version = manager.read_version_file()?;
        assert_eq!(version, Version::new(1, 0, 0));

        // Write preserves comment
        manager.write_version_file(&Version::new(2, 0, 0))?;
        let content = fs::read_to_string(temp_dir.path().join("version.txt"))?;
        assert_eq!(content, "2.0.0 # x-release-please-version\n");
        Ok(())
    }

    #[test]
    fn test_default_version_file_constant() {
        assert_eq!(DEFAULT_VERSION_FILE, "VERSION");
    }

    #[test]
    fn test_new_uses_default_filename() {
        let manager = VersionManager::new("/tmp");
        assert_eq!(manager.version_file, "VERSION");
    }

    #[test]
    fn test_bump_cascade_dry_run_does_not_write_files() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create top-level VERSION
        fs::write(temp_dir.path().join("VERSION"), "1.0.0")?;

        // Create subdirectories with different manifest types
        fs::create_dir(temp_dir.path().join("rust-tool"))?;
        fs::write(
            temp_dir.path().join("rust-tool/Cargo.toml"),
            "[package]\nname = \"rust-tool\"\nversion = \"1.0.0\"\n",
        )?;

        fs::create_dir(temp_dir.path().join("python-client"))?;
        fs::write(
            temp_dir.path().join("python-client/pyproject.toml"),
            "[project]\nname = \"python-client\"\nversion = \"1.0.0\"\n",
        )?;

        let manager = VersionManager::new(temp_dir.path());
        let changes = manager.bump_cascade_dry_run(BumpType::Patch)?;

        // Should return preview of changes
        assert_eq!(changes.new_version, Version::new(1, 0, 1));
        assert_eq!(changes.files_to_update.len(), 3); // VERSION + Cargo.toml + pyproject.toml

        // Verify files were NOT changed
        let version = manager.read_version_file()?;
        assert_eq!(
            version,
            Version::new(1, 0, 0),
            "VERSION should not change in dry-run"
        );

        let cargo_content = fs::read_to_string(temp_dir.path().join("rust-tool/Cargo.toml"))?;
        assert!(
            cargo_content.contains("version = \"1.0.0\""),
            "Cargo.toml should not change in dry-run"
        );

        let pyproject_content =
            fs::read_to_string(temp_dir.path().join("python-client/pyproject.toml"))?;
        assert!(
            pyproject_content.contains("version = \"1.0.0\""),
            "pyproject.toml should not change in dry-run"
        );

        Ok(())
    }
}
