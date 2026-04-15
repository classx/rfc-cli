use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct RfcFrontmatter {
    pub title: String,
    pub status: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    pub superseded_by: Option<String>,
    #[serde(default)]
    pub links: Vec<String>,
}

/// Parses YAML frontmatter from RFC file content.
/// Expects content to start with `---` and have a closing `---`.
pub fn parse_frontmatter(content: &str) -> Result<RfcFrontmatter, String> {
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() || lines[0].trim() != "---" {
        return Err("Missing opening --- in frontmatter".to_string());
    }

    let closing = lines
        .iter()
        .skip(1)
        .position(|line| line.trim() == "---")
        .map(|pos| pos + 1);

    let closing_idx = match closing {
        Some(idx) => idx,
        None => return Err("Missing closing --- in frontmatter".to_string()),
    };

    let yaml_content = lines[1..closing_idx].join("\n");

    serde_yaml::from_str(&yaml_content)
        .map_err(|e| format!("Failed to parse frontmatter YAML: {}", e))
}

/// Generates RFC file content from template
pub fn generate_rfc_content(number: u32, title: &str) -> String {
    format!(
        r#"---
title: "RFC-{:04}: {}"
status: draft
dependencies: []
superseded_by: null
links: []
---

## Problem

## Goal

## Design

## Alternatives

## Voting

## Migration
"#,
        number, title
    )
}

/// Normalizes RFC number: "1" → "0001", "0001" → "0001", "42" → "0042"
pub fn normalize_number(input: &str) -> Result<String, String> {
    let n: u32 = input.parse().map_err(|_| {
        format!(
            "Invalid RFC number: '{}'. Expected a positive integer.",
            input
        )
    })?;
    Ok(format!("{:04}", n))
}

/// Returns path to RFC file by number
pub fn rfc_path(project_root: &Path, number: &str) -> Result<PathBuf, String> {
    let normalized = normalize_number(number)?;
    Ok(project_root
        .join("docs/rfcs")
        .join(format!("{}.md", normalized)))
}

/// Valid RFC statuses
pub const VALID_STATUSES: &[&str] = &[
    "draft",
    "review",
    "accepted",
    "implemented",
    "superseded",
    "deprecated",
];

/// Checks if a status transition is valid per RFC-0001
pub fn is_valid_transition(from: &str, to: &str) -> bool {
    matches!(
        (from, to),
        ("draft", "review")
            | ("draft", "deprecated")
            | ("review", "accepted")
            | ("review", "draft")
            | ("accepted", "implemented")
            | ("accepted", "deprecated")
            | ("accepted", "superseded")
            | ("implemented", "superseded")
            | ("implemented", "deprecated")
    )
}

/// Updates a field in the YAML frontmatter of an RFC file.
/// Returns the full file content after the update.
pub fn update_frontmatter_field(
    file_path: &Path,
    field: &str,
    value: &str,
) -> Result<String, String> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read {}: {}", file_path.display(), e))?;

    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() || lines[0].trim() != "---" {
        return Err("Missing opening --- in frontmatter".to_string());
    }

    let closing_idx = lines
        .iter()
        .skip(1)
        .position(|line| line.trim() == "---")
        .map(|pos| pos + 1)
        .ok_or_else(|| "Missing closing --- in frontmatter".to_string())?;

    // Build updated lines
    let mut updated_lines: Vec<String> = Vec::new();
    let mut field_found = false;

    for (i, line) in lines.iter().enumerate() {
        if i > 0 && i < closing_idx {
            // Inside frontmatter — check if this line starts with our field
            if line.starts_with(&format!("{}:", field)) {
                updated_lines.push(format!("{}: {}", field, value));
                field_found = true;
                continue;
            }
        }
        updated_lines.push(line.to_string());
    }

    if !field_found {
        return Err(format!("Field '{}' not found in frontmatter", field));
    }

    // Ensure trailing newline
    let new_content = updated_lines.join("\n") + "\n";

    std::fs::write(file_path, &new_content)
        .map_err(|e| format!("Failed to write {}: {}", file_path.display(), e))?;

    Ok(new_content)
}

/// Normalizes a link path: removes leading `./`, rejects absolute paths.
pub fn normalize_link_path(path: &str) -> Result<String, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("Link path cannot be empty.".to_string());
    }
    if trimmed.starts_with('/') {
        return Err(format!("Absolute paths are not allowed: {}", trimmed));
    }
    let normalized = trimmed.replace('\\', "/");
    let normalized = normalized.strip_prefix("./").unwrap_or(&normalized);
    Ok(normalized.to_string())
}

