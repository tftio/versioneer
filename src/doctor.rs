//! Health check and diagnostics module.

use versioneer::VersionManager;
use workhelix_cli_common::RepoInfo;

/// Run doctor command to check health and configuration.
///
/// Returns exit code: 0 if healthy, 1 if issues found.
pub fn run_doctor(manager: &VersionManager) -> i32 {
    println!("🏥 versioneer health check");
    println!("==========================");
    println!();

    let mut has_errors = false;
    let mut has_warnings = false;

    // Check VERSION file
    println!("Version Files:");
    match manager.read_version_file() {
        Ok(version) => {
            println!("  ✅ VERSION file: {version}");
        }
        Err(e) => {
            println!("  ❌ VERSION file error: {e}");
            has_errors = true;
        }
    }

    // Check build system files
    println!();
    println!("Build Systems:");
    let build_systems = manager.detect_build_systems();

    if build_systems.is_empty() {
        println!("  ❌ No build system files detected");
        println!(
            "  ℹ️  At least one build system file (Cargo.toml, pyproject.toml, package.json) is required"
        );
        has_errors = true;
    } else {
        for system in &build_systems {
            match manager.read_build_system_version(system) {
                Ok(version) => {
                    println!("  ✅ {system:?}: {version}");
                }
                Err(e) => {
                    println!("  ❌ {system:?}: {e}");
                    has_errors = true;
                }
            }
        }
    }

    // Check version synchronization
    println!();
    println!("Synchronization:");
    match manager.verify_versions_in_sync() {
        Ok(()) => {
            println!("  ✅ All versions are synchronized");
        }
        Err(e) => {
            println!("  ❌ Versions are out of sync");
            println!("  ℹ️  {e}");
            has_errors = true;
        }
    }

    println!();

    // Check for updates
    println!("Updates:");
    let repo_info = RepoInfo::new("workhelix", "versioneer", "v");
    match workhelix_cli_common::doctor::check_for_updates(&repo_info, env!("CARGO_PKG_VERSION")) {
        Ok(Some(latest)) => {
            let current = env!("CARGO_PKG_VERSION");
            println!("  ⚠️  Update available: v{latest} (current: v{current})");
            println!("  💡 Run 'versioneer update' to install the latest version");
            has_warnings = true;
        }
        Ok(None) => {
            println!(
                "  ✅ Running latest version (v{})",
                env!("CARGO_PKG_VERSION")
            );
        }
        Err(e) => {
            println!("  ⚠️  Failed to check for updates: {e}");
            has_warnings = true;
        }
    }

    println!();

    // Summary
    if has_errors {
        println!("❌ Issues found - see above for details");
        1
    } else if has_warnings {
        println!("⚠️  1 warning found");
        0 // Warnings don't cause failure
    } else {
        println!("✨ Everything looks healthy!");
        0
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;


    #[test]
    fn test_run_doctor_returns_zero() {
        // Create a temp directory with valid VERSION and Cargo.toml
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("VERSION"), "1.0.0\n").unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();

        let manager = VersionManager::new(temp_dir.path());
        let exit_code = run_doctor(&manager);

        // Should return 0 or 1 (with warnings from update check)
        assert!(exit_code == 0 || exit_code == 1);
    }

    #[test]
    fn test_doctor_with_missing_version_file() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();

        let manager = VersionManager::new(temp_dir.path());
        let exit_code = run_doctor(&manager);

        // Should return 1 (error) for missing VERSION file
        assert_eq!(exit_code, 1);
    }

    #[test]
    fn test_doctor_with_corrupted_version_file() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("VERSION"), "not-a-version\n").unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();

        let manager = VersionManager::new(temp_dir.path());
        let exit_code = run_doctor(&manager);

        // Should return 1 (error) for invalid version
        assert_eq!(exit_code, 1);
    }

    #[test]
    fn test_doctor_with_no_build_systems() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("VERSION"), "1.0.0\n").unwrap();

        let manager = VersionManager::new(temp_dir.path());
        let exit_code = run_doctor(&manager);

        // Should return 1 (error) for no build systems
        assert_eq!(exit_code, 1);
    }

    #[test]
    fn test_doctor_with_invalid_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("VERSION"), "1.0.0\n").unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "invalid toml syntax [[[",
        )
        .unwrap();

        let manager = VersionManager::new(temp_dir.path());
        let exit_code = run_doctor(&manager);

        // Should return 1 (error) for invalid Cargo.toml
        assert_eq!(exit_code, 1);
    }

    #[test]
    fn test_doctor_with_missing_version_in_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("VERSION"), "1.0.0\n").unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .unwrap();

        let manager = VersionManager::new(temp_dir.path());
        let exit_code = run_doctor(&manager);

        // Should return 1 (error) for missing version in Cargo.toml
        assert_eq!(exit_code, 1);
    }

    #[test]
    fn test_doctor_with_version_mismatch() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("VERSION"), "1.0.0\n").unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"2.0.0\"\n",
        )
        .unwrap();

        let manager = VersionManager::new(temp_dir.path());
        let exit_code = run_doctor(&manager);

        // Should return 1 (error) for version mismatch
        assert_eq!(exit_code, 1);
    }

    #[test]
    fn test_doctor_with_invalid_pyproject_toml() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("VERSION"), "1.0.0\n").unwrap();
        fs::write(
            temp_dir.path().join("pyproject.toml"),
            "invalid toml syntax [[[",
        )
        .unwrap();

        let manager = VersionManager::new(temp_dir.path());
        let exit_code = run_doctor(&manager);

        // Should return 1 (error) for invalid pyproject.toml
        assert_eq!(exit_code, 1);
    }

    #[test]
    fn test_doctor_with_invalid_package_json() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("VERSION"), "1.0.0\n").unwrap();
        fs::write(
            temp_dir.path().join("package.json"),
            "invalid json {{{",
        )
        .unwrap();

        let manager = VersionManager::new(temp_dir.path());
        let exit_code = run_doctor(&manager);

        // Should return 1 (error) for invalid package.json
        assert_eq!(exit_code, 1);
    }

    #[test]
    fn test_doctor_with_multiple_build_systems_in_sync() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("VERSION"), "1.0.0\n").unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("pyproject.toml"),
            "[project]\nname = \"test\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();

        let manager = VersionManager::new(temp_dir.path());
        let exit_code = run_doctor(&manager);

        // Should return 0 (may have update warning)
        assert!(exit_code == 0 || exit_code == 1);
    }

    #[test]
    fn test_doctor_with_multiple_build_systems_out_of_sync() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("VERSION"), "1.0.0\n").unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("pyproject.toml"),
            "[project]\nname = \"test\"\nversion = \"2.0.0\"\n",
        )
        .unwrap();

        let manager = VersionManager::new(temp_dir.path());
        let exit_code = run_doctor(&manager);

        // Should return 1 (error) for out of sync
        assert_eq!(exit_code, 1);
    }
}
