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
    }
}
