use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Output;

// Helper: create a temporary directory for test isolation
fn create_temp_dir(test_name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("rfc_cli_test_{}", test_name));
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }
    fs::create_dir_all(&dir).unwrap();
    dir
}

// Helper: clean up temp directory
fn cleanup(dir: &Path) {
    if dir.exists() {
        fs::remove_dir_all(dir).unwrap();
    }
}

// Helper: run rfc-cli binary with given args in a specific directory
fn run_rfc_cli(project_dir: &Path, args: &[&str]) -> Output {
    let binary = env!("CARGO_BIN_EXE_rfc-cli");
    std::process::Command::new(binary)
        .args(args)
        .env("RFC_HOME", project_dir.as_os_str())
        .output()
        .expect("Failed to execute rfc-cli")
}

// Helper: run rfc-cli with a custom $EDITOR
fn run_rfc_cli_with_editor(project_dir: &Path, args: &[&str], editor: &str) -> Output {
    let binary = env!("CARGO_BIN_EXE_rfc-cli");
    std::process::Command::new(binary)
        .args(args)
        .env("RFC_HOME", project_dir.as_os_str())
        .env("EDITOR", editor)
        .output()
        .expect("Failed to execute rfc-cli")
}

// Helper: run rfc-cli with $EDITOR explicitly removed
fn run_rfc_cli_without_editor(project_dir: &Path, args: &[&str]) -> Output {
    let binary = env!("CARGO_BIN_EXE_rfc-cli");
    std::process::Command::new(binary)
        .args(args)
        .env("RFC_HOME", project_dir.as_os_str())
        .env_remove("EDITOR")
        .output()
        .expect("Failed to execute rfc-cli")
}

// Helper: write an RFC file with given status AND update the index entry
fn write_rfc_with_status(dir: &Path, number: &str, title: &str, status: &str) {
    let content = format!(
        "---\ntitle: \"RFC-{}: {}\"\nstatus: {}\ndependencies: []\nsuperseded_by: null\nlinks: []\n---\n\n## Problem\n",
        number, title, status
    );
    let rfc_path = dir.join(format!("docs/rfcs/{}.md", number));
    fs::write(&rfc_path, &content).unwrap();

    // Also update the index so status is consistent
    let index_path = dir.join("docs/rfcs/.index.json");
    let index_content = fs::read_to_string(&index_path).unwrap();
    let mut parsed: serde_json::Value = serde_json::from_str(&index_content).unwrap();

    let full_title = format!("RFC-{}: {}", number, title);
    let mtime = fs::metadata(&rfc_path)
        .and_then(|m| m.modified())
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default();

    let rfcs = parsed["rfcs"].as_array_mut().unwrap();
    // Update existing entry or add new one
    let existing = rfcs
        .iter_mut()
        .find(|e| e["number"].as_str() == Some(number));
    if let Some(entry) = existing {
        entry["status"] = serde_json::json!(status);
        entry["title"] = serde_json::json!(full_title);
        entry["mtime"] = serde_json::json!(mtime);
    } else {
        rfcs.push(serde_json::json!({
            "number": number,
            "title": full_title,
            "status": status,
            "dependencies": [],
            "superseded_by": null,
            "links": [],
            "mtime": mtime,
            "content_hash": null
        }));
    }

    fs::write(&index_path, serde_json::to_string_pretty(&parsed).unwrap()).unwrap();
}

// Helper: create a temporary shell script to use as a fake $EDITOR
fn create_fake_editor_script(dir: &Path, name: &str, body: &str) -> String {
    let script_path = dir.join(name);
    let script_content = format!("#!/bin/sh\n{}\n", body);
    fs::write(&script_path, &script_content).unwrap();
    fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755)).unwrap();
    script_path.to_string_lossy().to_string()
}

// Helper: write an RFC file with custom dependencies and links, AND update the index entry
fn write_rfc_with_deps_and_links(
    dir: &Path,
    number: &str,
    title: &str,
    status: &str,
    deps: &[&str],
    links: &[&str],
) {
    let deps_yaml = if deps.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", deps.join(", "))
    };
    let links_yaml = if links.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", links.join(", "))
    };
    let content = format!(
        "---\ntitle: \"RFC-{}: {}\"\nstatus: {}\ndependencies: {}\nsuperseded_by: null\nlinks: {}\n---\n\n## Problem\n",
        number, title, status, deps_yaml, links_yaml
    );
    let rfc_path = dir.join(format!("docs/rfcs/{}.md", number));
    fs::write(&rfc_path, &content).unwrap();

    // Update index
    let index_path = dir.join("docs/rfcs/.index.json");
    let index_content = fs::read_to_string(&index_path).unwrap();
    let mut parsed: serde_json::Value = serde_json::from_str(&index_content).unwrap();

    let full_title = format!("RFC-{}: {}", number, title);
    let mtime = fs::metadata(&rfc_path)
        .and_then(|m| m.modified())
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default();

    let deps_json: Vec<serde_json::Value> = deps.iter().map(|d| serde_json::json!(d)).collect();
    let links_json: Vec<serde_json::Value> = links.iter().map(|l| serde_json::json!(l)).collect();

    let rfcs = parsed["rfcs"].as_array_mut().unwrap();
    let existing = rfcs
        .iter_mut()
        .find(|e| e["number"].as_str() == Some(number));
    if let Some(entry) = existing {
        entry["status"] = serde_json::json!(status);
        entry["title"] = serde_json::json!(full_title);
        entry["mtime"] = serde_json::json!(mtime);
        entry["dependencies"] = serde_json::json!(deps_json);
        entry["links"] = serde_json::json!(links_json);
    } else {
        rfcs.push(serde_json::json!({
            "number": number,
            "title": full_title,
            "status": status,
            "dependencies": deps_json,
            "superseded_by": null,
            "links": links_json,
            "mtime": mtime,
            "content_hash": null
        }));
    }

    fs::write(&index_path, serde_json::to_string_pretty(&parsed).unwrap()).unwrap();
}

// ============================================================
// Tests for `init` command
// ============================================================

