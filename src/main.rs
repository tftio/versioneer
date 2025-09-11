//! Versioneer CLI - A tool to synchronize VERSION files with build system version declarations

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::env;
use versioneer::{BumpType, VersionManager};

#[derive(Parser)]
#[command(name = "versioneer")]
#[command(about = "A tool to synchronize VERSION files with build system version declarations")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Bump the major version (x.y.z -> (x+1).0.0)
    Major,
    /// Bump the minor version (x.y.z -> x.(y+1).0)
    Minor,
    /// Bump the patch version (x.y.z -> x.y.(z+1))
    Patch,
    /// Show the current version
    Show,
    /// Synchronize all version files to match the VERSION file
    Sync,
    /// Show which build systems are detected
    Status,
    /// Verify that all version files are synchronized
    Verify,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let current_dir = env::current_dir().context("Failed to get current directory")?;

    let manager = VersionManager::new(current_dir);

    match cli.command {
        None => {
            // No subcommand provided - show status if build system files exist, otherwise error
            let build_systems = manager.detect_build_systems();
            if build_systems.is_empty() {
                eprintln!("Error: No build system files (Cargo.toml or pyproject.toml) found in current directory.");
                eprintln!("Versioneer requires at least one build system file to manage versions.");
                std::process::exit(1);
            } else {
                // Show status
                let version = manager
                    .read_version_file()
                    .context("Failed to read VERSION file")?;
                println!("Current version: {version}");

                println!("Detected build systems:");
                for system in &build_systems {
                    match manager.read_build_system_version(system) {
                        Ok(sys_version) => {
                            let status = if sys_version == version { "✓" } else { "✗" };
                            println!("  {system:?}: {sys_version} {status}");
                        }
                        Err(e) => {
                            println!("  {system:?}: Error reading version: {e}");
                        }
                    }
                }
            }
        }
        Some(command) => match command {
            Commands::Major => {
                manager
                    .bump_version(BumpType::Major)
                    .context("Failed to bump major version")?;
                let new_version = manager.read_version_file()?;
                println!("Bumped to version {new_version}");
            }
            Commands::Minor => {
                manager
                    .bump_version(BumpType::Minor)
                    .context("Failed to bump minor version")?;
                let new_version = manager.read_version_file()?;
                println!("Bumped to version {new_version}");
            }
            Commands::Patch => {
                manager
                    .bump_version(BumpType::Patch)
                    .context("Failed to bump patch version")?;
                let new_version = manager.read_version_file()?;
                println!("Bumped to version {new_version}");
            }
            Commands::Show => {
                let version = manager
                    .read_version_file()
                    .context("Failed to read VERSION file")?;
                println!("{version}");
            }
            Commands::Sync => {
                manager
                    .sync_versions()
                    .context("Failed to synchronize versions")?;
                let version = manager.read_version_file()?;
                println!("Synchronized all files to version {version}");
            }
            Commands::Status => {
                let version = manager
                    .read_version_file()
                    .context("Failed to read VERSION file")?;
                println!("Current version: {version}");

                let build_systems = manager.detect_build_systems();
                if build_systems.is_empty() {
                    println!("No build system files detected");
                } else {
                    println!("Detected build systems:");
                    for system in &build_systems {
                        match manager.read_build_system_version(system) {
                            Ok(sys_version) => {
                                let status = if sys_version == version { "✓" } else { "✗" };
                                println!("  {system:?}: {sys_version} {status}");
                            }
                            Err(e) => {
                                println!("  {system:?}: Error reading version: {e}");
                            }
                        }
                    }
                }
            }
            Commands::Verify => match manager.verify_versions_in_sync() {
                Ok(()) => {
                    println!("✓ All version files are synchronized");
                }
                Err(e) => {
                    eprintln!("✗ {e}");
                    std::process::exit(1);
                }
            },
        },
    }

    Ok(())
}
