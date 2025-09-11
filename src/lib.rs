//! Versioneer - A tool to synchronize VERSION files with build system version declarations
//! 
//! This library provides functionality to read, parse, and update version information
//! across different file formats including VERSION files, Cargo.toml, and pyproject.toml.

use anyhow::{Context, Result};
use semver::Version;
use std::fs;
use std::path::Path;

/// Represents different types of build system files that can contain version information
#[derive(Debug, Clone, PartialEq)]
pub enum BuildSystem {
    /// Cargo.toml file for Rust projects
    Cargo,
    /// pyproject.toml file for Python projects
    PyProject,
}

/// Represents a version bump type following semantic versioning
#[derive(Debug, Clone, PartialEq)]
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
    /// Create a new VersionManager for the given directory
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Read the current version from the VERSION file
    pub fn read_version_file(&self) -> Result<Version> {
        let version_path = self.base_path.join("VERSION");
        let content = fs::read_to_string(&version_path)
            .with_context(|| format!("Failed to read VERSION file at {}", version_path.display()))?;
        
        let version_str = content.trim();
        Version::parse(version_str)
            .with_context(|| format!("Invalid version format in VERSION file: {}", version_str))
    }

    /// Write a version to the VERSION file
    pub fn write_version_file(&self, version: &Version) -> Result<()> {
        let version_path = self.base_path.join("VERSION");
        fs::write(&version_path, version.to_string())
            .with_context(|| format!("Failed to write VERSION file at {}", version_path.display()))
    }

    /// Detect which build system files are present
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
    pub fn read_build_system_version(&self, system: &BuildSystem) -> Result<Version> {
        match system {
            BuildSystem::Cargo => self.read_cargo_version(),
            BuildSystem::PyProject => self.read_pyproject_version(),
        }
    }

    /// Update version in a specific build system file
    pub fn update_build_system_version(&self, system: &BuildSystem, version: &Version) -> Result<()> {
        match system {
            BuildSystem::Cargo => self.update_cargo_version(version),
            BuildSystem::PyProject => self.update_pyproject_version(version),
        }
    }

    /// Bump version according to semantic versioning rules
    pub fn bump_version(&self, bump_type: BumpType) -> Result<()> {
        // Ensure all versions are in sync before bumping
        self.verify_versions_in_sync()?;
        
        let current_version = self.read_version_file()?;
        let new_version = match bump_type {
            BumpType::Major => Version::new(current_version.major + 1, 0, 0),
            BumpType::Minor => Version::new(current_version.major, current_version.minor + 1, 0),
            BumpType::Patch => Version::new(current_version.major, current_version.minor, current_version.patch + 1),
        };

        // Update VERSION file
        self.write_version_file(&new_version)?;

        // Update all detected build system files
        let build_systems = self.detect_build_systems();
        for system in &build_systems {
            self.update_build_system_version(system, &new_version)
                .with_context(|| format!("Failed to update {:?} version", system))?;
        }

        Ok(())
    }

    /// Verify that all version files are synchronized
    pub fn verify_versions_in_sync(&self) -> Result<()> {
        let version_file_version = self.read_version_file()?;
        let build_systems = self.detect_build_systems();
        
        let mut mismatched = Vec::new();
        
        for system in &build_systems {
            match self.read_build_system_version(system) {
                Ok(system_version) => {
                    if system_version != version_file_version {
                        mismatched.push(format!(
                            "{:?} has version {} but VERSION file has {}",
                            system, system_version, version_file_version
                        ));
                    }
                }
                Err(e) => {
                    mismatched.push(format!("Failed to read {:?} version: {}", system, e));
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
    pub fn sync_versions(&self) -> Result<()> {
        let version = self.read_version_file()?;
        let build_systems = self.detect_build_systems();
        
        for system in &build_systems {
            self.update_build_system_version(system, &version)
                .with_context(|| format!("Failed to sync {:?} version", system))?;
        }
        
        Ok(())
    }

    /// Read version from Cargo.toml
    fn read_cargo_version(&self) -> Result<Version> {
        let cargo_path = self.base_path.join("Cargo.toml");
        let content = fs::read_to_string(&cargo_path)
            .with_context(|| format!("Failed to read Cargo.toml at {}", cargo_path.display()))?;
        
        let cargo_toml: toml::Value = toml::from_str(&content)
            .with_context(|| "Failed to parse Cargo.toml")?;
        
        let version_str = cargo_toml
            .get("package")
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
            .context("No version found in Cargo.toml [package] section")?;
        
        Version::parse(version_str)
            .with_context(|| format!("Invalid version format in Cargo.toml: {}", version_str))
    }

    /// Update version in Cargo.toml
    fn update_cargo_version(&self, version: &Version) -> Result<()> {
        let cargo_path = self.base_path.join("Cargo.toml");
        let content = fs::read_to_string(&cargo_path)
            .with_context(|| format!("Failed to read Cargo.toml at {}", cargo_path.display()))?;
        
        let updated_content = self.update_toml_version(&content, version, "package")?;
        
        fs::write(&cargo_path, updated_content)
            .with_context(|| format!("Failed to write Cargo.toml at {}", cargo_path.display()))
    }

    /// Read version from pyproject.toml
    fn read_pyproject_version(&self) -> Result<Version> {
        let pyproject_path = self.base_path.join("pyproject.toml");
        let content = fs::read_to_string(&pyproject_path)
            .with_context(|| format!("Failed to read pyproject.toml at {}", pyproject_path.display()))?;
        
        let pyproject_toml: toml::Value = toml::from_str(&content)
            .with_context(|| "Failed to parse pyproject.toml")?;
        
        let version_str = pyproject_toml
            .get("project")
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
            .context("No version found in pyproject.toml [project] section")?;
        
        Version::parse(version_str)
            .with_context(|| format!("Invalid version format in pyproject.toml: {}", version_str))
    }

    /// Update version in pyproject.toml
    fn update_pyproject_version(&self, version: &Version) -> Result<()> {
        let pyproject_path = self.base_path.join("pyproject.toml");
        let content = fs::read_to_string(&pyproject_path)
            .with_context(|| format!("Failed to read pyproject.toml at {}", pyproject_path.display()))?;
        
        let updated_content = self.update_toml_version(&content, version, "project")?;
        
        fs::write(&pyproject_path, updated_content)
            .with_context(|| format!("Failed to write pyproject.toml at {}", pyproject_path.display()))
    }

    /// Helper to update version in TOML content
    fn update_toml_version(&self, content: &str, version: &Version, section: &str) -> Result<String> {
        use regex::Regex;
        
        // More flexible regex that handles multiline TOML sections with better whitespace handling
        let pattern = format!(r#"(?s)(\[{}\][^\[]*?version\s*=\s*")[^"]*(")"#, section);
        let re = Regex::new(&pattern)
            .context("Failed to create regex for version replacement")?;
        
        let result = re.replace(content, format!("${{1}}{}${{2}}", version));
        
        if result == content {
            anyhow::bail!("No version field found in [{}] section", section);
        }
        
        Ok(result.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_files(dir: &Path, version: &str) -> Result<()> {
        // Create VERSION file
        fs::write(dir.join("VERSION"), version)?;
        
        // Create Cargo.toml
        let cargo_content = format!(r#"[package]
name = "test"
version = "{}"
edition = "2021"

[dependencies]
"#, version);
        fs::write(dir.join("Cargo.toml"), cargo_content)?;
        
        // Create pyproject.toml
        let pyproject_content = format!(r#"[project]
name = "test"
version = "{}"
description = "Test project"

[build-system]
requires = ["setuptools"]
build-backend = "setuptools.build_meta"
"#, version);
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
}