#[test]
fn test_init_creates_directory_and_index() {
    let dir = create_temp_dir("init_creates");

    let output = run_rfc_cli(&dir, &["init"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "init should succeed");

    let rfcs_dir = dir.join("docs/rfcs");
    assert!(rfcs_dir.exists(), "docs/rfcs/ should be created");
    assert!(rfcs_dir.is_dir(), "docs/rfcs/ should be a directory");

    let index_path = rfcs_dir.join(".index.json");
    assert!(index_path.exists(), ".index.json should be created");

    // Verify index content is valid empty JSON
    let index_content = fs::read_to_string(&index_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&index_content).unwrap();
    assert_eq!(parsed["rfcs"], serde_json::json!([]));

    assert!(
        stdout.contains("Created"),
        "should print 'Created' messages"
    );

    cleanup(&dir);
}

#[test]
fn test_init_is_idempotent() {
    let dir = create_temp_dir("init_idempotent");

    // First init
    let output1 = run_rfc_cli(&dir, &["init"]);
    assert!(output1.status.success());

    // Second init
    let output2 = run_rfc_cli(&dir, &["init"]);
    assert!(output2.status.success());

    let stdout = String::from_utf8_lossy(&output2.stdout);
    assert!(
        stdout.contains("Already initialized"),
        "second init should say 'Already initialized', got: {}",
        stdout
    );

    cleanup(&dir);
}

#[test]
fn test_init_creates_nested_docs_directory() {
    let dir = create_temp_dir("init_nested");

    // docs/ doesn't exist yet
    assert!(!dir.join("docs").exists());

    let output = run_rfc_cli(&dir, &["init"]);
    assert!(output.status.success());

    assert!(dir.join("docs").exists(), "docs/ should be created");
    assert!(
        dir.join("docs/rfcs").exists(),
        "docs/rfcs/ should be created"
    );

    cleanup(&dir);
}

// ============================================================
// Tests for `new` command
// ============================================================

#[test]
fn test_new_creates_first_rfc() {
    let dir = create_temp_dir("new_first");

    // Init first
    run_rfc_cli(&dir, &["init"]);

    // Create new RFC
    let output = run_rfc_cli(&dir, &["new", "тестовый RFC"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "new should succeed");

    let rfc_path = dir.join("docs/rfcs/0001.md");
    assert!(rfc_path.exists(), "0001.md should be created");

    assert!(
        stdout.contains("0001.md"),
        "should print path with 0001.md, got: {}",
        stdout
    );

    // Verify content
    let content = fs::read_to_string(&rfc_path).unwrap();
    assert!(content.contains("title: \"RFC-0001: тестовый RFC\""));
    assert!(content.contains("status: draft"));
    assert!(content.contains("dependencies: []"));
    assert!(content.contains("superseded_by: null"));
    assert!(content.contains("links: []"));
    assert!(content.contains("## Problem"));
    assert!(content.contains("## Goal"));
    assert!(content.contains("## Design"));
    assert!(content.contains("## Alternatives"));
    assert!(content.contains("## Voting"));
    assert!(content.contains("## Migration"));

    cleanup(&dir);
}

#[test]
fn test_new_auto_increments_number() {
    let dir = create_temp_dir("new_increment");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "first RFC"]);
    run_rfc_cli(&dir, &["new", "second RFC"]);
    run_rfc_cli(&dir, &["new", "third RFC"]);

    assert!(
        dir.join("docs/rfcs/0001.md").exists(),
        "0001.md should exist"
    );
    assert!(
        dir.join("docs/rfcs/0002.md").exists(),
        "0002.md should exist"
    );
    assert!(
        dir.join("docs/rfcs/0003.md").exists(),
        "0003.md should exist"
    );

    // Verify titles
    let content1 = fs::read_to_string(dir.join("docs/rfcs/0001.md")).unwrap();
    assert!(content1.contains("RFC-0001: first RFC"));

    let content2 = fs::read_to_string(dir.join("docs/rfcs/0002.md")).unwrap();
    assert!(content2.contains("RFC-0002: second RFC"));

    let content3 = fs::read_to_string(dir.join("docs/rfcs/0003.md")).unwrap();
    assert!(content3.contains("RFC-0003: third RFC"));

    cleanup(&dir);
}

#[test]
fn test_new_handles_gaps_in_numbering() {
    let dir = create_temp_dir("new_gaps");

    run_rfc_cli(&dir, &["init"]);

    // Create 0001 and 0003 manually (simulating a gap where 0002 was deleted)
    fs::write(
        dir.join("docs/rfcs/0001.md"),
        "---\ntitle: \"RFC-0001: first\"\nstatus: draft\ndependencies: []\nsuperseded_by: null\nlinks: []\n---\n",
    )
    .unwrap();
    fs::write(
        dir.join("docs/rfcs/0003.md"),
        "---\ntitle: \"RFC-0003: third\"\nstatus: draft\ndependencies: []\nsuperseded_by: null\nlinks: []\n---\n",
    )
    .unwrap();

    // New RFC should be 0004 (max + 1)
    let output = run_rfc_cli(&dir, &["new", "after gap"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("0004.md"),
        "should create 0004.md (max existing is 0003), got: {}",
        stdout
    );
    assert!(dir.join("docs/rfcs/0004.md").exists());

    cleanup(&dir);
}

#[test]
fn test_new_fails_without_init() {
    let dir = create_temp_dir("new_no_init");

    // Don't run init
    let output = run_rfc_cli(&dir, &["new", "should fail"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "new should fail without init");
    assert!(
        stderr.contains("rfc-cli init"),
        "error should suggest running init, got: {}",
        stderr
    );

    cleanup(&dir);
}

#[test]
fn test_new_updates_index() {
    let dir = create_temp_dir("new_index");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "indexed RFC"]);

    let index_content = fs::read_to_string(dir.join("docs/rfcs/.index.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&index_content).unwrap();

    let rfcs = parsed["rfcs"].as_array().unwrap();
    assert_eq!(rfcs.len(), 1, "index should have 1 entry");

    let entry = &rfcs[0];
    assert_eq!(entry["number"], "0001");
    assert_eq!(entry["title"], "RFC-0001: indexed RFC");
    assert_eq!(entry["status"], "draft");
    assert!(entry["content_hash"].is_null());
    assert_eq!(entry["dependencies"], serde_json::json!([]));
    assert_eq!(entry["links"], serde_json::json!([]));

    cleanup(&dir);
}

#[test]
fn test_new_multiple_updates_index_correctly() {
    let dir = create_temp_dir("new_multi_index");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "first"]);
    run_rfc_cli(&dir, &["new", "second"]);

    let index_content = fs::read_to_string(dir.join("docs/rfcs/.index.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&index_content).unwrap();

    let rfcs = parsed["rfcs"].as_array().unwrap();
    assert_eq!(rfcs.len(), 2, "index should have 2 entries");
    assert_eq!(rfcs[0]["number"], "0001");
    assert_eq!(rfcs[1]["number"], "0002");

    cleanup(&dir);
}

// ============================================================
// Tests for RFC_HOME environment variable
// ============================================================

#[test]
fn test_rfc_home_environment_variable() {
    let dir = create_temp_dir("rfc_home");

    // Run init with RFC_HOME pointing to our temp dir
    let binary = env!("CARGO_BIN_EXE_rfc-cli");
    let output = std::process::Command::new(binary)
        .args(["init"])
        .env("RFC_HOME", dir.as_os_str())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(dir.join("docs/rfcs").exists());
    assert!(dir.join("docs/rfcs/.index.json").exists());

    cleanup(&dir);
}

#[test]
fn test_rfc_home_invalid_path() {
    let binary = env!("CARGO_BIN_EXE_rfc-cli");
    let output = std::process::Command::new(binary)
        .args(["init"])
        .env("RFC_HOME", "/nonexistent/path/that/does/not/exist")
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "should fail with invalid RFC_HOME"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("non-existent"),
        "should mention non-existent directory, got: {}",
        stderr
    );
}

// ============================================================
// Tests for template content correctness
// ============================================================

#[test]
fn test_new_template_has_all_required_sections() {
    let dir = create_temp_dir("new_sections");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "complete template"]);

    let content = fs::read_to_string(dir.join("docs/rfcs/0001.md")).unwrap();

    // Required sections per RFC-0001
    let required_sections = vec!["## Problem", "## Goal", "## Design", "## Alternatives"];

    // Optional sections per RFC-0001
    let optional_sections = vec!["## Voting", "## Migration"];

    for section in &required_sections {
        assert!(
            content.contains(section),
            "Template must contain required section '{}'\nContent:\n{}",
            section,
            content
        );
    }

    for section in &optional_sections {
        assert!(
            content.contains(section),
            "Template should contain optional section '{}'\nContent:\n{}",
            section,
            content
        );
    }

    cleanup(&dir);
}

