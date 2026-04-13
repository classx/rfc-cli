use std::env;
use std::path::PathBuf;

/// Determines the project root directory.
/// Uses RFC_HOME environment variable if set, otherwise uses current working directory.
pub fn get_project_root() -> Result<PathBuf, String> {
    if let Ok(home) = env::var("RFC_HOME") {
        let path = PathBuf::from(home);
        if path.is_dir() {
            Ok(path)
        } else {
            Err(format!(
                "RFC_HOME points to non-existent directory: {}",
                path.display()
            ))
        }
    } else {
        env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))
    }
}
