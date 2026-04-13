use std::fs;
use std::path::Path;

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
fn run_rfc_cli(project_dir: &Path, args: &[&str]) -> std::process::Output {
    let binary = env!("CARGO_BIN_EXE_rfc-cli");
    std::process::Command::new(binary)
        .args(args)
        .env("RFC_HOME", project_dir.as_os_str())
        .output()
        .expect("Failed to execute rfc-cli")
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
    assert!(content.contains("## Проблема"));
    assert!(content.contains("## Задача"));
    assert!(content.contains("## Дизайн"));
    assert!(content.contains("## Альтернативы"));
    assert!(content.contains("## Голосование"));
    assert!(content.contains("## Миграция"));

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
    let required_sections = vec!["## Проблема", "## Задача", "## Дизайн", "## Альтернативы"];

    // Optional sections per RFC-0001
    let optional_sections = vec!["## Голосование", "## Миграция"];

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