#[test]
fn test_new_template_has_valid_frontmatter() {
    let dir = create_temp_dir("new_frontmatter");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "frontmatter test"]);

    let content = fs::read_to_string(dir.join("docs/rfcs/0001.md")).unwrap();

    // Check frontmatter structure
    assert!(content.starts_with("---\n"), "should start with ---");

    let lines: Vec<&str> = content.lines().collect();
    let second_separator = lines.iter().skip(1).position(|l| l.trim() == "---");
    assert!(
        second_separator.is_some(),
        "should have closing --- separator"
    );

    // Extract and verify YAML
    let closing_idx = second_separator.unwrap() + 1;
    let yaml_content = lines[1..closing_idx].join("\n");
    let frontmatter: serde_yaml::Value = serde_yaml::from_str(&yaml_content).unwrap();

    assert!(frontmatter["title"]
        .as_str()
        .unwrap()
        .starts_with("RFC-0001:"));
    assert_eq!(frontmatter["status"].as_str().unwrap(), "draft");
    assert!(frontmatter["dependencies"]
        .as_sequence()
        .unwrap()
        .is_empty());
    assert!(frontmatter["superseded_by"].is_null());
    assert!(frontmatter["links"].as_sequence().unwrap().is_empty());

    cleanup(&dir);
}

// ============================================================
// Tests for CLI help and error handling
// ============================================================

#[test]
fn test_help_flag() {
    let dir = create_temp_dir("help");

    let output = run_rfc_cli(&dir, &["--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("init"));
    assert!(stdout.contains("new"));

    cleanup(&dir);
}

#[test]
fn test_new_help() {
    let dir = create_temp_dir("new_help");

    let output = run_rfc_cli(&dir, &["new", "--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.to_lowercase().contains("title")
            || stdout.contains("TITLE")
            || stdout.contains("<TITLE>"),
        "new --help should mention title argument, got: {}",
        stdout
    );

    cleanup(&dir);
}

#[test]
fn test_new_without_title_fails() {
    let dir = create_temp_dir("new_no_title");

    run_rfc_cli(&dir, &["init"]);

    let output = run_rfc_cli(&dir, &["new"]);

    assert!(!output.status.success(), "new without title should fail");

    cleanup(&dir);
}

// ============================================================
// Tests for `list` command
// ============================================================

#[test]
fn test_list_empty_index() {
    let dir = create_temp_dir("list_empty");

    run_rfc_cli(&dir, &["init"]);

    let output = run_rfc_cli(&dir, &["list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("No RFCs found"),
        "empty list should say 'No RFCs found.', got: {}",
        stdout
    );

    cleanup(&dir);
}

#[test]
fn test_list_shows_all_rfcs_sorted() {
    let dir = create_temp_dir("list_all");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "first RFC"]);
    run_rfc_cli(&dir, &["new", "second RFC"]);
    run_rfc_cli(&dir, &["new", "third RFC"]);

    let output = run_rfc_cli(&dir, &["list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("0001"), "should contain 0001");
    assert!(stdout.contains("0002"), "should contain 0002");
    assert!(stdout.contains("0003"), "should contain 0003");
    assert!(stdout.contains("draft"), "should show draft status");

    // Verify order: 0001 appears before 0002, 0002 before 0003
    let pos1 = stdout.find("0001").unwrap();
    let pos2 = stdout.find("0002").unwrap();
    let pos3 = stdout.find("0003").unwrap();
    assert!(pos1 < pos2, "0001 should appear before 0002");
    assert!(pos2 < pos3, "0002 should appear before 0003");

    cleanup(&dir);
}

#[test]
fn test_list_filter_by_status() {
    let dir = create_temp_dir("list_filter");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "draft one"]);
    run_rfc_cli(&dir, &["new", "draft two"]);

    // Manually change second RFC to accepted
    write_rfc_with_status(&dir, "0002", "accepted one", "accepted");

    let output = run_rfc_cli(&dir, &["list", "--status", "draft"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("0001"), "should contain draft RFC 0001");
    assert!(
        !stdout.contains("0002"),
        "should NOT contain accepted RFC 0002"
    );

    cleanup(&dir);
}

#[test]
fn test_list_filter_no_match() {
    let dir = create_temp_dir("list_filter_none");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "a draft"]);

    let output = run_rfc_cli(&dir, &["list", "--status", "accepted"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("No RFCs found"),
        "should say 'No RFCs found.' when filter matches nothing, got: {}",
        stdout
    );

    cleanup(&dir);
}

// ============================================================
// Tests for `view` command
// ============================================================

#[test]
fn test_view_shows_content() {
    let dir = create_temp_dir("view_content");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "viewable RFC"]);

    let output = run_rfc_cli(&dir, &["view", "1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("RFC-0001: viewable RFC"),
        "should contain the RFC title, got: {}",
        stdout
    );
    assert!(
        stdout.contains("status: draft"),
        "should contain status field"
    );
    assert!(stdout.contains("## Problem"), "should contain RFC sections");

    cleanup(&dir);
}

#[test]
fn test_view_normalizes_number() {
    let dir = create_temp_dir("view_normalize");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "normalize test"]);

    // view 1 and view 0001 should produce identical output
    let output_short = run_rfc_cli(&dir, &["view", "1"]);
    let output_long = run_rfc_cli(&dir, &["view", "0001"]);

    let stdout_short = String::from_utf8_lossy(&output_short.stdout);
    let stdout_long = String::from_utf8_lossy(&output_long.stdout);

    assert!(output_short.status.success());
    assert!(output_long.status.success());
    assert_eq!(
        stdout_short, stdout_long,
        "view 1 and view 0001 should produce identical output"
    );

    cleanup(&dir);
}

#[test]
fn test_view_nonexistent_rfc() {
    let dir = create_temp_dir("view_missing");

    run_rfc_cli(&dir, &["init"]);

    let output = run_rfc_cli(&dir, &["view", "99"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("RFC-0099 not found"),
        "should report RFC-0099 not found, got: {}",
        stderr
    );

    cleanup(&dir);
}

#[test]
fn test_view_invalid_number() {
    let dir = create_temp_dir("view_invalid");

    run_rfc_cli(&dir, &["init"]);

    let output = run_rfc_cli(&dir, &["view", "abc"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("Invalid RFC number"),
        "should report invalid number, got: {}",
        stderr
    );

    cleanup(&dir);
}

// ============================================================
// Tests for `status` command
// ============================================================

#[test]
fn test_status_shows_status() {
    let dir = create_temp_dir("status_show");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "status test"]);

    let output = run_rfc_cli(&dir, &["status", "1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("RFC-0001: draft"),
        "should output 'RFC-0001: draft', got: {}",
        stdout
    );

    cleanup(&dir);
}

#[test]
fn test_status_normalizes_number() {
    let dir = create_temp_dir("status_normalize");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "normalize test"]);

    let output_short = run_rfc_cli(&dir, &["status", "1"]);
    let output_long = run_rfc_cli(&dir, &["status", "0001"]);

    let stdout_short = String::from_utf8_lossy(&output_short.stdout);
    let stdout_long = String::from_utf8_lossy(&output_long.stdout);

    assert!(output_short.status.success());
    assert!(output_long.status.success());
    assert_eq!(
        stdout_short, stdout_long,
        "status 1 and status 0001 should produce identical output"
    );

    cleanup(&dir);
}

