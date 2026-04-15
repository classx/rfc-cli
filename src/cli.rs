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
    /// Вывод списка всех RFC
    List {
        /// Фильтр по статусу
        #[arg(long)]
        status: Option<String>,
    },
    /// Просмотр содержимого RFC
    View {
        /// Номер RFC
        number: String,
    },
    /// Показать статус RFC
    Status {
        /// Номер RFC
        number: String,
    },
    /// Открыть RFC в редакторе
    Edit {
        /// Номер RFC
        number: String,
        /// Разрешить редактирование accepted/implemented RFC
        #[arg(long)]
        force: bool,
    },
    /// Изменить статус RFC
    Set {
        /// Номер RFC
        number: String,
        /// Целевой статус
        status: String,
        /// Замещающий RFC (для перехода в superseded)
        #[arg(long)]
        by: Option<String>,
    },
    /// Проверить валидность RFC
    Check {
        /// Номер RFC (если не указан — проверить все)
        number: Option<String>,
    },
    /// Полностью перестроить индекс
    Reindex,
    /// Связать RFC с файлом исходного кода
    Link {
        /// Номер RFC
        number: String,
        /// Путь к файлу (относительно корня проекта)
        path: String,
        /// Разрешить изменение accepted/implemented RFC
        #[arg(long)]
        force: bool,
    },
    /// Убрать связь RFC с файлом
    Unlink {
        /// Номер RFC
        number: String,
        /// Путь к файлу
        path: String,
        /// Разрешить изменение accepted/implemented RFC
        #[arg(long)]
        force: bool,
    },
    /// Показать дерево зависимостей RFC
    Deps {
        /// Номер RFC
        number: String,
        /// Показать обратные зависимости (кто зависит от данного RFC)
        #[arg(long)]
        reverse: bool,
    },
}
