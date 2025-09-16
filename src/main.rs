//! Versioneer CLI - A tool to synchronize VERSION files with build system version declarations

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::env;
use versioneer::{output::OutputFormatter, BumpType, VersionManager};

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
    /// Bump the major version (x.y.z -> (x+1).0.0)
    Major {
        /// Create a git tag after bumping version
        #[arg(short, long)]
        tag: bool,
        /// Custom tag format (default: {repository_name}-v{version})
        #[arg(long)]
        tag_format: Option<String>,
    },
    /// Bump the minor version (x.y.z -> x.(y+1).0)
    Minor {
        /// Create a git tag after bumping version
        #[arg(short, long)]
        tag: bool,
        /// Custom tag format (default: {repository_name}-v{version})
        #[arg(long)]
        tag_format: Option<String>,
    },
    /// Bump the patch version (x.y.z -> x.y.(z+1))
    Patch {
        /// Create a git tag after bumping version
        #[arg(short, long)]
        tag: bool,
        /// Custom tag format (default: {repository_name}-v{version})
        #[arg(long)]
        tag_format: Option<String>,
    },
    /// Show the current version
    Show,
    /// Synchronize all version files to match the VERSION file
    Sync,
    /// Show which build systems are detected
    Status,
    /// Verify that all version files are synchronized
    Verify,
    /// Create a git tag for the current version
    Tag {
        /// Custom tag format (default: {repository_name}-v{version})
        #[arg(long)]
        tag_format: Option<String>,
    },
    /// Reset the version to a specific version or 0.0.0
    Reset {
        /// The version to reset to (default: 0.0.0)
        version: Option<String>,
        /// Create a git tag after resetting version
        #[arg(short, long)]
        tag: bool,
        /// Custom tag format (default: {repository_name}-v{version})
        #[arg(long)]
        tag_format: Option<String>,
    },
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
                            println!("  {:?}: {} {}", system, sys_version, status);
                        }
                        Err(e) => {
                            eprintln!("{}", formatter.error(&format!("  {:?}: Error reading version: {}", system, e)));
                        }
                    }
                }
            }
        }
        Some(command) => match command {
            Commands::Version => {
                println!("versioneer {}", env!("CARGO_PKG_VERSION"));
            }
            Commands::Major { tag, tag_format } => {
                manager
                    .bump_version(BumpType::Major)
                    .context("Failed to bump major version")?;
                let new_version = manager.read_version_file()?;
                println!("{}", formatter.success(&format!("Bumped to version {}", new_version)));
                
                if tag {
                    handle_git_tagging(&manager, &formatter, tag_format.as_deref());
                }
            }
            Commands::Minor { tag, tag_format } => {
                manager
                    .bump_version(BumpType::Minor)
                    .context("Failed to bump minor version")?;
                let new_version = manager.read_version_file()?;
                println!("{}", formatter.success(&format!("Bumped to version {}", new_version)));
                
                if tag {
                    handle_git_tagging(&manager, &formatter, tag_format.as_deref());
                }
            }
            Commands::Patch { tag, tag_format } => {
                manager
                    .bump_version(BumpType::Patch)
                    .context("Failed to bump patch version")?;
                let new_version = manager.read_version_file()?;
                println!("{}", formatter.success(&format!("Bumped to version {}", new_version)));
                
                if tag {
                    handle_git_tagging(&manager, &formatter, tag_format.as_deref());
                }
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
                println!("{}", formatter.success(&format!("Synchronized all files to version {}", version)));
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
                                println!("  {:?}: {} {}", system, sys_version, status);
                            }
                            Err(e) => {
                                eprintln!("{}", formatter.error(&format!("  {:?}: Error reading version: {}", system, e)));
                            }
                        }
                    }
                }
            }
            Commands::Verify => match manager.verify_versions_in_sync() {
                Ok(()) => {
                    println!("{}", formatter.success("All version files are synchronized"));
                }
                Err(e) => {
                    eprintln!("{}", formatter.error(&e.to_string()));
                    std::process::exit(1);
                }
            },
            Commands::Tag { tag_format } => {
                handle_git_tagging(&manager, &formatter, tag_format.as_deref());
            }
            Commands::Reset { version, tag, tag_format } => {
                let target_version = version.as_deref().unwrap_or("0.0.0");
                
                match manager.reset_version(target_version) {
                    Ok(()) => {
                        println!("{}", formatter.success(&format!("Version reset to {}", target_version)));

                        if tag {
                            handle_git_tagging(&manager, &formatter, tag_format.as_deref());
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", formatter.error(&format!("Failed to reset version: {}", e)));
                        std::process::exit(1);
                    }
                }
            }
        },
    }

    Ok(())
}

fn handle_git_tagging(manager: &VersionManager, formatter: &OutputFormatter, tag_format: Option<&str>) {
    if !manager.is_git_repository() {
        println!("{}", formatter.warning("Not in a git repository, skipping tag creation"));
        return;
    }

    match manager.create_git_tag(tag_format) {
        Ok(tag_name) => {
            println!("{}", formatter.git_tag(&tag_name));
        }
        Err(e) => {
            eprintln!("{}", formatter.error(&format!("Failed to create git tag: {}", e)));
            std::process::exit(1);
        }
    }
}
