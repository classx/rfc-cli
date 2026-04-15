use std::path::Path;

use crate::rfclib::index;
use crate::rfclib::rfc;

/// Executes the `deps` command: shows dependency tree for an RFC
pub fn execute(project_root: &Path, number: &str, reverse: bool) -> Result<(), String> {
    let normalized = rfc::normalize_number(number)?;

    // Load and refresh index
    let mut idx = index::load_index(project_root)?;
    index::refresh_index(project_root, &mut idx)?;

    // Check that RFC exists in index
    let entry = idx
        .rfcs
        .iter()
        .find(|e| e.number == normalized)
        .ok_or_else(|| format!("RFC-{} not found.", normalized))?;

    if reverse {
        // Reverse dependencies: find all RFCs that depend on this one
        let dep_ref = format!("RFC-{}", normalized);
        let reverse_deps: Vec<&index::IndexEntry> = idx
            .rfcs
            .iter()
            .filter(|e| e.dependencies.contains(&dep_ref))
            .collect();

        if reverse_deps.is_empty() {
            println!("RFC-{} has no reverse dependencies.", normalized);
        } else {
            println!("RFC-{} is depended on by:", normalized);
            for dep in &reverse_deps {
                // Extract short title (strip "RFC-NNNN: " prefix if present)
                let short_title = dep
                    .title
                    .strip_prefix(&format!("RFC-{}: ", dep.number))
                    .unwrap_or(&dep.title);
                println!("  - RFC-{} ({}) [{}]", dep.number, short_title, dep.status);
            }
        }
    } else {
        // Forward dependencies
        if entry.dependencies.is_empty() {
            println!("RFC-{} has no dependencies.", normalized);
        } else {
            println!("RFC-{} depends on:", normalized);
            for dep_str in &entry.dependencies {
                // Extract number from "RFC-NNNN"
                let dep_num = dep_str.strip_prefix("RFC-").unwrap_or(dep_str);
                match rfc::normalize_number(dep_num) {
                    Ok(dep_normalized) => {
                        if let Some(dep_entry) =
                            idx.rfcs.iter().find(|e| e.number == dep_normalized)
                        {
                            let short_title = dep_entry
                                .title
                                .strip_prefix(&format!("RFC-{}: ", dep_entry.number))
                                .unwrap_or(&dep_entry.title);
                            println!(
                                "  - RFC-{} ({}) [{}]",
                                dep_normalized, short_title, dep_entry.status
                            );
                        } else {
                            println!("  - RFC-{} (not found) ⚠️", dep_normalized);
                        }
                    }
                    Err(_) => {
                        println!("  - {} (invalid reference) ⚠️", dep_str);
                    }
                }
            }
        }
    }

    Ok(())
}
