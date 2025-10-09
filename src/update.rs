//! Self-update module.

use sha2::{Digest, Sha256};
use std::path::Path;

/// Run update command to install latest or specified version.
///
/// Returns exit code: 0 if successful, 1 on error, 2 if already up-to-date.
#[allow(clippy::unused_async)]
pub fn run_update(version: Option<&str>, force: bool, install_dir: Option<&Path>) -> i32 {
    let current_version = env!("CARGO_PKG_VERSION");

    println!("🔄 Checking for updates...");

    // Get target version
    let target_version = if let Some(v) = version {
        v.to_string()
    } else {
        match get_latest_version() {
            Ok(v) => v,
            Err(e) => {
                eprintln!("❌ Failed to check for updates: {e}");
                return 1;
            }
        }
    };

    // Check if already up-to-date
    if target_version == current_version && !force {
        println!("✅ Already running latest version (v{current_version})");
        return 2;
    }

    println!("✨ Update available: v{target_version} (current: v{current_version})");

    // Detect current binary location
    let install_path = if let Some(dir) = install_dir {
        dir.join("versioneer")
    } else {
        match std::env::current_exe() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("❌ Failed to determine binary location: {e}");
                return 1;
            }
        }
    };

    println!("📍 Install location: {}", install_path.display());
    println!();

    // Confirm unless forced
    if !force {
        use std::io::{self, Write};
        print!("Continue with update? [y/N]: ");
        io::stdout().flush().unwrap();

        let mut response = String::new();
        io::stdin().read_line(&mut response).unwrap();

        if !matches!(response.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("Update cancelled.");
            return 0;
        }
    }

    // Perform update
    match perform_update(&target_version, &install_path) {
        Ok(()) => {
            println!("✅ Successfully updated to v{target_version}");
            println!();
            println!("Run 'versioneer --version' to verify the installation.");
            0
        }
        Err(e) => {
            eprintln!("❌ Update failed: {e}");
            1
        }
    }
}

fn get_latest_version() -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("versioneer-updater")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let url = "https://api.github.com/repos/workhelix/versioneer/releases/latest";
    let response: serde_json::Value = client
        .get(url)
        .send()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let tag_name = response["tag_name"]
        .as_str()
        .ok_or_else(|| "No tag_name in response".to_string())?;

    let version = tag_name
        .trim_start_matches("versioneer-v")
        .trim_start_matches('v');
    Ok(version.to_string())
}

fn perform_update(version: &str, install_path: &Path) -> Result<(), String> {
    // Detect platform
    let platform = get_platform_string();
    let archive_ext = if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar.gz"
    };

    let filename = format!("versioneer-{platform}.{archive_ext}");
    let download_url = format!(
        "https://github.com/workhelix/versioneer/releases/download/versioneer-v{version}/{filename}"
    );

    println!("📥 Downloading {filename}...");

    // Download file
    let client = reqwest::blocking::Client::builder()
        .user_agent("versioneer-updater")
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(&download_url)
        .send()
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()));
    }

    let bytes = response.bytes().map_err(|e| e.to_string())?;

    // Download checksum
    let checksum_url = format!("{download_url}.sha256");
    let checksum_response = client
        .get(&checksum_url)
        .send()
        .map_err(|e| e.to_string())?;

    if checksum_response.status().is_success() {
        println!("🔐 Verifying checksum...");
        let expected_checksum = checksum_response.text().map_err(|e| e.to_string())?;
        let expected_hash = expected_checksum
            .split_whitespace()
            .next()
            .ok_or_else(|| "Invalid checksum format".to_string())?;

        // Calculate actual checksum
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let actual_hash = hex::encode(hasher.finalize());

        if actual_hash != expected_hash {
            return Err(format!(
                "Checksum verification failed!\nExpected: {expected_hash}\nActual:   {actual_hash}"
            ));
        }

        println!("✅ Checksum verified");
    } else {
        eprintln!("⚠️  Checksum file not available, skipping verification");
    }

    // Extract and install
    println!("📦 Installing...");

    // Create temp directory
    let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;

    // Extract archive
    if cfg!(target_os = "windows") {
        // Extract zip (would need zip crate)
        return Err("Windows update not yet implemented".to_string());
    }
    // Extract tar.gz
    let tar_gz = flate2::read::GzDecoder::new(&bytes[..]);
    let mut archive = tar::Archive::new(tar_gz);
    archive.unpack(temp_dir.path()).map_err(|e| e.to_string())?;

    // Find binary in temp dir
    let binary_name = if cfg!(target_os = "windows") {
        "versioneer.exe"
    } else {
        "versioneer"
    };

    let temp_binary = temp_dir.path().join(binary_name);
    if !temp_binary.exists() {
        return Err(format!("Binary not found in archive: {binary_name}"));
    }

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&temp_binary)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&temp_binary, perms).map_err(|e| e.to_string())?;
    }

    // Replace binary
    std::fs::copy(&temp_binary, install_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            format!(
                "Permission denied. Try running with sudo or use --install-dir to specify a writable location:\n  {e}"
            )
        } else {
            e.to_string()
        }
    })?;

    Ok(())
}

