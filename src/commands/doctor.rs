use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::rfclib::index;

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub rfc_number: String,
    pub severity: Severity,
    pub message: String,
}

pub fn execute(project_root: &Path, stale_days: u64) -> Result<(), String> {
    let rfcs_dir = project_root.join("docs/rfcs");
    if !rfcs_dir.exists() {
        return Err("docs/rfcs/ not found. Run \"rfc-cli init\" first.".to_string());
    }

    let mut idx = index::load_index(project_root)?;
    index::refresh_index(project_root, &mut idx)?;

    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    for entry in &idx.rfcs {
        diagnostics.extend(check_code_drift(project_root, entry));
        diagnostics.extend(check_no_implementation(entry));
        diagnostics.extend(check_dead_links(project_root, entry));
        diagnostics.extend(check_stale_draft(entry, stale_days));
        diagnostics.extend(check_unresolved_dependencies(entry, &idx));
    }

    diagnostics.extend(check_dependency_cycles(&idx));

    if diagnostics.is_empty() {
        println!("All RFCs are healthy. ✅");
        return Ok(());
    }

    // Group by RFC number
    let mut by_rfc: HashMap<String, Vec<&Diagnostic>> = HashMap::new();
    for d in &diagnostics {
        by_rfc.entry(d.rfc_number.clone()).or_default().push(d);
    }

    let mut rfc_numbers: Vec<&String> = by_rfc.keys().collect();
    rfc_numbers.sort();

    let mut error_count = 0usize;
    let mut warning_count = 0usize;

    for num in &rfc_numbers {
        // Find short title
        let short_title = idx
            .rfcs
            .iter()
            .find(|e| &e.number == *num)
            .map(|e| {
                e.title
                    .strip_prefix(&format!("RFC-{}: ", e.number))
                    .unwrap_or(&e.title)
                    .to_string()
            })
            .unwrap_or_else(|| "unknown".to_string());

        println!("\nRFC-{} ({}):", num, short_title);

        for d in &by_rfc[*num] {
            match d.severity {
                Severity::Error => {
                    println!("  ❌ {}", d.message);
                    error_count += 1;
                }
                Severity::Warning => {
                    println!("  ⚠️  {}", d.message);
                    warning_count += 1;
                }
            }
        }
    }

    println!(
        "\nSummary: {} error(s), {} warning(s) across {} RFC(s).",
        error_count,
        warning_count,
        rfc_numbers.len()
    );

    if error_count > 0 {
        Err(format!("Found {} error(s).", error_count))
    } else {
        Ok(())
    }
}

/// D1: Code drift — linked file modified after RFC acceptance
fn check_code_drift(project_root: &Path, entry: &index::IndexEntry) -> Vec<Diagnostic> {
    let mut results = Vec::new();

    if entry.status != "accepted" && entry.status != "implemented" {
        return results;
    }
    if entry.links.is_empty() {
        return results;
    }

    let rfc_path = project_root
        .join("docs/rfcs")
        .join(format!("{}.md", entry.number));

    let rfc_mtime = match fs::metadata(&rfc_path).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return results,
    };

    for link in &entry.links {
        let link_path = project_root.join(link);
        if !link_path.exists() {
            continue; // D3 handles missing files
        }
        if let Ok(link_mtime) = fs::metadata(&link_path).and_then(|m| m.modified()) {
            if link_mtime > rfc_mtime {
                results.push(Diagnostic {
                    rfc_number: entry.number.clone(),
                    severity: Severity::Error,
                    message: format!("code drift: {} modified after RFC acceptance", link),
                });
            }
        }
    }

    results
}

/// D2: No implementation — accepted RFC with no links
fn check_no_implementation(entry: &index::IndexEntry) -> Vec<Diagnostic> {
    if entry.status == "accepted" && entry.links.is_empty() {
        vec![Diagnostic {
            rfc_number: entry.number.clone(),
            severity: Severity::Warning,
            message: "no linked files (status: accepted)".to_string(),
        }]
    } else {
        Vec::new()
    }
}

/// D3: Dead links — linked files that don't exist on disk
fn check_dead_links(project_root: &Path, entry: &index::IndexEntry) -> Vec<Diagnostic> {
    let mut results = Vec::new();

    for link in &entry.links {
        let link_path = project_root.join(link);
        if !link_path.exists() {
            results.push(Diagnostic {
                rfc_number: entry.number.clone(),
                severity: Severity::Error,
                message: format!("dead link: {} (file not found)", link),
            });
        }
    }

    results
}