#[test]
fn test_status_nonexistent_rfc() {
    let dir = create_temp_dir("status_missing");

    run_rfc_cli(&dir, &["init"]);

    let output = run_rfc_cli(&dir, &["status", "99"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("RFC-0099 not found"),
        "should report RFC-0099 not found, got: {}",
        stderr
    );

    cleanup(&dir);
}

// ============================================================
// Tests for `edit` command
// ============================================================

#[test]
fn test_edit_fails_without_editor() {
    let dir = create_temp_dir("edit_no_editor");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "editor test"]);

    let output = run_rfc_cli_without_editor(&dir, &["edit", "1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("EDITOR is not set"),
        "should report EDITOR is not set, got: {}",
        stderr
    );

    cleanup(&dir);
}

#[test]
fn test_edit_nonexistent_rfc() {
    let dir = create_temp_dir("edit_missing");

    run_rfc_cli(&dir, &["init"]);

    let output = run_rfc_cli_with_editor(&dir, &["edit", "99"], "true");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("RFC-0099 not found"),
        "should report RFC-0099 not found, got: {}",
        stderr
    );

    cleanup(&dir);
}

#[test]
fn test_edit_blocks_accepted_rfc() {
    let dir = create_temp_dir("edit_blocked");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "locked RFC"]);

    // Manually set status to accepted
    write_rfc_with_status(&dir, "0001", "locked RFC", "accepted");

    let output = run_rfc_cli_with_editor(&dir, &["edit", "1"], "true");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "edit should fail for accepted RFC without --force"
    );
    assert!(
        stderr.contains("accepted"),
        "error should mention 'accepted', got: {}",
        stderr
    );
    assert!(
        stderr.contains("--force"),
        "error should suggest --force, got: {}",
        stderr
    );

    cleanup(&dir);
}

#[test]
fn test_edit_force_allows_accepted_rfc() {
    let dir = create_temp_dir("edit_force");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "locked RFC"]);

    // Manually set status to accepted
    write_rfc_with_status(&dir, "0001", "locked RFC", "accepted");

    // "true" is a command that exits immediately with success
    let output = run_rfc_cli_with_editor(&dir, &["edit", "1", "--force"], "true");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "edit --force should succeed for accepted RFC, stderr: {}",
        stderr
    );
    assert!(
        stderr.contains("Warning"),
        "should print warning, got: {}",
        stderr
    );

    cleanup(&dir);
}

#[test]
fn test_edit_updates_index_after_save() {
    let dir = create_temp_dir("edit_index_update");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "original title"]);

    // Create a fake editor script that changes the title
    let editor = create_fake_editor_script(
        &dir,
        "fake_editor.sh",
        "sed -i.bak 's/original/changed/' \"$1\"",
    );

    let output = run_rfc_cli_with_editor(&dir, &["edit", "1"], &editor);
    assert!(
        output.status.success(),
        "edit should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify file was changed
    let content = fs::read_to_string(dir.join("docs/rfcs/0001.md")).unwrap();
    assert!(
        content.contains("changed title"),
        "file should contain 'changed title', got: {}",
        content
    );

    // Verify index was updated with new title
    let index_content = fs::read_to_string(dir.join("docs/rfcs/.index.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&index_content).unwrap();
    let title = parsed["rfcs"][0]["title"].as_str().unwrap();
    assert!(
        title.contains("changed"),
        "index title should contain 'changed', got: {}",
        title
    );

    cleanup(&dir);
}

#[test]
fn test_edit_normalizes_number() {
    let dir = create_temp_dir("edit_normalize");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "normalize test"]);

    // Both "edit 1" and "edit 0001" should work identically
    let output = run_rfc_cli_with_editor(&dir, &["edit", "1"], "true");
    assert!(
        output.status.success(),
        "edit 1 should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = run_rfc_cli_with_editor(&dir, &["edit", "0001"], "true");
    assert!(
        output.status.success(),
        "edit 0001 should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    cleanup(&dir);
}

// ============================================================
// Tests for `set` command
// ============================================================

#[test]
fn test_set_draft_to_review() {
    let dir = create_temp_dir("set_draft_review");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "transition test"]);

    let output = run_rfc_cli(&dir, &["set", "1", "review"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("draft → review"),
        "should show transition, got: {}",
        stdout
    );
    assert!(stdout.contains("✅"), "should show checkmark");

    // Verify frontmatter updated
    let content = fs::read_to_string(dir.join("docs/rfcs/0001.md")).unwrap();
    assert!(
        content.contains("status: review"),
        "frontmatter should say review, got: {}",
        content
    );

    // Verify index updated
    let index_content = fs::read_to_string(dir.join("docs/rfcs/.index.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&index_content).unwrap();
    assert_eq!(parsed["rfcs"][0]["status"], "review");

    cleanup(&dir);
}

#[test]
fn test_set_review_to_accepted_stores_content_hash() {
    let dir = create_temp_dir("set_accepted_hash");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "hash test"]);
    run_rfc_cli(&dir, &["set", "1", "review"]);
    run_rfc_cli(&dir, &["set", "1", "accepted"]);

    // Verify content_hash is set in index
    let index_content = fs::read_to_string(dir.join("docs/rfcs/.index.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&index_content).unwrap();
    assert!(
        !parsed["rfcs"][0]["content_hash"].is_null(),
        "content_hash should be set for accepted RFC, got: {}",
        parsed["rfcs"][0]["content_hash"]
    );

    cleanup(&dir);
}

#[test]
fn test_set_invalid_transition() {
    let dir = create_temp_dir("set_invalid");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "invalid test"]);

    // draft → implemented is not allowed
    let output = run_rfc_cli(&dir, &["set", "1", "implemented"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("not allowed"),
        "should say transition not allowed, got: {}",
        stderr
    );

    cleanup(&dir);
}

#[test]
fn test_set_invalid_status() {
    let dir = create_temp_dir("set_bad_status");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "bad status test"]);

    let output = run_rfc_cli(&dir, &["set", "1", "bogus"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    let stderr_lower = stderr.to_lowercase();
    assert!(
        stderr_lower.contains("invalid status")
            || stderr_lower.contains("invalid value")
            || stderr_lower.contains("possible values"),
        "should say invalid status, got: {}",
        stderr
    );

    cleanup(&dir);
}

#[test]
fn test_set_nonexistent_rfc() {
    let dir = create_temp_dir("set_missing");

    run_rfc_cli(&dir, &["init"]);

    let output = run_rfc_cli(&dir, &["set", "99", "review"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("RFC-0099 not found"),
        "should say RFC not found, got: {}",
        stderr
    );

    cleanup(&dir);
}

#[test]
fn test_set_superseded_requires_by() {
    let dir = create_temp_dir("set_superseded_no_by");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "old RFC"]);
    run_rfc_cli(&dir, &["set", "1", "review"]);
    run_rfc_cli(&dir, &["set", "1", "accepted"]);

    let output = run_rfc_cli(&dir, &["set", "1", "superseded"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("--by"),
        "should require --by flag, got: {}",
        stderr
    );

    cleanup(&dir);
}

#[test]
fn test_set_superseded_with_by() {
    let dir = create_temp_dir("set_superseded_ok");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "old RFC"]);
    run_rfc_cli(&dir, &["new", "new RFC"]);
    run_rfc_cli(&dir, &["set", "1", "review"]);
    run_rfc_cli(&dir, &["set", "1", "accepted"]);

    let output = run_rfc_cli(&dir, &["set", "1", "superseded", "--by", "2"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "set superseded --by should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("superseded"),
        "should mention superseded, got: {}",
        stdout
    );
    assert!(
        stdout.contains("RFC-0002"),
        "should mention replacing RFC, got: {}",
        stdout
    );

    // Verify frontmatter has superseded_by
    let content = fs::read_to_string(dir.join("docs/rfcs/0001.md")).unwrap();
    assert!(
        content.contains("superseded_by: RFC-0002"),
        "frontmatter should have superseded_by, got: {}",
        content
    );

    cleanup(&dir);
}

#[test]
fn test_set_superseded_by_nonexistent() {
    let dir = create_temp_dir("set_superseded_bad_by");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "old RFC"]);
    run_rfc_cli(&dir, &["set", "1", "review"]);
    run_rfc_cli(&dir, &["set", "1", "accepted"]);

    let output = run_rfc_cli(&dir, &["set", "1", "superseded", "--by", "99"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("RFC-0099 not found"),
        "should say replacing RFC not found, got: {}",
        stderr
    );

    cleanup(&dir);
}

