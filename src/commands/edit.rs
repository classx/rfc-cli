use std::env;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

use crate::rfclib::index;
use crate::rfclib::rfc;

/// Executes the `edit` command: opens RFC in $EDITOR
pub fn execute(project_root: &Path, number: &str, force: bool) -> Result<(), String> {
    // 1. Check $EDITOR
    let editor = env::var("EDITOR")
        .map_err(|_| "Error: $EDITOR is not set. Run: export EDITOR=vim".to_string())?;

    // 2. Normalize number and get absolute path
    let normalized = rfc::normalize_number(number)?;
    let path = rfc::rfc_path(project_root, number)?;

    // 3. Check file exists
    if !path.exists() {
        return Err(format!("RFC-{} not found.", normalized));
    }

    // 4. Check status — block accepted/implemented without --force
    let mut idx = index::load_index(project_root)?;
    index::refresh_index(project_root, &mut idx)?;

    if let Some(entry) = idx.rfcs.iter().find(|e| e.number == normalized) {
        if entry.status == "accepted" || entry.status == "implemented" {
            if !force {
                return Err(format!(
                    "RFC-{} is {}. Use 'rfc-cli view {}' to read, or 'rfc-cli edit {} --force' to edit anyway.",
                    normalized, entry.status, normalized, normalized
                ));
            }
            eprintln!(
                "Warning: RFC-{} is {}. Changes to accepted/implemented RFCs require a new superseding RFC.",
                normalized, entry.status
            );
        }
    }

    // 5. Launch editor
    let status = Command::new(&editor)
        .arg(&path)
        .status()
        .map_err(|e| format!("Failed to launch editor '{}': {}", editor, e))?;

    if !status.success() {
        return Err(format!("Editor '{}' exited with error", editor));
    }

    // 6. Update index after editor closes — reparse frontmatter and update mtime
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let frontmatter = rfc::parse_frontmatter(&content)?;

    let file_mtime = std::fs::metadata(&path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default();

    // Preserve existing content_hash
    let existing_hash = idx
        .rfcs
        .iter()
        .find(|e| e.number == normalized)
        .and_then(|e| e.content_hash.clone());

    // Remove old entry and add updated one
    idx.rfcs.retain(|e| e.number != normalized);
    idx.rfcs.push(index::IndexEntry {
        number: normalized.clone(),
        title: frontmatter.title,
        status: frontmatter.status,
        dependencies: frontmatter.dependencies,
        superseded_by: frontmatter.superseded_by,
        links: frontmatter.links,
        mtime: file_mtime,
        content_hash: existing_hash,
    });

    idx.rfcs.sort_by(|a, b| a.number.cmp(&b.number));
    index::save_index(project_root, &idx)?;

    Ok(())
}
