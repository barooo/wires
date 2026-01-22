use std::io::{self, IsTerminal};

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Table,
}

impl Format {
    /// Parse format string, defaulting based on TTY detection
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

/// Format a list of wires as a table
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

/// Format wire details as a table
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

/// Print output in the specified format
pub fn print_json<T: serde::Serialize>(data: &T) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string(data)?);
    Ok(())
}
