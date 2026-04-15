use std::fs;
use std::path::Path;

use crate::rfclib::index;
use crate::rfclib::rfc;

/// Executes the `check` command: validates RFC documents
pub fn execute(project_root: &Path, number: Option<&str>) -> Result<(), String> {
    let rfcs_dir = project_root.join("docs/rfcs");
    if !rfcs_dir.exists() {
        return Err("docs/rfcs/ not found. Run \"rfc-cli init\" first.".to_string());
    }

    // Load and refresh index
    let mut idx = index::load_index(project_root)?;
    index::refresh_index(project_root, &mut idx)?;

    // Determine which files to check
    let files_to_check: Vec<(String, std::path::PathBuf)> = if let Some(num) = number {
        let normalized = rfc::normalize_number(num)?;
        let path = rfc::rfc_path(project_root, num)?;
        if !path.exists() {
            return Err(format!("RFC-{} not found.", normalized));
        }
        vec![(normalized, path)]
    } else {
        // Scan all .md files
        let mut files = Vec::new();
        let entries = fs::read_dir(&rfcs_dir)
            .map_err(|e| format!("Failed to read {}: {}", rfcs_dir.display(), e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy().to_string();
            if let Some(stem) = name_str.strip_suffix(".md") {
                if stem.parse::<u32>().is_ok() {
                    files.push((stem.to_string(), rfcs_dir.join(&name_str)));
                }
            }
        }
        files.sort_by(|a, b| a.0.cmp(&b.0));
        files
    };

    let mut errors: Vec<String> = Vec::new();

    for (file_number, file_path) in &files_to_check {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("RFC-{}: failed to read file: {}", file_number, e));
                continue;
            }
        };

        // (a) Parse frontmatter
        let frontmatter = match rfc::parse_frontmatter(&content) {
            Ok(fm) => fm,
            Err(e) => {
                errors.push(format!("RFC-{}: {}", file_number, e));
                continue; // Can't do further checks without frontmatter
            }
        };

        // (b) Check required fields are non-empty (title, status)
        if frontmatter.title.trim().is_empty() {
            errors.push(format!("RFC-{}: title is empty", file_number));
        }
        if frontmatter.status.trim().is_empty() {
            errors.push(format!("RFC-{}: status is empty", file_number));
        }

        // (c) Check status is valid
        if !rfc::VALID_STATUSES.contains(&frontmatter.status.as_str()) {
            errors.push(format!(
                "RFC-{}: invalid status '{}' (expected one of: {})",
                file_number,
                frontmatter.status,
                rfc::VALID_STATUSES.join(", ")
            ));
        }

        // (d) Check required sections
        let required_sections = ["## Problem", "## Goal", "## Design", "## Alternatives"];
        for section in &required_sections {
            if !content.contains(section) {
                errors.push(format!(
                    "RFC-{}: missing required section '{}'",
                    file_number, section
                ));
            }
        }

        // (e) Check number in filename matches title
        let expected_prefix = format!("RFC-{}", file_number);
        if !frontmatter.title.contains(&expected_prefix) {
            errors.push(format!(
                "RFC-{}: title '{}' does not contain '{}'",
                file_number, frontmatter.title, expected_prefix
            ));
        }

        // (f) Check dependencies exist
        for dep in &frontmatter.dependencies {
            // Extract number from "RFC-NNNN"
            let dep_num = dep.strip_prefix("RFC-").unwrap_or(dep);
            if let Ok(dep_path) = rfc::rfc_path(project_root, dep_num) {
                if !dep_path.exists() {
                    errors.push(format!("RFC-{}: dependency {} not found", file_number, dep));
                }
            } else {
                errors.push(format!(
                    "RFC-{}: invalid dependency reference '{}'",
                    file_number, dep
                ));
            }
        }

        // (g) Check links exist
        for link in &frontmatter.links {
            let link_path = project_root.join(link);
            if !link_path.exists() {
                errors.push(format!(
                    "RFC-{}: dead link: {} (file not found)",
                    file_number, link
                ));
            }
        }

        // (h) Check content_hash for accepted/implemented
        if frontmatter.status == "accepted" || frontmatter.status == "implemented" {
            if let Some(index_entry) = idx.rfcs.iter().find(|e| e.number == *file_number) {
                if let Some(ref stored_hash) = index_entry.content_hash {
                    let actual_hash = index::compute_content_hash(&content);
                    if actual_hash != *stored_hash {
                        errors.push(format!(
                            "RFC-{}: content_hash mismatch (accepted RFC was modified)",
                            file_number
                        ));
                    }
                }
            }
        }
    }

    // Refresh index after check
    index::refresh_index(project_root, &mut idx).ok();

    if errors.is_empty() {
        println!("All checks passed. ✅");
        Ok(())
    } else {
        let error_count = errors.len();
        for err in &errors {
            eprintln!("{}", err);
        }
        eprintln!();
        // Return error to signal non-zero exit code
        Err(format!("Found {} error(s).", error_count))
    }
}
