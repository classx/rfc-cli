use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    pub rfcs: Vec<IndexEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexEntry {
    pub number: String,
    pub title: String,
    pub status: String,
    pub dependencies: Vec<String>,
    pub superseded_by: Option<String>,
    pub links: Vec<String>,
    pub mtime: String,
    pub content_hash: Option<String>,
}

impl Index {
    pub fn empty() -> Self {
        Index { rfcs: Vec::new() }
    }
}

/// Loads index from docs/rfcs/.index.json
pub fn load_index(project_root: &Path) -> Result<Index, String> {
    let index_path = project_root.join("docs/rfcs/.index.json");

    if !index_path.exists() {
        return Ok(Index::empty());
    }

    let content =
        fs::read_to_string(&index_path).map_err(|e| format!("Failed to read index file: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse index file: {}", e))
}

/// Saves index to docs/rfcs/.index.json
pub fn save_index(project_root: &Path, index: &Index) -> Result<(), String> {
    let index_path = project_root.join("docs/rfcs/.index.json");

    let content = serde_json::to_string_pretty(index)
        .map_err(|e| format!("Failed to serialize index: {}", e))?;

    fs::write(&index_path, content).map_err(|e| format!("Failed to write index file: {}", e))
}

/// Adds an entry to the index
pub fn add_entry(index: &mut Index, entry: IndexEntry) {
    index.rfcs.push(entry);
}

/// Computes SHA-256 hash of content, returns hex string
pub fn compute_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Refreshes index: compares mtime of files with index entries,
/// reparses changed ones, adds new ones, removes stale ones.
pub fn refresh_index(project_root: &Path, index: &mut Index) -> Result<(), String> {
    let rfcs_dir = project_root.join("docs/rfcs");
    if !rfcs_dir.exists() {
        return Ok(());
    }

    // Scan all .md files in docs/rfcs/
    let mut found_numbers: Vec<String> = Vec::new();
    let entries = fs::read_dir(&rfcs_dir)
        .map_err(|e| format!("Failed to read {}: {}", rfcs_dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Match files like "0001.md"
        let stem = match name_str.strip_suffix(".md") {
            Some(s) => s,
            None => continue,
        };
        if stem.parse::<u32>().is_err() {
            continue;
        }
        let number = stem.to_string();
        found_numbers.push(number.clone());

        // Get file mtime
        let metadata = entry
            .metadata()
            .map_err(|e| format!("Failed to get metadata for {}: {}", name_str, e))?;
        let file_mtime = metadata
            .modified()
            .unwrap_or(SystemTime::UNIX_EPOCH)
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default();

        // Find existing entry in index
        let existing = index.rfcs.iter().find(|e| e.number == number);

        let needs_update = match existing {
            Some(e) => e.mtime != file_mtime,
            None => true,
        };

        if needs_update {
            // Read and parse the file
            let file_path = rfcs_dir.join(&*name_str);
            let content = fs::read_to_string(&file_path)
                .map_err(|e| format!("Failed to read {}: {}", file_path.display(), e))?;

            // Check content_hash integrity for accepted/implemented RFCs
            if let Some(existing_entry) = existing {
                if existing_entry.status == "accepted" || existing_entry.status == "implemented" {
                    if let Some(ref stored_hash) = existing_entry.content_hash {
                        let current_hash = compute_content_hash(&content);
                        if current_hash != *stored_hash {
                            return Err(format!(
                                "RFC-{} ({}) was modified. Changes to accepted/implemented RFCs require a new superseding RFC.",
                                number, existing_entry.status
                            ));
                        }
                    }
                }
            }

            let frontmatter = crate::rfclib::rfc::parse_frontmatter(&content)?;

            let new_entry = IndexEntry {
                number: number.clone(),
                title: frontmatter.title,
                status: frontmatter.status,
                dependencies: frontmatter.dependencies,
                superseded_by: frontmatter.superseded_by,
                links: frontmatter.links,
                mtime: file_mtime,
                content_hash: existing.and_then(|e| e.content_hash.clone()),
            };

            // Remove old entry if exists, then add new one
            index.rfcs.retain(|e| e.number != number);
            index.rfcs.push(new_entry);
        }
    }

    // Remove entries for which files no longer exist
    index.rfcs.retain(|e| found_numbers.contains(&e.number));

    // Sort by number
    index.rfcs.sort_by(|a, b| a.number.cmp(&b.number));

    // Save
    save_index(project_root, index)?;

    Ok(())
}

/// Completely rebuilds the index from scratch by scanning all RFC files.
pub fn rebuild_index(project_root: &Path) -> Result<Index, String> {
    let rfcs_dir = project_root.join("docs/rfcs");

    if !rfcs_dir.exists() {
        return Err("docs/rfcs/ not found. Run \"rfc-cli init\" first.".to_string());
    }

    let mut index = Index::empty();

    let entries = fs::read_dir(&rfcs_dir)
        .map_err(|e| format!("Failed to read {}: {}", rfcs_dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        let stem = match name_str.strip_suffix(".md") {
            Some(s) => s,
            None => continue,
        };
        if stem.parse::<u32>().is_err() {
            continue;
        }
        let number = stem.to_string();

        let file_path = rfcs_dir.join(&*name_str);
        let content = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read {}: {}", file_path.display(), e))?;

        let frontmatter = crate::rfclib::rfc::parse_frontmatter(&content)?;

        let metadata = entry
            .metadata()
            .map_err(|e| format!("Failed to get metadata for {}: {}", name_str, e))?;
        let file_mtime = metadata
            .modified()
            .unwrap_or(SystemTime::UNIX_EPOCH)
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default();

        let content_hash =
            if frontmatter.status == "accepted" || frontmatter.status == "implemented" {
                Some(compute_content_hash(&content))
            } else {
                None
            };

        index.rfcs.push(IndexEntry {
            number,
            title: frontmatter.title,
            status: frontmatter.status,
            dependencies: frontmatter.dependencies,
            superseded_by: frontmatter.superseded_by,
            links: frontmatter.links,
            mtime: file_mtime,
            content_hash,
        });
    }

    index.rfcs.sort_by(|a, b| a.number.cmp(&b.number));

    save_index(project_root, &index)?;

    Ok(index)
}
