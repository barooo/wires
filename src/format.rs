//! Output formatting utilities.
//!
//! This module handles output formatting for wires, supporting both:
//! - **JSON** - Machine-readable format for programmatic use
//! - **Table** - Human-readable format for terminal display
//!
//! The format is auto-detected based on whether stdout is a TTY:
//! - TTY → table format
//! - Piped/redirected → JSON format
//!
//! Users can override with `--format json` or `--format table`.

use std::io::{self, IsTerminal};

/// Output format options.
///
/// The format determines how wires are displayed to the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// JSON output for programmatic parsing
    Json,
    /// Human-readable table format
    Table,
}

impl Format {
    /// Parses a format string, defaulting based on TTY detection.
    ///
    /// # Arguments
    ///
    /// * `s` - Optional format string ("json" or "table")
    ///
    /// # Returns
    ///
    /// - `Some("json")` → `Format::Json`
    /// - `Some("table")` → `Format::Table`
    /// - `None` → Auto-detect based on stdout TTY status
    ///
    /// # Errors
    ///
    /// Returns an error for unrecognized format strings.
    pub fn from_str_or_auto(s: Option<&str>) -> Result<Self, String> {
        match s {
            Some("json") => Ok(Format::Json),
            Some("table") => Ok(Format::Table),
            Some(other) => Err(format!("Invalid format: {}. Valid: json, table", other)),
            None => {
                // Auto-detect: table for TTY, json for pipes
                if io::stdout().is_terminal() {
                    Ok(Format::Table)
                } else {
                    Ok(Format::Json)
                }
            }
        }
    }
}

/// Formats a list of wires as a table.
///
/// The table includes columns for ID, status, priority, and title.
/// Returns "No wires found." if the list is empty.
pub fn format_wire_table(wires: &[crate::models::Wire]) -> String {
    if wires.is_empty() {
        return String::from("No wires found.");
    }

    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "{:<8} {:<12} {:>3}  {}\n",
        "ID", "STATUS", "PRI", "TITLE"
    ));
    output.push_str(&"-".repeat(60));
    output.push('\n');

    // Rows
    for wire in wires {
        output.push_str(&format!(
            "{:<8} {:<12} {:>3}  {}\n",
            &wire.id[..7.min(wire.id.len())],
            wire.status.as_str(),
            wire.priority,
            truncate(&wire.title, 40)
        ));
    }

    output
}

/// Formats a wire's details as a table.
///
/// Includes all wire fields plus dependency relationships.
pub fn format_wire_detail_table(wire: &crate::models::WireWithDeps) -> String {
    let mut output = String::new();

    output.push_str(&format!("ID:          {}\n", wire.wire.id));
    output.push_str(&format!("Title:       {}\n", wire.wire.title));
    output.push_str(&format!("Status:      {}\n", wire.wire.status.as_str()));
    output.push_str(&format!("Priority:    {}\n", wire.wire.priority));

    if let Some(ref desc) = wire.wire.description {
        output.push_str(&format!("Description: {}\n", desc));
    }

    if !wire.depends_on.is_empty() {
        output.push_str("\nDepends on:\n");
        for dep in &wire.depends_on {
            output.push_str(&format!(
                "  {} ({}) - {}\n",
                &dep.id[..7.min(dep.id.len())],
                dep.status.as_str(),
                dep.title
            ));
        }
    }

    if !wire.blocks.is_empty() {
        output.push_str("\nBlocks:\n");
        for blocker in &wire.blocks {
            output.push_str(&format!(
                "  {} ({}) - {}\n",
                &blocker.id[..7.min(blocker.id.len())],
                blocker.status.as_str(),
                blocker.title
            ));
        }
    }

    output
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Prints data as JSON to stdout.
///
/// # Arguments
///
/// * `data` - Any serializable data
///
/// # Errors
///
/// Returns an error if JSON serialization fails.
pub fn print_json<T: serde::Serialize>(data: &T) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string(data)?);
    Ok(())
}