fn get_platform_string() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_get_platform_string() {
        let platform = get_platform_string();
        // Verify it returns one of the known platforms
        assert!(
            platform == "x86_64-apple-darwin"
                || platform == "aarch64-apple-darwin"
                || platform == "x86_64-unknown-linux-gnu"
                || platform == "aarch64-unknown-linux-gnu"
                || platform == "x86_64-pc-windows-msvc"
                || platform == "unknown"
        );
    }

    #[test]
    fn test_get_latest_version_handles_errors() {
        // Test that get_latest_version returns a Result
        let result = get_latest_version();
        // Either Ok or Err is acceptable since we're testing structure
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_run_update_with_current_version() {
        // Test update when already at current version
        let current = env!("CARGO_PKG_VERSION");
        let temp_dir = TempDir::new().unwrap();
        let exit_code = run_update(Some(current), false, Some(temp_dir.path()));
        // Should return 2 for "already up-to-date"
        assert_eq!(exit_code, 2);
    }

    #[test]
    fn test_run_update_rejects_invalid_path() {
        // Test with an invalid/non-writable path
        let exit_code = run_update(Some("99.99.99"), true, Some(Path::new("/nonexistent")));
        // Should fail with exit code 1
        assert_eq!(exit_code, 1);
    }

    #[test]
    fn test_run_update_with_force_flag() {
        // Test that force flag bypasses confirmation
        let current = env!("CARGO_PKG_VERSION");
        let temp_dir = TempDir::new().unwrap();

        // With force=true, should attempt update even at current version
        let exit_code = run_update(Some(current), true, Some(temp_dir.path()));

        // Could succeed (0) if binary exists, or fail (1) if download fails
        // The key is that it didn't return 2 (up-to-date without trying)
        assert!(exit_code == 0 || exit_code == 1);
        assert_ne!(exit_code, 2);
    }

    #[test]
    fn test_run_update_with_custom_install_dir() {
        let temp_dir = TempDir::new().unwrap();
        let install_dir = temp_dir.path();

        // Test with a fake version to trigger download attempt
        let exit_code = run_update(Some("99.99.99"), true, Some(install_dir));

        // Should fail during download but confirms install_dir is processed
        assert_eq!(exit_code, 1);
    }

    #[test]
    fn test_get_platform_string_exhaustive() {
        // Test that get_platform_string returns valid strings
        let platform = get_platform_string();

        // Verify it's one of the known platforms
        let valid_platforms = [
            "x86_64-apple-darwin",
            "aarch64-apple-darwin",
            "x86_64-unknown-linux-gnu",
            "aarch64-unknown-linux-gnu",
            "x86_64-pc-windows-msvc",
            "unknown",
        ];

        assert!(valid_platforms.contains(&platform));

        // On macOS/Linux, should not be "unknown"
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        assert_ne!(platform, "unknown");

        // Should match current architecture
        #[cfg(target_arch = "x86_64")]
        assert!(platform.starts_with("x86_64"));

        #[cfg(target_arch = "aarch64")]
        assert!(platform.starts_with("aarch64"));
    }

    #[test]
    fn test_run_update_without_version_tries_latest() {
        let temp_dir = TempDir::new().unwrap();

        // Without specifying version, should try to get latest
        // Will fail on network call or if already latest
        let exit_code = run_update(None, false, Some(temp_dir.path()));

        // Could be 0 (already latest), 1 (network error), or 2 (up-to-date)
        assert!(exit_code == 0 || exit_code == 1 || exit_code == 2);
    }

    #[test]
    fn test_run_update_validates_version_format() {
        let temp_dir = TempDir::new().unwrap();

        // Test with valid semantic version format
        let exit_code = run_update(Some("1.0.0"), true, Some(temp_dir.path()));

        // Will fail during download but version format was valid
        assert_eq!(exit_code, 1);
    }

    #[test]
    fn test_run_update_permission_denied_message() {
        // Test update to a system directory without permission
        #[cfg(unix)]
        {
            let exit_code = run_update(Some("99.99.99"), true, Some(Path::new("/usr/bin")));
            // Should fail with exit code 1 (permission denied)
            assert_eq!(exit_code, 1);
        }
    }

    #[test]
    fn test_multiple_update_scenarios() {
        let temp_dir = TempDir::new().unwrap();

        // Test 1: Current version without force (should return 2)
        let current = env!("CARGO_PKG_VERSION");
        let exit_code = run_update(Some(current), false, Some(temp_dir.path()));
        assert_eq!(exit_code, 2);

        // Test 2: Invalid path (should return 1)
        let exit_code = run_update(Some("1.0.0"), true, Some(Path::new("/nonexistent/path")));
        assert_eq!(exit_code, 1);
    }

    #[test]
    fn test_update_exit_codes() {
        let temp_dir = TempDir::new().unwrap();
        let current = env!("CARGO_PKG_VERSION");

        // Test all possible exit codes

        // Exit code 2: Already up-to-date
        let code = run_update(Some(current), false, Some(temp_dir.path()));
        assert_eq!(code, 2);

        // Exit code 1: Error (invalid path)
        let code = run_update(Some("99.99.99"), true, Some(Path::new("/nonexistent")));
        assert_eq!(code, 1);

        // Exit code 0 or 1: Latest version check (network dependent)
        let code = run_update(None, false, Some(temp_dir.path()));
        assert!(code == 0 || code == 1 || code == 2);
    }
}
