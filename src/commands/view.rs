use std::fs;
use std::path::Path;

use crate::rfclib::rfc;

/// Executes the `view` command: prints RFC content to stdout
pub fn execute(project_root: &Path, number: &str) -> Result<(), String> {
    let normalized = rfc::normalize_number(number)?;
    let path = rfc::rfc_path(project_root, number)?;

    if !path.exists() {
        return Err(format!("RFC-{} not found.", normalized));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    print!("{}", content);

    Ok(())
}
