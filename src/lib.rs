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

/// Core version management functionality
pub struct VersionManager {
    /// The current working directory path
    pub base_path: std::path::PathBuf,
}

impl VersionManager {
    /// Create a new `VersionManager` for the given directory
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Read the current version from the VERSION file
    ///
    /// # Errors
    ///
    /// Returns an error if the VERSION file cannot be read or contains an invalid version format.
    pub fn read_version_file(&self) -> Result<Version> {
        let version_path = self.base_path.join("VERSION");
        let content = fs::read_to_string(&version_path).with_context(|| {
            format!("Failed to read VERSION file at {}", version_path.display())
        })?;

        let version_str = content.trim();
        Version::parse(version_str)
            .with_context(|| format!("Invalid version format in VERSION file: {version_str}"))
    }

    /// Write a version to the VERSION file
    ///
    /// # Errors
    ///
    /// Returns an error if the VERSION file cannot be written to.
    pub fn write_version_file(&self, version: &Version) -> Result<()> {
        let version_path = self.base_path.join("VERSION");
        fs::write(&version_path, version.to_string())
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

        let result = re.replace(content, format!("${{1}}{version}${{2}}"));

        if result == content {
            anyhow::bail!("No version field found in [{section}] section");
        }

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
}
