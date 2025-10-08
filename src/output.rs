//! Output formatting utilities for versioneer

use console::{Emoji, style};
use std::io::IsTerminal;

/// Output formatter that strips colors and emojis for non-TTY output
pub struct OutputFormatter {
    /// Whether output is going to a TTY
    is_tty: bool,
}

impl OutputFormatter {
    /// Create a new output formatter
    #[must_use]
    pub fn new() -> Self {
        Self {
            is_tty: std::io::stdout().is_terminal(),
        }
    }

    /// Format a success message with checkmark
    #[must_use]
    pub fn success(&self, msg: &str) -> String {
        if self.is_tty {
            format!("{} {}", Emoji("✨", "✓"), style(msg).green())
        } else {
            format!("✓ {msg}")
        }
    }

    /// Format an error message with X mark
    #[must_use]
    pub fn error(&self, msg: &str) -> String {
        if self.is_tty {
            format!("{} {}", Emoji("❌", "✗"), style(msg).red())
        } else {
            format!("✗ {msg}")
        }
    }

    /// Format a warning message
    #[must_use]
    pub fn warning(&self, msg: &str) -> String {
        if self.is_tty {
            format!("{} {}", Emoji("⚠️", "!"), style(msg).yellow())
        } else {
            format!("! {msg}")
        }
    }

    /// Format a version display
    #[must_use]
    pub fn version(&self, version: &str) -> String {
        if self.is_tty {
            format!(
                "{} Current version: {}",
                Emoji("📦", ""),
                style(version).cyan().bold()
            )
        } else {
            format!("Current version: {version}")
        }
    }

    /// Format build systems header
    #[must_use]
    pub fn build_systems_header(&self) -> String {
        if self.is_tty {
            format!("{} Detected build systems:", Emoji("🔍", ""))
        } else {
            "Detected build systems:".to_string()
        }
    }

    /// Format a sync status symbol
    #[must_use]
    pub fn sync_status(&self, in_sync: bool) -> String {
        if self.is_tty {
            if in_sync {
                format!("{}", style("✓").green().bold())
            } else {
                format!("{}", style("✗").red().bold())
            }
        } else if in_sync {
            "✓".to_string()
        } else {
            "✗".to_string()
        }
    }

    /// Format a git tag creation message
    #[must_use]
    pub fn git_tag(&self, tag_name: &str) -> String {
        if self.is_tty {
            format!(
                "{} Created git tag: {}",
                Emoji("🏷️", ""),
                style(tag_name).magenta().bold()
            )
        } else {
            format!("Created git tag: {tag_name}")
        }
    }
}

impl Default for OutputFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_tty_output() {
        let formatter = OutputFormatter { is_tty: false };

        assert_eq!(formatter.success("test"), "✓ test");
        assert_eq!(formatter.error("test"), "✗ test");
        assert_eq!(formatter.warning("test"), "! test");
        assert_eq!(formatter.version("1.0.0"), "Current version: 1.0.0");
    }

    #[test]
    fn test_sync_status() {
        let formatter_no_tty = OutputFormatter { is_tty: false };

        assert_eq!(formatter_no_tty.sync_status(true), "✓");
        assert_eq!(formatter_no_tty.sync_status(false), "✗");
    }
}
