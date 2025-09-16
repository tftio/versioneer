//! Integration tests for the versioneer reset command

use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_reset_command_default_version() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    // Create initial test files
    fs::write(temp_path.join("VERSION"), "1.5.2").expect("Failed to write VERSION file");
    fs::write(
        temp_path.join("Cargo.toml"),
        r#"[package]
name = "test-project"
version = "1.5.2"
edition = "2021"
"#,
    )
    .expect("Failed to write Cargo.toml");

    // Run reset command with default version
    let binary_path = std::env::current_dir()
        .expect("Failed to get current directory")
        .join("target/debug/versioneer");
    let output = Command::new(&binary_path)
        .args(["reset"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute reset command");

    assert!(
        output.status.success(),
        "Reset command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Version reset to 0.0.0"));

    // Verify VERSION file was updated
    let version_content =
        fs::read_to_string(temp_path.join("VERSION")).expect("Failed to read VERSION file");
    assert_eq!(version_content.trim(), "0.0.0");

    // Verify Cargo.toml was updated
    let cargo_content =
        fs::read_to_string(temp_path.join("Cargo.toml")).expect("Failed to read Cargo.toml");
    assert!(cargo_content.contains(r#"version = "0.0.0""#));
}

#[test]
fn test_reset_command_specific_version() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    // Create initial test files
    fs::write(temp_path.join("VERSION"), "1.0.0").expect("Failed to write VERSION file");
    fs::write(
        temp_path.join("Cargo.toml"),
        r#"[package]
name = "test-project"
version = "1.0.0"
edition = "2021"
"#,
    )
    .expect("Failed to write Cargo.toml");

    // Run reset command with specific version
    let binary_path = std::env::current_dir()
        .expect("Failed to get current directory")
        .join("target/debug/versioneer");
    let output = Command::new(&binary_path)
        .args(["reset", "2.3.1"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute reset command");

    assert!(
        output.status.success(),
        "Reset command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Version reset to 2.3.1"));

    // Verify VERSION file was updated
    let version_content =
        fs::read_to_string(temp_path.join("VERSION")).expect("Failed to read VERSION file");
    assert_eq!(version_content.trim(), "2.3.1");

    // Verify Cargo.toml was updated
    let cargo_content =
        fs::read_to_string(temp_path.join("Cargo.toml")).expect("Failed to read Cargo.toml");
    assert!(cargo_content.contains(r#"version = "2.3.1""#));
}

#[test]
fn test_reset_command_invalid_version() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    // Create initial test files
    fs::write(temp_path.join("VERSION"), "1.0.0").expect("Failed to write VERSION file");

    // Run reset command with invalid version
    let binary_path = std::env::current_dir()
        .expect("Failed to get current directory")
        .join("target/debug/versioneer");
    let output = Command::new(&binary_path)
        .args(["reset", "invalid-version"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute reset command");

    assert!(!output.status.success(), "Reset command should have failed");
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid semantic version format"));

    // Verify VERSION file was not changed
    let version_content =
        fs::read_to_string(temp_path.join("VERSION")).expect("Failed to read VERSION file");
    assert_eq!(version_content.trim(), "1.0.0");
}