#[test]
fn test_set_normalizes_number() {
    let dir = create_temp_dir("set_normalize");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "normalize test"]);

    let output = run_rfc_cli(&dir, &["set", "1", "review"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("RFC-0001"),
        "should show normalized number, got: {}",
        stdout
    );

    cleanup(&dir);
}

#[test]
fn test_set_full_lifecycle() {
    let dir = create_temp_dir("set_lifecycle");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "lifecycle test"]);

    // draft → review → accepted → implemented
    let o1 = run_rfc_cli(&dir, &["set", "1", "review"]);
    assert!(o1.status.success(), "draft → review should work");

    let o2 = run_rfc_cli(&dir, &["set", "1", "accepted"]);
    assert!(o2.status.success(), "review → accepted should work");

    let o3 = run_rfc_cli(&dir, &["set", "1", "implemented"]);
    assert!(o3.status.success(), "accepted → implemented should work");

    // Verify final state
    let output = run_rfc_cli(&dir, &["status", "1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("implemented"),
        "final status should be implemented, got: {}",
        stdout
    );

    cleanup(&dir);
}

// ============================================================
// Tests for `check` command
// ============================================================

#[test]
fn test_check_all_valid() {
    let dir = create_temp_dir("check_valid");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "valid RFC"]);

    let output = run_rfc_cli(&dir, &["check"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("All checks passed"),
        "should pass all checks, got: {}",
        stdout
    );

    cleanup(&dir);
}

#[test]
fn test_check_single_valid() {
    let dir = create_temp_dir("check_single");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "single check"]);

    let output = run_rfc_cli(&dir, &["check", "1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("All checks passed"),
        "should pass check, got: {}",
        stdout
    );

    cleanup(&dir);
}

#[test]
fn test_check_invalid_status() {
    let dir = create_temp_dir("check_bad_status");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "bad status"]);

    // Manually write invalid status
    let rfc_path = dir.join("docs/rfcs/0001.md");
    let content = fs::read_to_string(&rfc_path).unwrap();
    let bad = content.replace("status: draft", "status: bogus");
    fs::write(&rfc_path, bad).unwrap();

    let output = run_rfc_cli(&dir, &["check", "1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("invalid status"),
        "should report invalid status, got: {}",
        stderr
    );

    cleanup(&dir);
}

#[test]
fn test_check_missing_section() {
    let dir = create_temp_dir("check_missing_section");

    run_rfc_cli(&dir, &["init"]);

    // Write RFC without required sections
    let content = "---\ntitle: \"RFC-0001: incomplete\"\nstatus: draft\ndependencies: []\nsuperseded_by: null\nlinks: []\n---\n\nSome content without required sections.\n";
    fs::write(dir.join("docs/rfcs/0001.md"), content).unwrap();

    let output = run_rfc_cli(&dir, &["check", "1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("missing required section"),
        "should report missing section, got: {}",
        stderr
    );

    cleanup(&dir);
}

#[test]
fn test_check_dead_link() {
    let dir = create_temp_dir("check_dead_link");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "dead link test"]);

    // Add a non-existent file to links
    let rfc_path = dir.join("docs/rfcs/0001.md");
    let content = fs::read_to_string(&rfc_path).unwrap();
    let updated = content.replace("links: []", "links:\n  - src/nonexistent_file.rs");
    fs::write(&rfc_path, updated).unwrap();

    let output = run_rfc_cli(&dir, &["check", "1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("dead link"),
        "should report dead link, got: {}",
        stderr
    );

    cleanup(&dir);
}

#[test]
fn test_check_missing_dependency() {
    let dir = create_temp_dir("check_bad_dep");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "dep test"]);

    // Add non-existent dependency
    let rfc_path = dir.join("docs/rfcs/0001.md");
    let content = fs::read_to_string(&rfc_path).unwrap();
    let updated = content.replace("dependencies: []", "dependencies: [RFC-0099]");
    fs::write(&rfc_path, updated).unwrap();

    let output = run_rfc_cli(&dir, &["check", "1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("dependency") && stderr.contains("not found"),
        "should report missing dependency, got: {}",
        stderr
    );

    cleanup(&dir);
}

#[test]
fn test_check_content_hash_mismatch() {
    let dir = create_temp_dir("check_hash_mismatch");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "hash test"]);
    run_rfc_cli(&dir, &["set", "1", "review"]);
    run_rfc_cli(&dir, &["set", "1", "accepted"]);

    // Tamper with the file content without updating index
    let rfc_path = dir.join("docs/rfcs/0001.md");
    let content = fs::read_to_string(&rfc_path).unwrap();
    // Append text to body (after frontmatter) to change hash
    let tampered = format!("{}This line was added secretly.\n", content);
    fs::write(&rfc_path, tampered).unwrap();

    // Update mtime in index so refresh_index sees it, but keep old hash
    // Actually, refresh_index would catch this too. Let's just run check directly
    // which reads the file and compares against stored hash.
    let output = run_rfc_cli(&dir, &["check", "1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("content_hash mismatch"),
        "should detect hash mismatch, got: {}",
        stderr
    );

    cleanup(&dir);
}

#[test]
fn test_check_nonexistent_rfc() {
    let dir = create_temp_dir("check_missing");

    run_rfc_cli(&dir, &["init"]);

    let output = run_rfc_cli(&dir, &["check", "99"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("RFC-0099 not found"),
        "should report RFC not found, got: {}",
        stderr
    );

    cleanup(&dir);
}

#[test]
fn test_check_exit_code_zero_on_success() {
    let dir = create_temp_dir("check_exit_ok");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "exit code test"]);

    let output = run_rfc_cli(&dir, &["check"]);

    assert!(output.status.success(), "exit code should be 0 on success");

    cleanup(&dir);
}

// ============================================================
// Tests for `reindex` command
// ============================================================

