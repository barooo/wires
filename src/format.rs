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

use clap::ValueEnum;
use owo_colors::{OwoColorize, Stream};
use std::io::{self, IsTerminal};

/// Output format options.
///
/// The format determines how wires are displayed to the user.
/// Implements [`ValueEnum`] for direct use with clap CLI arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// JSON output for programmatic parsing
    Json,
    /// Human-readable table format
    Table,
}

impl Format {
    /// Returns the appropriate format based on an optional override and TTY detection.
    ///
    /// # Arguments
    ///
    /// * `format` - Optional format override from CLI
    ///
    /// # Returns
    ///
    /// - `Some(format)` → uses the specified format
    /// - `None` → auto-detects based on stdout TTY status (table for TTY, json for pipes)
    pub fn resolve(format: Option<Format>) -> Self {
        format.unwrap_or_else(|| {
            if io::stdout().is_terminal() {
                Format::Table
            } else {
                Format::Json
            }
        })
    }
}

/// Returns a colored status symbol for terminal display.
///
/// Colors are applied when stdout is a TTY and the terminal supports colors.
/// The symbol is always returned, but colors are only applied in appropriate contexts.
fn format_status_symbol(status: crate::models::Status) -> String {
    use crate::models::Status;

    let symbol = status.symbol();

    match status {
        Status::Done => symbol
            .if_supports_color(Stream::Stdout, |text| text.green())
            .to_string(),
        Status::InProgress => symbol
            .if_supports_color(Stream::Stdout, |text| text.yellow())
            .to_string(),
        Status::Todo => symbol.to_string(),
        Status::Cancelled => symbol
            .if_supports_color(Stream::Stdout, |text| text.red())
            .to_string(),
    }
}

/// Formats a list of wires as a table.
///
/// The table includes status symbol, ID, title, and optional blocker info.
/// Returns "No wires found." if the list is empty.
pub fn format_wire_table(wires: &[crate::models::WireWithDeps]) -> String {
    if wires.is_empty() {
        return String::from("No wires found.");
    }

    let mut output = String::new();

    // No header - symbols are self-explanatory

    // Rows
    for wire_with_deps in wires {
        let wire = &wire_with_deps.wire;
        let symbol = format_status_symbol(wire.status);

        // Base line: symbol + id + title
        output.push_str(&format!("{} {}  {}", symbol, wire.id.as_str(), wire.title));

        // Add blocker suffix if this wire has blocking dependencies
        let blocker_ids: Vec<_> = wire_with_deps
            .depends_on
            .iter()
            .filter(|dep| dep.status.is_blocking())
            .map(|dep| dep.id.as_str())
            .collect();

        if !blocker_ids.is_empty() {
            output.push_str(&format!("  ← blocked by {}", blocker_ids.join(", ")));
        }

        output.push('\n');
    }

    output
}

