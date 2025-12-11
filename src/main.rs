//! Versioneer CLI - A tool to synchronize VERSION files with build system version declarations

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::env;
use versioneer::{BumpType, VersionManager, output::OutputFormatter};
use workhelix_cli_common::LicenseType;

mod doctor;

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
    /// Show version information
    Version,
    /// Show license information
    License,
    /// Bump the major version (x.y.z -> (x+1).0.0)
    Major {
        /// Update all manifests in subdirectories recursively
        #[arg(long)]
        cascade: bool,
        /// Preview changes without writing files (requires --cascade)
        #[arg(long)]
        dry_run: bool,
        /// Suppress output (only show errors)
        #[arg(long, short)]
        quiet: bool,
    },
    /// Bump the minor version (x.y.z -> x.(y+1).0)
    Minor {
        /// Update all manifests in subdirectories recursively
        #[arg(long)]
        cascade: bool,
        /// Preview changes without writing files (requires --cascade)
        #[arg(long)]
        dry_run: bool,
        /// Suppress output (only show errors)
        #[arg(long, short)]
        quiet: bool,
    },
    /// Bump the patch version (x.y.z -> x.y.(z+1))
    Patch {
        /// Update all manifests in subdirectories recursively
        #[arg(long)]
        cascade: bool,
        /// Preview changes without writing files (requires --cascade)
        #[arg(long)]
        dry_run: bool,
        /// Suppress output (only show errors)
        #[arg(long, short)]
        quiet: bool,
    },
    /// Show the current version
    Show,
    /// Synchronize all version files to match the VERSION file
    Sync {
        /// Update all manifests in subdirectories recursively
        #[arg(long)]
        cascade: bool,
        /// Preview changes without writing files (requires --cascade)
        #[arg(long)]
        dry_run: bool,
        /// Suppress output (only show errors)
        #[arg(long, short)]
        quiet: bool,
    },
    /// Show which build systems are detected
    Status,
    /// Verify that all version files are synchronized
    Verify,
    /// Reset the version to a specific version or 0.0.0
    Reset {
        /// The version to reset to (default: 0.0.0)
        version: Option<String>,
        /// Update all manifests in subdirectories recursively
        #[arg(long)]
        cascade: bool,
        /// Preview changes without writing files (requires --cascade)
        #[arg(long)]
        dry_run: bool,
        /// Suppress output (only show errors)
        #[arg(long, short)]
        quiet: bool,
    },
    /// Generate shell completion scripts
    Completions {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
    },
    /// Check health and configuration
    Doctor,
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    let cli = Cli::parse();

    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let formatter = OutputFormatter::new();
    let manager = VersionManager::new(current_dir);

    match cli.command {
        None => {
            // No subcommand provided - show status if build system files exist, otherwise error
            let build_systems = manager.detect_build_systems();
            if build_systems.is_empty() {
                eprintln!("{}", formatter.error("No build system files (Cargo.toml or pyproject.toml) found in current directory."));
                eprintln!("Versioneer requires at least one build system file to manage versions.");
                std::process::exit(1);
            } else {
                // Show status
                let version = manager
                    .read_version_file()
                    .context("Failed to read VERSION file")?;
                println!("{}", formatter.version(&version.to_string()));

                println!("\n{}", formatter.build_systems_header());
                for system in &build_systems {
                    match manager.read_build_system_version(system) {
                        Ok(sys_version) => {
                            let status = formatter.sync_status(sys_version == version);
                            println!("  {system:?}: {sys_version} {status}");
                        }
                        Err(e) => {
                            eprintln!(
                                "{}",
                                formatter
                                    .error(&format!("  {system:?}: Error reading version: {e}"))
                            );
                        }
                    }
                }
            }
        }
        Some(command) => match command {
            Commands::Version => {
                println!("versioneer {}", env!("CARGO_PKG_VERSION"));
            }
            Commands::License => {
                println!(
                    "{}",
                    workhelix_cli_common::license::display_license("versioneer", LicenseType::MIT)
                );
            }
            Commands::Major {
                cascade,
                dry_run,
                quiet,
            } => {
                if dry_run && !cascade {
                    eprintln!("{}", formatter.error("--dry-run requires --cascade"));
                    std::process::exit(1);
                }

                if dry_run {
                    let changes = manager
                        .bump_cascade_dry_run(BumpType::Major)
                        .context("Failed to preview major version bump")?;
                    if !quiet {
                        println!(
                            "{}",
                            formatter
                                .success(&format!("Would bump to version {}", changes.new_version))
                        );
                        println!("\nFiles to update:");
                        for file in &changes.files_to_update {
                            println!("  {}", file.display());
                        }
                    }
                } else if cascade {
                    manager
                        .bump_cascade(BumpType::Major)
                        .context("Failed to bump major version")?;
                    if !quiet {
                        let new_version = manager.read_version_file()?;
                        println!(
                            "{}",
                            formatter.success(&format!("Bumped to version {new_version}"))
                        );
                    }
                } else {
                    manager
                        .bump_version(BumpType::Major)
                        .context("Failed to bump major version")?;
                    if !quiet {
                        let new_version = manager.read_version_file()?;
                        println!(
                            "{}",
                            formatter.success(&format!("Bumped to version {new_version}"))
                        );
                    }
                }
            }
            Commands::Minor {
                cascade,
                dry_run,
                quiet,
            } => {
                if dry_run && !cascade {
                    eprintln!("{}", formatter.error("--dry-run requires --cascade"));
                    std::process::exit(1);
                }

                if dry_run {
                    let changes = manager
                        .bump_cascade_dry_run(BumpType::Minor)
                        .context("Failed to preview minor version bump")?;
                    if !quiet {
                        println!(
                            "{}",
                            formatter
                                .success(&format!("Would bump to version {}", changes.new_version))
                        );
                        println!("\nFiles to update:");
                        for file in &changes.files_to_update {
                            println!("  {}", file.display());
                        }
                    }
                } else if cascade {
                    manager
                        .bump_cascade(BumpType::Minor)
                        .context("Failed to bump minor version")?;
                    if !quiet {
                        let new_version = manager.read_version_file()?;
                        println!(
                            "{}",
                            formatter.success(&format!("Bumped to version {new_version}"))
                        );
                    }
                } else {
                    manager
                        .bump_version(BumpType::Minor)
                        .context("Failed to bump minor version")?;
                    if !quiet {
                        let new_version = manager.read_version_file()?;
                        println!(
                            "{}",
                            formatter.success(&format!("Bumped to version {new_version}"))
                        );
                    }
                }
            }
            Commands::Patch {
                cascade,
                dry_run,
                quiet,
            } => {
                if dry_run && !cascade {
                    eprintln!("{}", formatter.error("--dry-run requires --cascade"));
                    std::process::exit(1);
                }

                if dry_run {
                    let changes = manager
                        .bump_cascade_dry_run(BumpType::Patch)
                        .context("Failed to preview patch version bump")?;
                    if !quiet {
                        println!(
                            "{}",
                            formatter
                                .success(&format!("Would bump to version {}", changes.new_version))
                        );
                        println!("\nFiles to update:");
                        for file in &changes.files_to_update {
                            println!("  {}", file.display());
                        }
                    }
                } else if cascade {
                    manager
                        .bump_cascade(BumpType::Patch)
                        .context("Failed to bump patch version")?;
                    if !quiet {
                        let new_version = manager.read_version_file()?;
                        println!(
                            "{}",
                            formatter.success(&format!("Bumped to version {new_version}"))
                        );
                    }
                } else {
                    manager
                        .bump_version(BumpType::Patch)
                        .context("Failed to bump patch version")?;
                    if !quiet {
                        let new_version = manager.read_version_file()?;
                        println!(
                            "{}",
                            formatter.success(&format!("Bumped to version {new_version}"))
                        );
                    }
                }
            }
            Commands::Show => {
                let version = manager
                    .read_version_file()
                    .context("Failed to read VERSION file")?;
                println!("{version}");
            }
            Commands::Sync {
                cascade,
                dry_run,
                quiet,
            } => {
                if dry_run && !cascade {
                    eprintln!("{}", formatter.error("--dry-run requires --cascade"));
                    std::process::exit(1);
                }

                if dry_run {
                    let changes = manager
                        .sync_cascade_dry_run()
                        .context("Failed to preview synchronization")?;
                    if !quiet {
                        println!(
                            "{}",
                            formatter
                                .success(&format!("Would sync to version {}", changes.new_version))
                        );
                        println!("\nFiles to update:");
                        for file in &changes.files_to_update {
                            println!("  {}", file.display());
                        }
                    }
                } else if cascade {
                    manager
                        .sync_cascade()
                        .context("Failed to synchronize versions")?;
                    if !quiet {
                        let version = manager.read_version_file()?;
                        println!(
                            "{}",
                            formatter
                                .success(&format!("Synchronized all files to version {version}"))
                        );
                    }
                } else {
                    manager
                        .sync_versions()
                        .context("Failed to synchronize versions")?;
                    if !quiet {
                        let version = manager.read_version_file()?;
                        println!(
                            "{}",
                            formatter
                                .success(&format!("Synchronized all files to version {version}"))
                        );
                    }
                }
            }
            Commands::Status => {
                let version = manager
                    .read_version_file()
                    .context("Failed to read VERSION file")?;
                println!("{}", formatter.version(&version.to_string()));

                let build_systems = manager.detect_build_systems();
                if build_systems.is_empty() {
                    println!("{}", formatter.warning("No build system files detected"));
                } else {
                    println!("\n{}", formatter.build_systems_header());
                    for system in &build_systems {
                        match manager.read_build_system_version(system) {
                            Ok(sys_version) => {
                                let status = formatter.sync_status(sys_version == version);
                                println!("  {system:?}: {sys_version} {status}");
                            }
                            Err(e) => {
                                eprintln!(
                                    "{}",
                                    formatter.error(&format!(
                                        "  {system:?}: Error reading version: {e}"
                                    ))
                                );
                            }
                        }
                    }
                }
            }
            Commands::Verify => match manager.verify_versions_in_sync() {
                Ok(()) => {
                    println!(
                        "{}",
                        formatter.success("All version files are synchronized")
                    );
                }
                Err(e) => {
                    eprintln!("{}", formatter.error(&e.to_string()));
                    std::process::exit(1);
                }
            },
            Commands::Reset {
                version,
                cascade,
                dry_run,
                quiet,
            } => {
                if dry_run && !cascade {
                    eprintln!("{}", formatter.error("--dry-run requires --cascade"));
                    std::process::exit(1);
                }

                let target_version = version.as_deref().unwrap_or("0.0.0");

                if dry_run {
                    match manager.reset_cascade_dry_run(target_version) {
                        Ok(changes) => {
                            if !quiet {
                                println!(
                                    "{}",
                                    formatter.success(&format!(
                                        "Would reset to version {}",
                                        changes.new_version
                                    ))
                                );
                                println!("\nFiles to update:");
                                for file in &changes.files_to_update {
                                    println!("  {}", file.display());
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "{}",
                                formatter.error(&format!("Failed to preview reset: {e}"))
                            );
                            std::process::exit(1);
                        }
                    }
                } else {
                    let result = if cascade {
                        manager.reset_cascade(target_version)
                    } else {
                        manager.reset_version(target_version)
                    };

                    match result {
                        Ok(()) => {
                            if !quiet {
                                println!(
                                    "{}",
                                    formatter
                                        .success(&format!("Version reset to {target_version}"))
                                );
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "{}",
                                formatter.error(&format!("Failed to reset version: {e}"))
                            );
                            std::process::exit(1);
                        }
                    }
                }
            }
            Commands::Completions { shell } => {
                workhelix_cli_common::completions::generate_completions::<Cli>(shell);
            }
            Commands::Doctor => {
                let exit_code = doctor::run_doctor(&manager);
                std::process::exit(exit_code);
            }
        },
    }

    Ok(())
}