#[test]
fn test_reindex_rebuilds_index() {
    let dir = create_temp_dir("reindex_rebuild");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "first"]);
    run_rfc_cli(&dir, &["new", "second"]);
    run_rfc_cli(&dir, &["new", "third"]);

    let output = run_rfc_cli(&dir, &["reindex"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("Reindexed 3 RFCs"),
        "should say 3 RFCs reindexed, got: {}",
        stdout
    );

    // Verify index has correct entries
    let index_content = fs::read_to_string(dir.join("docs/rfcs/.index.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&index_content).unwrap();
    let rfcs = parsed["rfcs"].as_array().unwrap();
    assert_eq!(rfcs.len(), 3);
    assert_eq!(rfcs[0]["number"], "0001");
    assert_eq!(rfcs[1]["number"], "0002");
    assert_eq!(rfcs[2]["number"], "0003");

    cleanup(&dir);
}

#[test]
fn test_reindex_recovers_corrupted_index() {
    let dir = create_temp_dir("reindex_corrupted");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "will survive"]);
    run_rfc_cli(&dir, &["new", "also survives"]);

    // Corrupt the index
    fs::write(dir.join("docs/rfcs/.index.json"), "CORRUPTED").unwrap();

    let output = run_rfc_cli(&dir, &["reindex"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "reindex should succeed even with corrupted index, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("Reindexed 2 RFCs"),
        "should reindex 2 RFCs, got: {}",
        stdout
    );

    // Verify the index is now valid
    let index_content = fs::read_to_string(dir.join("docs/rfcs/.index.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&index_content).unwrap();
    let rfcs = parsed["rfcs"].as_array().unwrap();
    assert_eq!(rfcs.len(), 2);

    cleanup(&dir);
}

#[test]
fn test_reindex_accepted_gets_content_hash() {
    let dir = create_temp_dir("reindex_hash");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "hash via reindex"]);
    run_rfc_cli(&dir, &["set", "1", "review"]);
    run_rfc_cli(&dir, &["set", "1", "accepted"]);

    // Clear index and rebuild
    fs::write(dir.join("docs/rfcs/.index.json"), "{\"rfcs\": []}").unwrap();

    run_rfc_cli(&dir, &["reindex"]);

    let index_content = fs::read_to_string(dir.join("docs/rfcs/.index.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&index_content).unwrap();
    assert!(
        !parsed["rfcs"][0]["content_hash"].is_null(),
        "reindex should compute content_hash for accepted RFC"
    );

    cleanup(&dir);
}

#[test]
fn test_reindex_draft_has_no_content_hash() {
    let dir = create_temp_dir("reindex_no_hash");

    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "draft rfc"]);

    run_rfc_cli(&dir, &["reindex"]);

    let index_content = fs::read_to_string(dir.join("docs/rfcs/.index.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&index_content).unwrap();
    assert!(
        parsed["rfcs"][0]["content_hash"].is_null(),
        "reindex should NOT set content_hash for draft RFC"
    );

    cleanup(&dir);
}

#[test]
fn test_reindex_without_init() {
    let dir = create_temp_dir("reindex_no_init");

    let output = run_rfc_cli(&dir, &["reindex"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("rfc-cli init"),
        "should suggest running init, got: {}",
        stderr
    );

    cleanup(&dir);
}

#[test]
fn test_reindex_message_format() {
    let dir = create_temp_dir("reindex_msg");

    run_rfc_cli(&dir, &["init"]);

    let output = run_rfc_cli(&dir, &["reindex"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("Reindexed 0 RFCs"),
        "empty project should say 0 RFCs, got: {}",
        stdout
    );

    cleanup(&dir);
}

// ============================================================
// Tests for `help` output with all commands
// ============================================================

#[test]
fn test_help_shows_new_commands() {
    let dir = create_temp_dir("help_new_cmds");

    let output = run_rfc_cli(&dir, &["--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("list"), "help should mention list");
    assert!(stdout.contains("view"), "help should mention view");
    assert!(stdout.contains("status"), "help should mention status");
    assert!(stdout.contains("edit"), "help should mention edit");
    assert!(stdout.contains("set"), "help should mention set");
    assert!(stdout.contains("check"), "help should mention check");
    assert!(stdout.contains("reindex"), "help should mention reindex");
    assert!(stdout.contains("link"), "help should mention link");
    assert!(stdout.contains("unlink"), "help should mention unlink");
    assert!(stdout.contains("deps"), "help should mention deps");

    cleanup(&dir);
}

// ==================== link ====================

#[test]
fn test_link_basic() {
    let dir = create_temp_dir("link_basic");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "test link"]);

    // Create a dummy file to link to
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    let output = run_rfc_cli(&dir, &["link", "1", "src/main.rs"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("linked src/main.rs"));

    // Verify frontmatter
    let content = fs::read_to_string(dir.join("docs/rfcs/0001.md")).unwrap();
    assert!(
        content.contains("src/main.rs"),
        "frontmatter should contain the link"
    );

    // Verify index
    let index = fs::read_to_string(dir.join("docs/rfcs/.index.json")).unwrap();
    assert!(
        index.contains("src/main.rs"),
        "index should contain the link"
    );

    cleanup(&dir);
}

#[test]
fn test_link_duplicate() {
    let dir = create_temp_dir("link_dup");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "test dup"]);

    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    run_rfc_cli(&dir, &["link", "1", "src/main.rs"]);
    let output = run_rfc_cli(&dir, &["link", "1", "src/main.rs"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("link already exists"));

    cleanup(&dir);
}

#[test]
fn test_link_nonexistent_file() {
    let dir = create_temp_dir("link_nofile");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "test nofile"]);

    let output = run_rfc_cli(&dir, &["link", "1", "src/nonexistent.rs"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("File not found"));

    cleanup(&dir);
}

#[test]
fn test_link_nonexistent_rfc() {
    let dir = create_temp_dir("link_norfc");
    run_rfc_cli(&dir, &["init"]);

    let output = run_rfc_cli(&dir, &["link", "99", "src/main.rs"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("not found"));

    cleanup(&dir);
}

#[test]
fn test_link_normalizes_path() {
    let dir = create_temp_dir("link_normalize");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "test normalize"]);

    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    let output = run_rfc_cli(&dir, &["link", "1", "./src/main.rs"]);
    assert!(output.status.success());

    let content = fs::read_to_string(dir.join("docs/rfcs/0001.md")).unwrap();
    assert!(content.contains("src/main.rs"), "path should be normalized");
    assert!(
        !content.contains("./src/main.rs"),
        "leading ./ should be stripped"
    );

    cleanup(&dir);
}

#[test]
fn test_link_accepted_blocked() {
    let dir = create_temp_dir("link_acc_block");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "test accepted"]);

    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    // Promote to accepted
    run_rfc_cli(&dir, &["set", "1", "review"]);
    run_rfc_cli(&dir, &["set", "1", "accepted"]);

    let output = run_rfc_cli(&dir, &["link", "1", "src/main.rs"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("accepted"));
    assert!(stderr.contains("--force"));

    cleanup(&dir);
}

#[test]
fn test_link_accepted_force() {
    let dir = create_temp_dir("link_acc_force");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "test force"]);

    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    run_rfc_cli(&dir, &["set", "1", "review"]);
    run_rfc_cli(&dir, &["set", "1", "accepted"]);

    let output = run_rfc_cli(&dir, &["link", "1", "src/main.rs", "--force"]);
    assert!(output.status.success());

    // Verify content_hash was updated in index
    let index = fs::read_to_string(dir.join("docs/rfcs/.index.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&index).unwrap();
    let entry = parsed["rfcs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["number"].as_str() == Some("0001"))
        .unwrap();
    assert!(
        entry["content_hash"].as_str().is_some(),
        "content_hash should be set after --force"
    );

    cleanup(&dir);
}

#[test]
fn test_link_multiple() {
    let dir = create_temp_dir("link_multi");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "test multi"]);

    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();
    fs::write(src_dir.join("lib.rs"), "// lib").unwrap();
    fs::write(src_dir.join("cli.rs"), "// cli").unwrap();

    run_rfc_cli(&dir, &["link", "1", "src/main.rs"]);
    run_rfc_cli(&dir, &["link", "1", "src/lib.rs"]);
    run_rfc_cli(&dir, &["link", "1", "src/cli.rs"]);

    let content = fs::read_to_string(dir.join("docs/rfcs/0001.md")).unwrap();
    assert!(content.contains("src/main.rs"));
    assert!(content.contains("src/lib.rs"));
    assert!(content.contains("src/cli.rs"));

    let index = fs::read_to_string(dir.join("docs/rfcs/.index.json")).unwrap();
    assert!(index.contains("src/main.rs"));
    assert!(index.contains("src/lib.rs"));
    assert!(index.contains("src/cli.rs"));

    cleanup(&dir);
}

// ==================== unlink ====================

#[test]
fn test_unlink_basic() {
    let dir = create_temp_dir("unlink_basic");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "test unlink"]);

    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    run_rfc_cli(&dir, &["link", "1", "src/main.rs"]);

    let output = run_rfc_cli(&dir, &["unlink", "1", "src/main.rs"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("unlinked src/main.rs"));

    let content = fs::read_to_string(dir.join("docs/rfcs/0001.md")).unwrap();
    assert!(
        !content.contains("  - src/main.rs"),
        "link should be removed from frontmatter"
    );

    cleanup(&dir);
}

