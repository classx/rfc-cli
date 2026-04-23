use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(name = "rfc-cli", about = "Manage RFC documents")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize RFC structure in the project
    Init,
    /// Create a new RFC from template
    New {
        /// RFC title
        title: String,
    },
    /// List all RFCs
    List {
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
    },
    /// View RFC contents
    View {
        /// RFC number
        number: String,
    },
    /// Show RFC status
    Status {
        /// RFC number
        number: String,
    },
    /// Open RFC in editor
    Edit {
        /// RFC number
        number: String,
        /// Allow editing accepted/implemented RFCs
        #[arg(long)]
        force: bool,
    },
    /// Change RFC status
    Set {
        /// RFC number
        number: String,
        /// Target status (one of: draft, review, accepted, implemented, superseded, deprecated)
        #[arg(value_parser = clap::builder::PossibleValuesParser::new(crate::rfclib::rfc::VALID_STATUSES))]
        status: String,
        /// Superseding RFC (for transition to superseded)
        #[arg(long)]
        by: Option<String>,
    },
    /// Validate RFC(s)
    Check {
        /// RFC number (if omitted, validate all)
        number: Option<String>,
    },
    /// Rebuild the index from scratch
    Reindex,
    /// Link RFC to a source code file
    Link {
        /// RFC number
        number: String,
        /// File path (relative to project root)
        path: String,
        /// Allow modifying accepted/implemented RFCs
        #[arg(long)]
        force: bool,
    },
    /// Remove link between RFC and a file
    Unlink {
        /// RFC number
        number: String,
        /// File path
        path: String,
        /// Allow modifying accepted/implemented RFCs
        #[arg(long)]
        force: bool,
    },
    /// Show RFC dependency tree
    Deps {
        /// RFC number
        number: String,
        /// Show reverse dependencies (which RFCs depend on this one)
        #[arg(long)]
        reverse: bool,
    },
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Run project-wide RFC health diagnostics
    Doctor {
        /// Threshold in days for detecting stale drafts (default: 30)
        #[arg(long, default_value = "30")]
        stale_days: u64,
    },
}
