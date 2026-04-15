use std::path::Path;

use crate::rfclib::index;
use crate::rfclib::rfc;

/// Executes the `status` command: shows RFC status from index
pub fn execute(project_root: &Path, number: &str) -> Result<(), String> {
    let mut idx = index::load_index(project_root)?;
    index::refresh_index(project_root, &mut idx)?;

    let normalized = rfc::normalize_number(number)?;

    let entry = idx.rfcs.iter().find(|e| e.number == normalized);

    match entry {
        Some(e) => {
            println!("RFC-{}: {}", normalized, e.status);
            Ok(())
        }
        None => Err(format!("RFC-{} not found.", normalized)),
    }
}