#[test]
fn test_unlink_not_found() {
    let dir = create_temp_dir("unlink_notfound");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "test unlink nf"]);

    let output = run_rfc_cli(&dir, &["unlink", "1", "src/nonexistent.rs"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("link not found"));

    cleanup(&dir);
}

#[test]
fn test_unlink_nonexistent_rfc() {
    let dir = create_temp_dir("unlink_norfc");
    run_rfc_cli(&dir, &["init"]);

    let output = run_rfc_cli(&dir, &["unlink", "99", "src/main.rs"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("not found"));

    cleanup(&dir);
}

#[test]
fn test_unlink_accepted_blocked() {
    let dir = create_temp_dir("unlink_acc_block");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "test unlink acc"]);

    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    run_rfc_cli(&dir, &["link", "1", "src/main.rs"]);
    run_rfc_cli(&dir, &["set", "1", "review"]);
    run_rfc_cli(&dir, &["set", "1", "accepted"]);

    let output = run_rfc_cli(&dir, &["unlink", "1", "src/main.rs"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("accepted"));
    assert!(stderr.contains("--force"));

    cleanup(&dir);
}

#[test]
fn test_unlink_accepted_force() {
    let dir = create_temp_dir("unlink_acc_force");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "test unlink force"]);

    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    run_rfc_cli(&dir, &["link", "1", "src/main.rs"]);
    run_rfc_cli(&dir, &["set", "1", "review"]);
    run_rfc_cli(&dir, &["set", "1", "accepted"]);

    let output = run_rfc_cli(&dir, &["unlink", "1", "src/main.rs", "--force"]);
    assert!(output.status.success());

    let content = fs::read_to_string(dir.join("docs/rfcs/0001.md")).unwrap();
    assert!(
        content.contains("links: []"),
        "links should be empty after unlink"
    );

    // Verify content_hash was updated
    let index = fs::read_to_string(dir.join("docs/rfcs/.index.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&index).unwrap();
    let entry = parsed["rfcs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["number"].as_str() == Some("0001"))
        .unwrap();
    assert!(
        entry["content_hash"].as_str().is_some(),
        "content_hash should be updated after --force"
    );

    cleanup(&dir);
}

#[test]
fn test_unlink_last_link() {
    let dir = create_temp_dir("unlink_last");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "test last link"]);

    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    run_rfc_cli(&dir, &["link", "1", "src/main.rs"]);
    run_rfc_cli(&dir, &["unlink", "1", "src/main.rs"]);

    let content = fs::read_to_string(dir.join("docs/rfcs/0001.md")).unwrap();
    assert!(content.contains("links: []"), "links should be empty list");

    cleanup(&dir);
}

// ==================== deps ====================

#[test]
fn test_deps_basic() {
    let dir = create_temp_dir("deps_basic");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "base"]);
    run_rfc_cli(&dir, &["new", "depends on base"]);

    // Write RFC-0002 with dependency on RFC-0001
    write_rfc_with_deps_and_links(&dir, "0002", "depends on base", "draft", &["RFC-0001"], &[]);

    let output = run_rfc_cli(&dir, &["deps", "2"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("RFC-0002 depends on:"));
    assert!(stdout.contains("RFC-0001"));

    cleanup(&dir);
}

#[test]
fn test_deps_no_dependencies() {
    let dir = create_temp_dir("deps_none");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "standalone"]);

    let output = run_rfc_cli(&dir, &["deps", "1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("has no dependencies"));

    cleanup(&dir);
}

#[test]
fn test_deps_missing_dependency() {
    let dir = create_temp_dir("deps_missing");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "has missing dep"]);

    // Write RFC with dependency on nonexistent RFC-0099
    write_rfc_with_deps_and_links(&dir, "0001", "has missing dep", "draft", &["RFC-0099"], &[]);

    let output = run_rfc_cli(&dir, &["deps", "1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("not found"));

    cleanup(&dir);
}

#[test]
fn test_deps_reverse() {
    let dir = create_temp_dir("deps_reverse");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "base rfc"]);
    run_rfc_cli(&dir, &["new", "child one"]);
    run_rfc_cli(&dir, &["new", "child two"]);

    write_rfc_with_deps_and_links(&dir, "0002", "child one", "draft", &["RFC-0001"], &[]);
    write_rfc_with_deps_and_links(&dir, "0003", "child two", "draft", &["RFC-0001"], &[]);

    let output = run_rfc_cli(&dir, &["deps", "1", "--reverse"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("is depended on by:"));
    assert!(stdout.contains("RFC-0002"));
    assert!(stdout.contains("RFC-0003"));

    cleanup(&dir);
}

#[test]
fn test_deps_reverse_none() {
    let dir = create_temp_dir("deps_rev_none");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "leaf node"]);

    let output = run_rfc_cli(&dir, &["deps", "1", "--reverse"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("has no reverse dependencies"));

    cleanup(&dir);
}

#[test]
fn test_deps_nonexistent_rfc() {
    let dir = create_temp_dir("deps_norfc");
    run_rfc_cli(&dir, &["init"]);

    let output = run_rfc_cli(&dir, &["deps", "99"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("not found"));

    cleanup(&dir);
}

// ==================== doctor ====================

#[test]
fn test_doctor_healthy_project() {
    let dir = create_temp_dir("doctor_healthy");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "healthy rfc"]);

    let output = run_rfc_cli(&dir, &["doctor"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("All RFCs are healthy."));

    cleanup(&dir);
}

