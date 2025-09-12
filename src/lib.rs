//! Versioneer - A tool to synchronize VERSION files with build system version declarations
//!
//! This library provides functionality to read, parse, and update version information
//! across different file formats including VERSION files, Cargo.toml, and pyproject.toml.

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

    /// Create a git tag for the current version
    ///
    /// # Errors
    ///
    /// Returns an error if git operations fail or if not in a git repository.
    pub fn create_git_tag(&self, tag_format: Option<&str>) -> Result<String> {
        let version = self.read_version_file()?;
        let repo = git2::Repository::open(&self.base_path)
            .context("Failed to open git repository. Make sure you're in a git repository.")?;

        let tag_name = if let Some(format) = tag_format {
            self.format_tag_string(format, &version)?
        } else {
            self.default_tag_format(&version)?
        };

        // Get the current HEAD commit
        let head = repo.head().context("Failed to get HEAD reference")?;
        let commit = head.peel_to_commit().context("Failed to get HEAD commit")?;

        // Create an annotated tag
        let signature = repo.signature()
            .context("Failed to get git signature. Make sure git user.name and user.email are configured.")?;

        let message = format!("Release {tag_name}");

        repo.tag(&tag_name, &commit.into_object(), &signature, &message, false)
            .with_context(|| format!("Failed to create git tag '{tag_name}'"))?;

        Ok(tag_name)
    }

    /// Get the repository name from git remote or directory name
    fn get_repository_name(&self) -> Result<String> {
        // Try to get repository name from git remote
        if let Ok(repo) = git2::Repository::open(&self.base_path) {
            if let Ok(remote) = repo.find_remote("origin") {
                if let Some(url) = remote.url() {
                    if let Some(name) = Self::extract_repo_name_from_url(url) {
                        return Ok(name);
                    }
                }
            }
        }

        // Fallback to directory name
        self.base_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(std::string::ToString::to_string)
            .context("Failed to get repository name from directory")
    }

    /// Extract repository name from git remote URL
    fn extract_repo_name_from_url(url: &str) -> Option<String> {
        // Handle GitHub URLs like git@github.com:user/repo.git or https://github.com/user/repo.git
        let url = url.trim_end_matches(".git");
        
        if let Some(name) = url.split('/').next_back() {
            return Some(name.to_string());
        }

        if let Some(colon_pos) = url.rfind(':') {
            if let Some(name) = url[(colon_pos + 1)..].split('/').next_back() {
                return Some(name.to_string());
            }
        }

        None
    }

    /// Generate default tag format: {repository_name}-v{version}
    fn default_tag_format(&self, version: &Version) -> Result<String> {
        let repo_name = self.get_repository_name()?;
        Ok(format!("{repo_name}-v{version}"))
    }

    /// Format tag string with placeholders
    #[allow(clippy::literal_string_with_formatting_args)]
    fn format_tag_string(&self, format: &str, version: &Version) -> Result<String> {
        let repo_name = self.get_repository_name()?;
        
        let result = format
            .replace("{repository_name}", &repo_name)
            .replace("{version}", &version.to_string())
            .replace("{major}", &version.major.to_string())
            .replace("{minor}", &version.minor.to_string())
            .replace("{patch}", &version.patch.to_string());
            
        Ok(result)
    }

    /// Check if the current directory is a git repository
    #[must_use]
    pub fn is_git_repository(&self) -> bool {
        git2::Repository::open(&self.base_path).is_ok()
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
    fn test_git_repository_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let manager = VersionManager::new(temp_dir.path());
        
        // Should return false for non-git directory
        assert!(!manager.is_git_repository());
        Ok(())
    }

    #[test]
    fn test_repository_name_from_path() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let manager = VersionManager::new(temp_dir.path());
        
        let repo_name = manager.get_repository_name()?;
        // Should get directory name as fallback
        assert!(!repo_name.is_empty());
        Ok(())
    }

    #[test]
    #[allow(clippy::literal_string_with_formatting_args)]
    fn test_tag_format_parsing() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_files(temp_dir.path(), "1.2.3")?;
        
        let manager = VersionManager::new(temp_dir.path());
        let version = Version::new(1, 2, 3);
        
        let result = manager.format_tag_string("v{version}", &version)?;
        assert_eq!(result, "v1.2.3");
        
        let result = manager.format_tag_string("release-{major}.{minor}.{patch}", &version)?;
        assert_eq!(result, "release-1.2.3");
        
        let result = manager.format_tag_string("{repository_name}-v{major}.{minor}", &version)?;
        assert!(result.contains("-v1.2"));
        
        Ok(())
    }

    #[test]
    fn test_extract_repo_name_from_url() {
        assert_eq!(
            VersionManager::extract_repo_name_from_url("git@github.com:user/repo.git"),
            Some("repo".to_string())
        );
        
        assert_eq!(
            VersionManager::extract_repo_name_from_url("https://github.com/user/myproject.git"),
            Some("myproject".to_string())
        );
        
        assert_eq!(
            VersionManager::extract_repo_name_from_url("https://github.com/user/myproject"),
            Some("myproject".to_string())
        );
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
        assert!(result.unwrap_err().to_string().contains("Invalid semantic version format"));

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
        assert!(result.unwrap_err().to_string().contains("Invalid semantic version format"));

        Ok(())
    }
}
