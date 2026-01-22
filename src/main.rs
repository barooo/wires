use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "wr")]
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
        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,
    },
    /// Show wire details
    Show {
        /// Wire ID
        id: String,
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
        /// New status
        #[arg(long)]
        status: Option<String>,
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
    Ready,
    /// Delete a wire and its dependencies
    Rm {
        /// Wire ID
        id: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => commands::init::run(),
        Commands::New {
            title,
            description,
            priority,
        } => commands::new::run(&title, description.as_deref(), priority),
        Commands::List { status } => commands::list::run(status.as_deref()),
        Commands::Show { id } => commands::show::run(&id),
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
            status.as_deref(),
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
        Commands::Ready => commands::ready::run(),
        Commands::Rm { id } => commands::rm::run(&id),
    }
}