#[test]
fn test_doctor_empty_project() {
    let dir = create_temp_dir("doctor_empty");
    run_rfc_cli(&dir, &["init"]);

    let output = run_rfc_cli(&dir, &["doctor"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("All RFCs are healthy."));

    cleanup(&dir);
}

#[test]
fn test_doctor_code_drift() {
    let dir = create_temp_dir("doctor_drift");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "drift test"]);

    // Create a source file, link it, then promote to accepted
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    run_rfc_cli(&dir, &["link", "1", "src/main.rs"]);
    run_rfc_cli(&dir, &["set", "1", "review"]);
    run_rfc_cli(&dir, &["set", "1", "accepted"]);

    // Now modify the linked file so its mtime is newer than the RFC
    std::thread::sleep(std::time::Duration::from_millis(1100));
    fs::write(src_dir.join("main.rs"), "fn main() { /* changed */ }").unwrap();

    let output = run_rfc_cli(&dir, &["doctor"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(!output.status.success());
    assert!(
        combined.contains("code drift"),
        "should report code drift, got: {}",
        combined
    );

    cleanup(&dir);
}

#[test]
fn test_doctor_no_drift() {
    let dir = create_temp_dir("doctor_no_drift");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "no drift"]);

    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    run_rfc_cli(&dir, &["link", "1", "src/main.rs"]);
    run_rfc_cli(&dir, &["set", "1", "review"]);
    run_rfc_cli(&dir, &["set", "1", "accepted"]);

    // Don't modify the linked file — no drift
    let output = run_rfc_cli(&dir, &["doctor"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should not contain code drift for this RFC
    assert!(
        !stdout.contains("code drift"),
        "should not report code drift"
    );

    cleanup(&dir);
}

#[test]
fn test_doctor_no_implementation() {
    let dir = create_temp_dir("doctor_no_impl");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "no impl"]);

    run_rfc_cli(&dir, &["set", "1", "review"]);
    run_rfc_cli(&dir, &["set", "1", "accepted"]);

    let output = run_rfc_cli(&dir, &["doctor"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "warnings only → exit 0");
    assert!(
        stdout.contains("no linked files"),
        "should warn about no links, got: {}",
        stdout
    );

    cleanup(&dir);
}

#[test]
fn test_doctor_accepted_with_links() {
    let dir = create_temp_dir("doctor_acc_links");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "has links"]);

    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    run_rfc_cli(&dir, &["link", "1", "src/main.rs"]);
    run_rfc_cli(&dir, &["set", "1", "review"]);
    run_rfc_cli(&dir, &["set", "1", "accepted"]);

    let output = run_rfc_cli(&dir, &["doctor"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stdout.contains("no linked files"),
        "should not warn about no links"
    );

    cleanup(&dir);
}

#[test]
fn test_doctor_dead_link() {
    let dir = create_temp_dir("doctor_dead");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "dead link"]);

    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    run_rfc_cli(&dir, &["link", "1", "src/main.rs"]);

    // Delete the linked file
    fs::remove_file(src_dir.join("main.rs")).unwrap();

    let output = run_rfc_cli(&dir, &["doctor"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(!output.status.success());
    assert!(
        combined.contains("dead link"),
        "should report dead link, got: {}",
        combined
    );

    cleanup(&dir);
}

#[test]
fn test_doctor_stale_draft() {
    let dir = create_temp_dir("doctor_stale");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "stale rfc"]);

    // Use --stale-days 0 so that any draft (even just created) counts as stale.
    // This avoids needing to manipulate file mtime on disk.
    let output = run_rfc_cli(&dir, &["doctor", "--stale-days", "0"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stale draft is warning only → exit 0"
    );
    assert!(
        stdout.contains("stale draft"),
        "should warn about stale draft, got: {}",
        stdout
    );

    cleanup(&dir);
}

#[test]
fn test_doctor_fresh_draft() {
    let dir = create_temp_dir("doctor_fresh");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "fresh rfc"]);

    // Default stale-days is 30, a just-created RFC should not be stale
    let output = run_rfc_cli(&dir, &["doctor"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stdout.contains("stale draft"),
        "fresh draft should not be stale"
    );

    cleanup(&dir);
}

#[test]
fn test_doctor_stale_days_flag() {
    let dir = create_temp_dir("doctor_stale_flag");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "any draft"]);

    // With --stale-days 0, any draft is stale
    let output = run_rfc_cli(&dir, &["doctor", "--stale-days", "0"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stale draft is warning only → exit 0"
    );
    assert!(
        stdout.contains("stale draft"),
        "with --stale-days 0 every draft should be stale, got: {}",
        stdout
    );

    cleanup(&dir);
}

#[test]
fn test_doctor_unresolved_deps() {
    let dir = create_temp_dir("doctor_unres_deps");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "base"]);
    run_rfc_cli(&dir, &["new", "depends on base"]);

    // RFC-0001 stays draft, RFC-0002 depends on it and gets accepted
    write_rfc_with_deps_and_links(&dir, "0002", "depends on base", "draft", &["RFC-0001"], &[]);
    run_rfc_cli(&dir, &["set", "2", "review"]);
    run_rfc_cli(&dir, &["set", "2", "accepted"]);

    let output = run_rfc_cli(&dir, &["doctor"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "unresolved deps is warning → exit 0"
    );
    assert!(
        stdout.contains("depends on RFC-0001") && stdout.contains("still in"),
        "should warn about unresolved dependency, got: {}",
        stdout
    );

    cleanup(&dir);
}

#[test]
fn test_doctor_resolved_deps() {
    let dir = create_temp_dir("doctor_res_deps");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "base"]);
    run_rfc_cli(&dir, &["new", "depends on base"]);

    // Accept both
    run_rfc_cli(&dir, &["set", "1", "review"]);
    run_rfc_cli(&dir, &["set", "1", "accepted"]);

    write_rfc_with_deps_and_links(&dir, "0002", "depends on base", "draft", &["RFC-0001"], &[]);
    run_rfc_cli(&dir, &["set", "2", "review"]);
    run_rfc_cli(&dir, &["set", "2", "accepted"]);

    let output = run_rfc_cli(&dir, &["doctor"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stdout.contains("still in"),
        "resolved deps should not trigger warning"
    );

    cleanup(&dir);
}

#[test]
fn test_doctor_cycle_detected() {
    let dir = create_temp_dir("doctor_cycle");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "alpha"]);
    run_rfc_cli(&dir, &["new", "beta"]);

    // Create a cycle: 0001 depends on 0002, 0002 depends on 0001
    write_rfc_with_deps_and_links(&dir, "0001", "alpha", "draft", &["RFC-0002"], &[]);
    write_rfc_with_deps_and_links(&dir, "0002", "beta", "draft", &["RFC-0001"], &[]);

    let output = run_rfc_cli(&dir, &["doctor"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(!output.status.success(), "cycles are errors → exit 1");
    assert!(
        combined.contains("circular dependency"),
        "should report cycle, got: {}",
        combined
    );

    cleanup(&dir);
}

#[test]
fn test_doctor_no_cycle() {
    let dir = create_temp_dir("doctor_no_cycle");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "first"]);
    run_rfc_cli(&dir, &["new", "second"]);

    // Linear: 0002 depends on 0001 — no cycle
    write_rfc_with_deps_and_links(&dir, "0002", "second", "draft", &["RFC-0001"], &[]);

    let output = run_rfc_cli(&dir, &["doctor"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stdout.contains("circular dependency"),
        "should not report cycle for linear deps"
    );

    cleanup(&dir);
}

#[test]
fn test_doctor_exit_code_error() {
    let dir = create_temp_dir("doctor_exit_err");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "with dead link"]);

    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    run_rfc_cli(&dir, &["link", "1", "src/main.rs"]);
    fs::remove_file(src_dir.join("main.rs")).unwrap();

    let output = run_rfc_cli(&dir, &["doctor"]);

    assert!(
        !output.status.success(),
        "errors should cause non-zero exit code"
    );

    cleanup(&dir);
}

#[test]
fn test_doctor_exit_code_warning_only() {
    let dir = create_temp_dir("doctor_exit_warn");
    run_rfc_cli(&dir, &["init"]);
    run_rfc_cli(&dir, &["new", "warning only"]);

    // Accepted RFC without links → warning only
    run_rfc_cli(&dir, &["set", "1", "review"]);
    run_rfc_cli(&dir, &["set", "1", "accepted"]);

    let output = run_rfc_cli(&dir, &["doctor"]);

    assert!(output.status.success(), "warnings only should still exit 0");

    cleanup(&dir);
}
