use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