/// D4: Stale draft — draft RFC not updated for too long
fn check_stale_draft(entry: &index::IndexEntry, stale_days: u64) -> Vec<Diagnostic> {
    if entry.status != "draft" {
        return Vec::new();
    }

    let mtime_secs: u64 = match entry.mtime.parse() {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let now_secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let age_secs = now_secs.saturating_sub(mtime_secs);
    let age_days = age_secs / 86400;

    if age_days >= stale_days {
        vec![Diagnostic {
            rfc_number: entry.number.clone(),
            severity: Severity::Warning,
            message: format!("stale draft (last modified {} days ago)", age_days),
        }]
    } else {
        Vec::new()
    }
}

/// D5: Unresolved dependencies — accepted RFC depends on non-accepted/implemented RFC
fn check_unresolved_dependencies(entry: &index::IndexEntry, idx: &index::Index) -> Vec<Diagnostic> {
    if entry.status != "accepted" {
        return Vec::new();
    }

    let mut results = Vec::new();

    for dep_str in &entry.dependencies {
        let dep_num = dep_str.strip_prefix("RFC-").unwrap_or(dep_str);
        let dep_normalized = match crate::rfclib::rfc::normalize_number(dep_num) {
            Ok(n) => n,
            Err(_) => {
                results.push(Diagnostic {
                    rfc_number: entry.number.clone(),
                    severity: Severity::Warning,
                    message: format!("invalid dependency reference: {}", dep_str),
                });
                continue;
            }
        };

        match idx.rfcs.iter().find(|e| e.number == dep_normalized) {
            Some(dep_entry) => {
                if dep_entry.status != "accepted" && dep_entry.status != "implemented" {
                    let short_title = dep_entry
                        .title
                        .strip_prefix(&format!("RFC-{}: ", dep_entry.number))
                        .unwrap_or(&dep_entry.title);
                    results.push(Diagnostic {
                        rfc_number: entry.number.clone(),
                        severity: Severity::Warning,
                        message: format!(
                            "depends on RFC-{} ({}) which is still in {}",
                            dep_normalized, short_title, dep_entry.status
                        ),
                    });
                }
            }
            None => {
                results.push(Diagnostic {
                    rfc_number: entry.number.clone(),
                    severity: Severity::Warning,
                    message: format!("depends on RFC-{} which does not exist", dep_normalized),
                });
            }
        }
    }

    results
}

/// D6: Dependency cycles — detect cycles in the dependency graph using DFS
fn check_dependency_cycles(idx: &index::Index) -> Vec<Diagnostic> {
    // Build adjacency list: number -> list of dependency numbers
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for entry in &idx.rfcs {
        let mut deps = Vec::new();
        for dep_str in &entry.dependencies {
            let dep_num = dep_str.strip_prefix("RFC-").unwrap_or(dep_str);
            if let Ok(normalized) = crate::rfclib::rfc::normalize_number(dep_num) {
                deps.push(normalized);
            }
        }
        graph.insert(entry.number.clone(), deps);
    }

    // DFS with three colors: 0=white, 1=gray, 2=black
    let mut color: HashMap<String, u8> = HashMap::new();
    for key in graph.keys() {
        color.insert(key.clone(), 0);
    }

    let mut reported_cycles: HashSet<String> = HashSet::new();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    let mut sorted_nodes: Vec<String> = graph.keys().cloned().collect();
    sorted_nodes.sort();

    for node in &sorted_nodes {
        if color[node] == 0 {
            let mut path = Vec::new();
            dfs_cycle(
                node,
                &graph,
                &mut color,
                &mut path,
                &mut reported_cycles,
                &mut diagnostics,
            );
        }
    }

    diagnostics
}

fn dfs_cycle(
    node: &str,
    graph: &HashMap<String, Vec<String>>,
    color: &mut HashMap<String, u8>,
    path: &mut Vec<String>,
    reported_cycles: &mut HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    color.insert(node.to_string(), 1); // Gray
    path.push(node.to_string());

    if let Some(neighbors) = graph.get(node) {
        for neighbor in neighbors {
            match color.get(neighbor.as_str()) {
                Some(1) => {
                    // Found a cycle — neighbor is in current path (gray)
                    let cycle_start = path.iter().position(|n| n == neighbor).unwrap_or(0);
                    let cycle_nodes: Vec<String> = path[cycle_start..].to_vec();

                    // Create a canonical key to avoid reporting same cycle twice
                    let mut key_parts = cycle_nodes.clone();
                    key_parts.sort();
                    let cycle_key = key_parts.join(",");

                    if !reported_cycles.contains(&cycle_key) {
                        reported_cycles.insert(cycle_key);
                        let mut cycle_display = cycle_nodes.clone();
                        cycle_display.push(neighbor.clone());
                        let cycle_str = cycle_display.join(" → ");

                        for cn in &cycle_nodes {
                            diagnostics.push(Diagnostic {
                                rfc_number: cn.clone(),
                                severity: Severity::Error,
                                message: format!("circular dependency detected: {}", cycle_str),
                            });
                        }
                    }
                }
                Some(0) => {
                    // White — recurse
                    dfs_cycle(neighbor, graph, color, path, reported_cycles, diagnostics);
                }
                _ => {} // Black (2) or unknown — already fully processed, skip
            }
        }
    }

    path.pop();
    color.insert(node.to_string(), 2); // Black
}
