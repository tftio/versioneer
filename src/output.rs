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

    #[test]
    fn test_build_systems_header() {
        let formatter_no_tty = OutputFormatter { is_tty: false };
        assert_eq!(
            formatter_no_tty.build_systems_header(),
            "Detected build systems:"
        );
    }

    #[test]
    fn test_git_tag() {
        let formatter_no_tty = OutputFormatter { is_tty: false };
        assert_eq!(
            formatter_no_tty.git_tag("v1.0.0"),
            "Created git tag: v1.0.0"
        );
    }

    #[test]
    fn test_default_formatter() {
        let formatter = OutputFormatter::default();
        // Verify default creates a formatter
        let msg = formatter.success("test");
        assert!(msg.contains("test"));
    }

    #[test]
    fn test_tty_output_contains_content() {
        // Test TTY mode still contains the message even if it adds formatting
        let formatter_tty = OutputFormatter { is_tty: true };

        let success_msg = formatter_tty.success("success test");
        assert!(success_msg.contains("success test"));

        let error_msg = formatter_tty.error("error test");
        assert!(error_msg.contains("error test"));

        let warning_msg = formatter_tty.warning("warning test");
        assert!(warning_msg.contains("warning test"));

        let version_msg = formatter_tty.version("1.2.3");
        assert!(version_msg.contains("1.2.3"));
    }

    #[test]
    fn test_special_characters_in_messages() {
        let formatter = OutputFormatter { is_tty: false };

        // Test with special characters
        assert_eq!(
            formatter.success("test with 日本語"),
            "✓ test with 日本語"
        );
        assert_eq!(
            formatter.error("error: 'quoted' \"values\""),
            "✗ error: 'quoted' \"values\""
        );
        assert_eq!(
            formatter.warning("path/to/file.txt"),
            "! path/to/file.txt"
        );
    }

    #[test]
    fn test_newlines_and_multiline() {
        let formatter = OutputFormatter { is_tty: false };

        // Test with newlines
        let msg_with_newline = formatter.success("line1\nline2");
        assert!(msg_with_newline.contains("line1"));
        assert!(msg_with_newline.contains("line2"));
    }

    #[test]
    fn test_empty_messages() {
        let formatter = OutputFormatter { is_tty: false };

        assert_eq!(formatter.success(""), "✓ ");
        assert_eq!(formatter.error(""), "✗ ");
        assert_eq!(formatter.warning(""), "! ");
        assert_eq!(formatter.version(""), "Current version: ");
    }

    #[test]
    fn test_long_messages() {
        let formatter = OutputFormatter { is_tty: false };

        let long_msg = "a".repeat(1000);
        let result = formatter.success(&long_msg);
        // Verify message is included even if very long
        assert!(result.contains(&long_msg));
        assert!(result.len() > 1000);
    }

    #[test]
    fn test_emoji_fallbacks_non_tty() {
        let formatter_no_tty = OutputFormatter { is_tty: false };

        // Verify all emojis fall back to ASCII characters in non-TTY mode
        assert!(formatter_no_tty.success("test").starts_with('✓'));
        assert!(formatter_no_tty.error("test").starts_with('✗'));
        assert!(formatter_no_tty.warning("test").starts_with('!'));
        assert!(!formatter_no_tty.build_systems_header().contains("🔍"));
        assert!(!formatter_no_tty.git_tag("v1.0.0").contains("🏷️"));
    }

    #[test]
    fn test_new_formatter_creates_valid_instance() {
        let formatter = OutputFormatter::new();
        // Verify it creates a formatter with the correct TTY detection
        let msg = formatter.success("test");
        assert!(msg.contains("test"));
    }

    #[test]
    fn test_all_output_methods_with_both_modes() {
        // Test both TTY and non-TTY modes produce valid output
        for is_tty in [true, false] {
            let formatter = OutputFormatter { is_tty };

            // All methods should produce non-empty output
            assert!(!formatter.success("msg").is_empty());
            assert!(!formatter.error("msg").is_empty());
            assert!(!formatter.warning("msg").is_empty());
            assert!(!formatter.version("1.0.0").is_empty());
            assert!(!formatter.build_systems_header().is_empty());
            assert!(!formatter.sync_status(true).is_empty());
            assert!(!formatter.sync_status(false).is_empty());
            assert!(!formatter.git_tag("v1.0.0").is_empty());
        }
    }
}
