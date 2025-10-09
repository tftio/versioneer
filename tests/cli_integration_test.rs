//! Integration tests for the versioneer CLI

use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn bin_path() -> std::path::PathBuf {
    std::env::current_dir()
        .expect("Failed to get current directory")
        .join("target/debug/versioneer")
}

#[test]
fn test_version_command() {
    let output = Command::new(bin_path())
        .arg("version")
        .output()
        .expect("Failed to execute version command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("versioneer"));
}

#[test]
fn test_show_command() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    fs::write(temp_dir.path().join("VERSION"), "1.2.3\n").unwrap();

    let output = Command::new(bin_path())
        .arg("show")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute show command");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "1.2.3");
}

#[test]
fn test_verify_command_in_sync() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    fs::write(temp_dir.path().join("VERSION"), "1.0.0\n").unwrap();
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();

    let output = Command::new(bin_path())
        .arg("verify")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute verify command");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("synchronized"));
}

#[test]
fn test_verify_command_out_of_sync() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    fs::write(temp_dir.path().join("VERSION"), "1.0.0\n").unwrap();
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"2.0.0\"\n",
    )
    .unwrap();

    let output = Command::new(bin_path())
        .arg("verify")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute verify command");

    assert!(!output.status.success());
}

#[test]
fn test_status_command() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    fs::write(temp_dir.path().join("VERSION"), "1.0.0\n").unwrap();
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();

    let output = Command::new(bin_path())
        .arg("status")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute status command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1.0.0"));
    assert!(stdout.contains("Cargo"));
}

#[test]
fn test_sync_command() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    fs::write(temp_dir.path().join("VERSION"), "3.1.4\n").unwrap();
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();

    let output = Command::new(bin_path())
        .arg("sync")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute sync command");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Synchronized"));

    // Verify Cargo.toml was updated
    let cargo_content = fs::read_to_string(temp_dir.path().join("Cargo.toml")).unwrap();
    assert!(cargo_content.contains(r#"version = "3.1.4""#));
}

#[test]
fn test_patch_command() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    fs::write(temp_dir.path().join("VERSION"), "1.0.0\n").unwrap();
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();

    let output = Command::new(bin_path())
        .arg("patch")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute patch command");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("1.0.1"));

    let version_content = fs::read_to_string(temp_dir.path().join("VERSION")).unwrap();
    assert_eq!(version_content.trim(), "1.0.1");
}

#[test]
fn test_minor_command() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    fs::write(temp_dir.path().join("VERSION"), "1.0.0\n").unwrap();
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();

    let output = Command::new(bin_path())
        .arg("minor")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute minor command");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("1.1.0"));

    let version_content = fs::read_to_string(temp_dir.path().join("VERSION")).unwrap();
    assert_eq!(version_content.trim(), "1.1.0");
}

#[test]
fn test_major_command() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    fs::write(temp_dir.path().join("VERSION"), "1.0.0\n").unwrap();
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();

    let output = Command::new(bin_path())
        .arg("major")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute major command");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("2.0.0"));

    let version_content = fs::read_to_string(temp_dir.path().join("VERSION")).unwrap();
    assert_eq!(version_content.trim(), "2.0.0");
}

#[test]
fn test_completions_command() {
    let output = Command::new(bin_path())
        .args(["completions", "bash"])
        .output()
        .expect("Failed to execute completions command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("versioneer"));
}

#[test]
fn test_doctor_command() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    fs::write(temp_dir.path().join("VERSION"), "1.0.0\n").unwrap();
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();

    let output = Command::new(bin_path())
        .arg("doctor")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute doctor command");

    // Doctor should succeed or warn
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("health check"));
}

#[test]
fn test_no_subcommand_with_build_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    fs::write(temp_dir.path().join("VERSION"), "1.0.0\n").unwrap();
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();

    let output = Command::new(bin_path())
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute versioneer");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1.0.0"));
}

#[test]
fn test_no_subcommand_no_build_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    fs::write(temp_dir.path().join("VERSION"), "1.0.0\n").unwrap();

    let output = Command::new(bin_path())
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute versioneer");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No build system files"));
}