/// Formats a wire's details with a compact header.
///
/// Shows a single-line header with symbol, ID, title, and priority,
/// followed by description and dependency information.
pub fn format_wire_detail_table(wire: &crate::models::WireWithDeps) -> String {
    let mut output = String::new();

    let symbol = format_status_symbol(wire.wire.status);

    // Compact header: symbol + id + title + [pri:N]
    output.push_str(&format!(
        "{} {}  {}  [pri:{}]\n",
        symbol, wire.wire.id.as_str(), wire.wire.title, wire.wire.priority
    ));

    // Description (if present)
    if let Some(ref desc) = wire.wire.description {
        output.push('\n');
        output.push_str(desc);
        output.push('\n');
    }

    // Dependencies
    if !wire.depends_on.is_empty() {
        output.push_str("\nDepends on:\n");
        for dep in &wire.depends_on {
            let dep_symbol = format_status_symbol(dep.status);
            output.push_str(&format!(
                "  {} {}  {}\n",
                dep_symbol,
                dep.id.as_str(),
                dep.title
            ));
        }
    }

    // Blocks
    if !wire.blocks.is_empty() {
        output.push_str("\nBlocks:\n");
        for blocker in &wire.blocks {
            let blocker_symbol = format_status_symbol(blocker.status);
            output.push_str(&format!(
                "  {} {}  {}\n",
                blocker_symbol,
                blocker.id.as_str(),
                blocker.title
            ));
        }
    }

    output
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DependencyInfo, Status, Wire, WireId, WireWithDeps};

    fn make_test_wire(id: &str, title: &str, status: Status) -> Wire {
        Wire {
            id: WireId::new(id).unwrap(),
            title: title.to_string(),
            description: None,
            status,
            created_at: 0,
            updated_at: 0,
            priority: 0,
        }
    }

    fn make_test_dep(id: &str, title: &str, status: Status) -> DependencyInfo {
        DependencyInfo {
            id: WireId::new(id).unwrap(),
            title: title.to_string(),
            status,
        }
    }

    #[test]
    fn test_format_status_symbol_contains_symbols() {
        // Just verify symbols are present (colors are TTY-dependent)
        assert!(format_status_symbol(Status::Done).contains(Status::Done.symbol()));
        assert!(format_status_symbol(Status::InProgress).contains(Status::InProgress.symbol()));
        assert!(format_status_symbol(Status::Todo).contains(Status::Todo.symbol()));
        assert!(format_status_symbol(Status::Cancelled).contains(Status::Cancelled.symbol()));
    }

    #[test]
    fn test_format_wire_table_empty() {
        let wires = vec![];
        let output = format_wire_table(&wires);
        assert_eq!(output, "No wires found.");
    }

    #[test]
    fn test_format_wire_table_single_wire() {
        let wire = make_test_wire("a1b2c3d", "Test wire", Status::Todo);
        let wire_with_deps = WireWithDeps {
            wire,
            depends_on: vec![],
            blocks: vec![],
        };
        let output = format_wire_table(&[wire_with_deps]);

        assert!(output.contains("a1b2c3d"));
        assert!(output.contains("Test wire"));
        assert!(output.contains(Status::Todo.symbol()));
    }

    #[test]
    fn test_format_wire_table_shows_blockers() {
        let wire = make_test_wire("a1b2c3d", "Blocked wire", Status::Todo);
        let dep = make_test_dep("b2c3d4e", "Blocker", Status::InProgress);
        let wire_with_deps = WireWithDeps {
            wire,
            depends_on: vec![dep],
            blocks: vec![],
        };
        let output = format_wire_table(&[wire_with_deps]);

        assert!(output.contains("Blocked wire"));
        assert!(output.contains("← blocked by b2c3d4e"));
    }

    #[test]
    fn test_format_wire_table_no_blocker_for_done_deps() {
        let wire = make_test_wire("a1b2c3d", "Unblocked wire", Status::Todo);
        let dep = make_test_dep("b2c3d4e", "Done blocker", Status::Done);
        let wire_with_deps = WireWithDeps {
            wire,
            depends_on: vec![dep],
            blocks: vec![],
        };
        let output = format_wire_table(&[wire_with_deps]);

        assert!(output.contains("Unblocked wire"));
        assert!(!output.contains("← blocked by"));
    }

    #[test]
    fn test_format_wire_table_no_blocker_for_cancelled_deps() {
        let wire = make_test_wire("a1b2c3d", "Unblocked wire", Status::Todo);
        let dep = make_test_dep("b2c3d4e", "Cancelled blocker", Status::Cancelled);
        let wire_with_deps = WireWithDeps {
            wire,
            depends_on: vec![dep],
            blocks: vec![],
        };
        let output = format_wire_table(&[wire_with_deps]);

        assert!(output.contains("Unblocked wire"));
        assert!(!output.contains("← blocked by"));
    }

    #[test]
    fn test_format_wire_table_multiple_blockers() {
        let wire = make_test_wire("a1b2c3d", "Multi-blocked", Status::Todo);
        let dep1 = make_test_dep("b2c3d4e", "Blocker 1", Status::Todo);
        let dep2 = make_test_dep("c3d4e5f", "Blocker 2", Status::InProgress);
        let wire_with_deps = WireWithDeps {
            wire,
            depends_on: vec![dep1, dep2],
            blocks: vec![],
        };
        let output = format_wire_table(&[wire_with_deps]);

        assert!(output.contains("← blocked by b2c3d4e, c3d4e5f"));
    }

    #[test]
    fn test_format_wire_detail_table_compact_header() {
        let wire = make_test_wire("a1b2c3d", "Test wire", Status::InProgress);
        let wire_with_deps = WireWithDeps {
            wire: Wire {
                priority: 2,
                ..wire
            },
            depends_on: vec![],
            blocks: vec![],
        };
        let output = format_wire_detail_table(&wire_with_deps);

        // Should have compact header with symbol, id, title, priority
        assert!(output.contains("a1b2c3d"));
        assert!(output.contains("Test wire"));
        assert!(output.contains("[pri:2]"));
        assert!(output.contains(Status::InProgress.symbol()));
    }

    #[test]
    fn test_format_wire_detail_table_with_description() {
        let wire = Wire {
            description: Some("Test description".to_string()),
            ..make_test_wire("a1b2c3d", "Test wire", Status::Todo)
        };
        let wire_with_deps = WireWithDeps {
            wire,
            depends_on: vec![],
            blocks: vec![],
        };
        let output = format_wire_detail_table(&wire_with_deps);

        assert!(output.contains("Test description"));
    }

    #[test]
    fn test_format_wire_detail_table_with_dependencies() {
        let wire = make_test_wire("a1b2c3d", "Test wire", Status::Todo);
        let dep = make_test_dep("b2c3d4e", "Dependency", Status::Done);
        let wire_with_deps = WireWithDeps {
            wire,
            depends_on: vec![dep],
            blocks: vec![],
        };
        let output = format_wire_detail_table(&wire_with_deps);

        assert!(output.contains("Depends on:"));
        assert!(output.contains("b2c3d4e"));
        assert!(output.contains("Dependency"));
        assert!(output.contains(Status::Done.symbol()));
    }

    #[test]
    fn test_format_wire_detail_table_with_blocks() {
        let wire = make_test_wire("a1b2c3d", "Test wire", Status::Done);
        let blocker = make_test_dep("b2c3d4e", "Blocked task", Status::Todo);
        let wire_with_deps = WireWithDeps {
            wire,
            depends_on: vec![],
            blocks: vec![blocker],
        };
        let output = format_wire_detail_table(&wire_with_deps);

        assert!(output.contains("Blocks:"));
        assert!(output.contains("b2c3d4e"));
        assert!(output.contains("Blocked task"));
    }
}
