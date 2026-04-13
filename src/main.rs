mod cli;
mod commands;
mod rfclib;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    let project_root = match rfclib::project::get_project_root() {
        Ok(root) => root,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let result = match cli.command {
        Commands::Init => commands::init::execute(&project_root),
        Commands::New { title } => commands::new::execute(&project_root, &title),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
