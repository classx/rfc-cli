use std::path::Path;

use crate::rfclib::index;

/// Executes the `list` command: shows table of all RFCs
pub fn execute(project_root: &Path, status_filter: Option<&str>) -> Result<(), String> {
    let mut idx = index::load_index(project_root)?;
    index::refresh_index(project_root, &mut idx)?;

    let mut rfcs: Vec<&index::IndexEntry> = idx.rfcs.iter().collect();

    // Filter by status if specified
    if let Some(filter) = status_filter {
        rfcs.retain(|e| e.status == filter);
    }

    // Sort by number
    rfcs.sort_by(|a, b| a.number.cmp(&b.number));

    if rfcs.is_empty() {
        println!("No RFCs found.");
        return Ok(());
    }

    // Calculate column widths
    let max_status = rfcs
        .iter()
        .map(|e| e.status.len())
        .max()
        .unwrap_or(6)
        .max(6);

    // Print header
    println!(
        " {:<6} {:<width$} {}",
        "#",
        "Status",
        "Title",
        width = max_status
    );

    // Print rows
    for entry in &rfcs {
        println!(
            " {:<6} {:<width$} {}",
            entry.number,
            entry.status,
            entry.title,
            width = max_status
        );
    }

    Ok(())
}
