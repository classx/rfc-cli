use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::rfclib::index::{self, IndexEntry};
use crate::rfclib::rfc;

/// Executes the `new` command: creates a new RFC from template
pub fn execute(project_root: &Path, title: &str) -> Result<(), String> {
    let rfcs_dir = project_root.join("docs/rfcs");

    if !rfcs_dir.exists() {
        return Err("docs/rfcs/ not found. Run \"rfc-cli init\" first.".to_string());
    }

    // Scan existing RFC files and determine next number
    let next_number = get_next_number(&rfcs_dir)?;

    // Generate content from template
    let content = rfc::generate_rfc_content(next_number, title);

    // Write file
    let filename = format!("{:04}.md", next_number);
    let file_path = rfcs_dir.join(&filename);
    fs::write(&file_path, &content)
        .map_err(|e| format!("Failed to write {}: {}", file_path.display(), e))?;

    // Update index
    let mut idx = index::load_index(project_root)?;
    let mtime = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default();

    let entry = IndexEntry {
        number: format!("{:04}", next_number),
        title: format!("RFC-{:04}: {}", next_number, title),
        status: "draft".to_string(),
        dependencies: Vec::new(),
        superseded_by: None,
        links: Vec::new(),
        mtime,
        content_hash: None,
    };
    index::add_entry(&mut idx, entry);
    index::save_index(project_root, &idx)?;

    println!("Created {}", file_path.display());

    Ok(())
}

/// Scans docs/rfcs/ for existing RFC files and returns the next available number
fn get_next_number(rfcs_dir: &Path) -> Result<u32, String> {
    let mut max_number: u32 = 0;

    let entries = fs::read_dir(rfcs_dir)
        .map_err(|e| format!("Failed to read {}: {}", rfcs_dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Match files like "0001.md", "0042.md"
        if let Some(stem) = name_str.strip_suffix(".md") {
            if let Ok(num) = stem.parse::<u32>() {
                if num > max_number {
                    max_number = num;
                }
            }
        }
    }

    Ok(max_number + 1)
}
