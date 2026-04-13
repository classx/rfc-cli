use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rfc-cli", about = "Управление RFC-документами")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Инициализация структуры RFC в проекте
    Init,
    /// Создание нового RFC из шаблона
    New {
        /// Название RFC
        title: String,
    },
}