/// Adds an item to a list field in YAML frontmatter.
/// Returns the full file content after the update.
pub fn add_to_frontmatter_list(
    file_path: &Path,
    field: &str,
    value: &str,
) -> Result<String, String> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read {}: {}", file_path.display(), e))?;

    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() || lines[0].trim() != "---" {
        return Err("Missing opening --- in frontmatter".to_string());
    }

    let closing_idx = lines
        .iter()
        .skip(1)
        .position(|line| line.trim() == "---")
        .map(|pos| pos + 1)
        .ok_or_else(|| "Missing closing --- in frontmatter".to_string())?;

    // Find the field line
    let field_prefix = format!("{}:", field);
    let field_line_idx = (1..closing_idx)
        .find(|&i| lines[i].starts_with(&field_prefix))
        .ok_or_else(|| format!("Field '{}' not found in frontmatter", field))?;

    // Determine the range of lines belonging to this field
    // The field line itself, plus any subsequent lines that are indented list items (  - ...)
    let mut field_end_idx = field_line_idx + 1;
    while field_end_idx < closing_idx && lines[field_end_idx].starts_with("  - ") {
        field_end_idx += 1;
    }

    // Parse existing items using parse_frontmatter
    let frontmatter = parse_frontmatter(&content)?;
    let mut items: Vec<String> = match field {
        "links" => frontmatter.links,
        "dependencies" => frontmatter.dependencies,
        _ => return Err(format!("Unsupported list field: {}", field)),
    };

    // Add the new value
    items.push(value.to_string());

    // Build replacement lines
    let mut replacement: Vec<String> = Vec::new();
    if items.is_empty() {
        replacement.push(format!("{}: []", field));
    } else {
        replacement.push(format!("{}:", field));
        for item in &items {
            replacement.push(format!("  - {}", item));
        }
    }

    // Reconstruct the file
    let mut new_lines: Vec<String> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if i == field_line_idx {
            // Replace with new block
            for r in &replacement {
                new_lines.push(r.clone());
            }
        } else if i > field_line_idx && i < field_end_idx {
            // Skip old list items
            continue;
        } else {
            new_lines.push(line.to_string());
        }
    }

    let new_content = new_lines.join("\n") + "\n";

    std::fs::write(file_path, &new_content)
        .map_err(|e| format!("Failed to write {}: {}", file_path.display(), e))?;

    Ok(new_content)
}

/// Removes an item from a list field in YAML frontmatter.
/// Returns the full file content after the update.
pub fn remove_from_frontmatter_list(
    file_path: &Path,
    field: &str,
    value: &str,
) -> Result<String, String> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read {}: {}", file_path.display(), e))?;

    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() || lines[0].trim() != "---" {
        return Err("Missing opening --- in frontmatter".to_string());
    }

    let closing_idx = lines
        .iter()
        .skip(1)
        .position(|line| line.trim() == "---")
        .map(|pos| pos + 1)
        .ok_or_else(|| "Missing closing --- in frontmatter".to_string())?;

    let field_prefix = format!("{}:", field);
    let field_line_idx = (1..closing_idx)
        .find(|&i| lines[i].starts_with(&field_prefix))
        .ok_or_else(|| format!("Field '{}' not found in frontmatter", field))?;

    let mut field_end_idx = field_line_idx + 1;
    while field_end_idx < closing_idx && lines[field_end_idx].starts_with("  - ") {
        field_end_idx += 1;
    }

    let frontmatter = parse_frontmatter(&content)?;
    let mut items: Vec<String> = match field {
        "links" => frontmatter.links,
        "dependencies" => frontmatter.dependencies,
        _ => return Err(format!("Unsupported list field: {}", field)),
    };

    // Remove the value
    let original_len = items.len();
    items.retain(|item| item != value);
    if items.len() == original_len {
        return Err(format!("Value '{}' not found in field '{}'", value, field));
    }

    // Build replacement lines
    let mut replacement: Vec<String> = Vec::new();
    if items.is_empty() {
        replacement.push(format!("{}: []", field));
    } else {
        replacement.push(format!("{}:", field));
        for item in &items {
            replacement.push(format!("  - {}", item));
        }
    }

    let mut new_lines: Vec<String> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if i == field_line_idx {
            for r in &replacement {
                new_lines.push(r.clone());
            }
        } else if i > field_line_idx && i < field_end_idx {
            continue;
        } else {
            new_lines.push(line.to_string());
        }
    }

    let new_content = new_lines.join("\n") + "\n";

    std::fs::write(file_path, &new_content)
        .map_err(|e| format!("Failed to write {}: {}", file_path.display(), e))?;

    Ok(new_content)
}
