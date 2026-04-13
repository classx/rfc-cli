use std::fs;
use std::path::Path;

use crate::rfclib::index::{self, Index};

/// Executes the `init` command: creates docs/rfcs/ directory and .index.json
pub fn execute(project_root: &Path) -> Result<(), String> {
    let rfcs_dir = project_root.join("docs/rfcs");
    let index_path = rfcs_dir.join(".index.json");

    let mut already_exists = true;

    if !rfcs_dir.exists() {
        fs::create_dir_all(&rfcs_dir)
            .map_err(|e| format!("Failed to create {}: {}", rfcs_dir.display(), e))?;
        println!("Created {}", rfcs_dir.display());
        already_exists = false;
    }

    if !index_path.exists() {
        let empty_index = Index::empty();
        index::save_index(project_root, &empty_index)?;
        println!("Created {}", index_path.display());
        already_exists = false;
    }

    if already_exists {
        println!("Already initialized.");
    }

    Ok(())
}
