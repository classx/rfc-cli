use serde::{Deserialize, Serialize};

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

## Проблема

## Задача

## Дизайн

## Альтернативы

## Голосование

## Миграция
"#,
        number, title
    )
}
