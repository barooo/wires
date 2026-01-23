use clap::{Parser, Subcommand};
use serde_json::json;
use std::io::IsTerminal;
use wr::format::Format;
use wr::models::Status;

mod commands;

#[derive(Parser)]
#[command(name = "wr")]
#[command(version)]
#[command(about = "Lightweight local task tracker optimized for AI coding agents", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new wires repository
    Init,
    /// Create a new wire
    New {
        /// Wire title
        title: String,
        /// Wire description
        #[arg(short, long)]
        description: Option<String>,
        /// Priority (default: 0)
        #[arg(short, long, default_value = "0")]
        priority: i32,
    },
    /// List wires
    List {
        /// Filter by status (todo, in-progress, done, cancelled)
        #[arg(short, long, value_enum)]
        status: Option<Status>,
        /// Output format (json, table). Auto-detects based on TTY.
        #[arg(short, long, value_enum)]
        format: Option<Format>,
    },
    /// Show wire details
    Show {
        /// Wire ID
        id: String,
        /// Output format (json, table). Auto-detects based on TTY.
        #[arg(short, long, value_enum)]
        format: Option<Format>,
    },
    /// Update wire fields
    Update {
        /// Wire ID
        id: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// New status (todo, in-progress, done, cancelled)
        #[arg(long, value_enum)]
        status: Option<Status>,
        /// New priority
        #[arg(long)]
        priority: Option<i32>,
    },
    /// Set wire status to IN_PROGRESS
    Start {
        /// Wire ID
        id: String,
    },
    /// Set wire status to DONE
    Done {
        /// Wire ID
        id: String,
    },
    /// Set wire status to CANCELLED
    Cancel {
        /// Wire ID
        id: String,
    },
    /// Add a dependency (wire_id depends on depends_on)
    Dep {
        /// Wire ID that has the dependency
        wire_id: String,
        /// Wire ID that it depends on
        depends_on: String,
    },
    /// Remove a dependency
    Undep {
        /// Wire ID that has the dependency
        wire_id: String,
        /// Wire ID that it depends on
        depends_on: String,
    },
    /// Find wires ready to work on
    Ready {
        /// Output format (json, table). Auto-detects based on TTY.
        #[arg(short, long, value_enum)]
        format: Option<Format>,
    },
    /// Delete a wire and its dependencies
    Rm {
        /// Wire ID
        id: String,
    },
    /// Export dependency graph
    Graph {
        /// Output format (json)
        #[arg(short, long, default_value = "json")]
        format: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init => commands::init::run(),
        Commands::New {
            title,
            description,
            priority,
        } => commands::new::run(&title, description.as_deref(), priority),
        Commands::List { status, format } => commands::list::run(status, format),
        Commands::Show { id, format } => commands::show::run(&id, format),
        Commands::Update {
            id,
            title,
            description,
            status,
            priority,
        } => commands::update::run(
            &id,
            title.as_deref(),
            description.as_deref(),
            status,
            priority,
        ),
        Commands::Start { id } => commands::start::run(&id),
        Commands::Done { id } => commands::done::run(&id),
        Commands::Cancel { id } => commands::cancel::run(&id),
        Commands::Dep {
            wire_id,
            depends_on,
        } => commands::dep::run(&wire_id, &depends_on),
        Commands::Undep {
            wire_id,
            depends_on,
        } => commands::undep::run(&wire_id, &depends_on),
        Commands::Ready { format } => commands::ready::run(format),
        Commands::Rm { id } => commands::rm::run(&id),
        Commands::Graph { format } => commands::graph::run(Some(&format)),
    };

    if let Err(e) = result {
        let error_msg = e.to_string();

        if std::io::stderr().is_terminal() {
            // Human-friendly output for interactive use
            eprintln!("Error: {}", error_msg);
        } else {
            // JSON output for programmatic use
            let error_json = json!({ "error": error_msg });
            eprintln!("{}", serde_json::to_string(&error_json).unwrap());
        }

        std::process::exit(1);
    }
}
