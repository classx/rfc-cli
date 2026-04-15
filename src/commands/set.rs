use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::rfclib::index;
use crate::rfclib::rfc;

/// Executes the `set` command: changes RFC status with transition validation
pub fn execute(
    project_root: &Path,
    number: &str,
    target_status: &str,
    by: Option<&str>,
) -> Result<(), String> {
    // 1. Normalize and find file
    let normalized = rfc::normalize_number(number)?;
    let path = rfc::rfc_path(project_root, number)?;

    if !path.exists() {
        return Err(format!("RFC-{} not found.", normalized));
    }

    // 2. Load and refresh index
    let mut idx = index::load_index(project_root)?;
    index::refresh_index(project_root, &mut idx)?;

    // 3. Find current status
    let current_status = idx
        .rfcs
        .iter()
        .find(|e| e.number == normalized)
        .map(|e| e.status.clone())
        .ok_or_else(|| format!("RFC-{} not found in index.", normalized))?;

    // 4. Validate target status
    if !rfc::VALID_STATUSES.contains(&target_status) {
        return Err(format!(
            "Invalid status '{}'. Expected one of: {}",
            target_status,
            rfc::VALID_STATUSES.join(", ")
        ));
    }

    // 5. Validate transition
    if !rfc::is_valid_transition(&current_status, target_status) {
        return Err(format!(
            "Transition {} → {} is not allowed.",
            current_status, target_status
        ));
    }

    // 6. Handle superseded — require --by
    if target_status == "superseded" {
        let by_number = by.ok_or_else(|| {
            "Error: --by <number> is required for transition to superseded.".to_string()
        })?;
        let by_normalized = rfc::normalize_number(by_number)?;
        let by_path = rfc::rfc_path(project_root, by_number)?;
        if !by_path.exists() {
            return Err(format!("RFC-{} not found.", by_normalized));
        }
        // Update superseded_by in frontmatter
        rfc::update_frontmatter_field(&path, "superseded_by", &format!("RFC-{}", by_normalized))?;
    }

    // 7. Update status in frontmatter
    let new_content = rfc::update_frontmatter_field(&path, "status", target_status)?;

    // 8. Compute content_hash for accepted/implemented
    let content_hash = if target_status == "accepted" || target_status == "implemented" {
        Some(index::compute_content_hash(&new_content))
    } else {
        None
    };

    // 9. Get fresh mtime
    let file_mtime = fs::metadata(&path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default();

    // 10. Re-read frontmatter to get latest state (including superseded_by)
    let frontmatter = rfc::parse_frontmatter(&new_content)?;

    // 11. Update index entry
    idx.rfcs.retain(|e| e.number != normalized);
    idx.rfcs.push(index::IndexEntry {
        number: normalized.clone(),
        title: frontmatter.title.clone(),
        status: target_status.to_string(),
        dependencies: frontmatter.dependencies,
        superseded_by: frontmatter.superseded_by,
        links: frontmatter.links,
        mtime: file_mtime,
        content_hash,
    });
    idx.rfcs.sort_by(|a, b| a.number.cmp(&b.number));
    index::save_index(project_root, &idx)?;

    // 12. Print success message
    let short_title = frontmatter.title.clone();
    if target_status == "superseded" {
        let by_normalized = rfc::normalize_number(by.unwrap())?;
        println!(
            "RFC-{} ({}): {} → superseded (by RFC-{}) ✅",
            normalized, short_title, current_status, by_normalized
        );
    } else {
        println!(
            "RFC-{} ({}): {} → {} ✅",
            normalized, short_title, current_status, target_status
        );
    }

    Ok(())
}
