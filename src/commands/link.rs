use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::rfclib::index;
use crate::rfclib::rfc;

/// Executes the `link` command: associates an RFC with a source file
pub fn execute(
    project_root: &Path,
    number: &str,
    link_path: &str,
    force: bool,
) -> Result<(), String> {
    // 1. Normalize RFC number
    let normalized = rfc::normalize_number(number)?;
    let path = rfc::rfc_path(project_root, number)?;

    if !path.exists() {
        return Err(format!("RFC-{} not found.", normalized));
    }

    // 2. Normalize link path
    let normalized_link = rfc::normalize_link_path(link_path)?;

    // 3. Check that target file exists
    let target = project_root.join(&normalized_link);
    if !target.exists() {
        return Err(format!("File not found: {}", normalized_link));
    }

    // 4. Load and refresh index
    let mut idx = index::load_index(project_root)?;
    index::refresh_index(project_root, &mut idx)?;

    // 5. Check status — block accepted/implemented without --force
    if let Some(entry) = idx.rfcs.iter().find(|e| e.number == normalized) {
        if (entry.status == "accepted" || entry.status == "implemented") && !force {
            return Err(format!(
                "RFC-{} is {}. Use --force to modify.",
                normalized, entry.status
            ));
        }
    }

    // 6. Check if link already exists
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    let frontmatter = rfc::parse_frontmatter(&content)?;

    if frontmatter.links.contains(&normalized_link) {
        println!(
            "RFC-{}: link already exists: {}",
            normalized, normalized_link
        );
        return Ok(());
    }

    // 7. Add link to frontmatter
    let new_content = rfc::add_to_frontmatter_list(&path, "links", &normalized_link)?;

    // 8. Update index
    let updated_fm = rfc::parse_frontmatter(&new_content)?;
    let file_mtime = fs::metadata(&path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default();

    // Compute content_hash if accepted/implemented and --force
    let content_hash =
        if force && (updated_fm.status == "accepted" || updated_fm.status == "implemented") {
            Some(index::compute_content_hash(&new_content))
        } else {
            idx.rfcs
                .iter()
                .find(|e| e.number == normalized)
                .and_then(|e| e.content_hash.clone())
        };

    idx.rfcs.retain(|e| e.number != normalized);
    idx.rfcs.push(index::IndexEntry {
        number: normalized.clone(),
        title: updated_fm.title,
        status: updated_fm.status,
        dependencies: updated_fm.dependencies,
        superseded_by: updated_fm.superseded_by,
        links: updated_fm.links,
        mtime: file_mtime,
        content_hash,
    });
    idx.rfcs.sort_by(|a, b| a.number.cmp(&b.number));
    index::save_index(project_root, &idx)?;

    // 9. Print confirmation
    println!("RFC-{}: linked {} ✅", normalized, normalized_link);

    Ok(())
}
