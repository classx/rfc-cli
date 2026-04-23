mod cli;
mod commands;
mod rfclib;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    if let Commands::Completions { shell } = &cli.command {
        let mut cmd = Cli::command();
        generate(*shell, &mut cmd, "rfc-cli", &mut std::io::stdout());
        return;
    }

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
        Commands::List { status } => commands::list::execute(&project_root, status.as_deref()),
        Commands::View { number } => commands::view::execute(&project_root, &number),
        Commands::Status { number } => commands::status::execute(&project_root, &number),
        Commands::Edit { number, force } => commands::edit::execute(&project_root, &number, force),
        Commands::Set { number, status, by } => {
            commands::set::execute(&project_root, &number, &status, by.as_deref())
        }
        Commands::Check { number } => commands::check::execute(&project_root, number.as_deref()),
        Commands::Reindex => commands::reindex::execute(&project_root),
        Commands::Link {
            number,
            path,
            force,
        } => commands::link::execute(&project_root, &number, &path, force),
        Commands::Unlink {
            number,
            path,
            force,
        } => commands::unlink::execute(&project_root, &number, &path, force),
        Commands::Deps { number, reverse } => {
            commands::deps::execute(&project_root, &number, reverse)
        }
        Commands::Completions { .. } => Ok(()),
        Commands::Doctor { stale_days, drift } => {
            commands::doctor::execute(&project_root, stale_days, drift)
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
