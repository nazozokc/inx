use owo_colors::OwoColorize;

/// Color/text styling helpers.
pub struct Colors;

impl Colors {
    // Success messages — green
    pub fn success(msg: &str) -> String {
        msg.green().to_string()
    }

    // Error messages — red
    pub fn error(msg: &str) -> String {
        msg.red().to_string()
    }

    // Warning messages — yellow
    pub fn warning(msg: &str) -> String {
        msg.yellow().to_string()
    }

    // Accent (package names, version) — bold magenta
    pub fn accent(msg: &str) -> String {
        msg.magenta().bold().to_string()
    }

    // Dim text (labels, secondary info)
    pub fn dim(msg: &str) -> String {
        msg.dimmed().to_string()
    }

    // Bold
    pub fn bold(msg: &str) -> String {
        msg.bold().to_string()
    }
}

// Icon helpers
pub const ICON_SUCCESS: &str = "✨";
pub const ICON_ERROR: &str = "✖";
pub const ICON_WARNING: &str = "⚠";
pub const ICON_INFO: &str = "ℹ";
pub const ICON_INSTALL: &str = "📦";
pub const ICON_REMOVE: &str = "🗑";
pub const ICON_SEARCH: &str = "🔍";
pub const ICON_SYNC: &str = "⟳";
pub const ICON_CHECK: &str = "✔";