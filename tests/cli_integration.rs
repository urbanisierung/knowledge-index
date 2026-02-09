//! Integration tests for knowledge-index CLI.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Get the path to the built binary
fn binary_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove 'deps'
    path.push("knowledge-index");

    // On Windows, add .exe extension
    #[cfg(windows)]
    {
        path.set_extension("exe");
    }

    path
}

/// Create a Command with an isolated config directory for testing
fn test_command(config_dir: &std::path::Path) -> Command {
    let mut cmd = Command::new(binary_path());
    cmd.env("KNOWLEDGE_INDEX_CONFIG_DIR", config_dir);
    cmd
}

/// Create a temporary test directory with sample files
fn create_test_repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();

    // Create some test files
    fs::write(
        tmp.path().join("README.md"),
        "# Test Project\n\nThis is a test project.",
    )
    .unwrap();
    fs::write(
        tmp.path().join("main.rs"),
        "fn main() {\n    println!(\"Hello\");\n}",
    )
    .unwrap();
    fs::write(
        tmp.path().join("lib.rs"),
        "pub fn greet(name: &str) -> String {\n    format!(\"Hello, {name}!\")\n}",
    )
    .unwrap();

    // Create a subdirectory
    let src_dir = tmp.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(
        src_dir.join("utils.rs"),
        "pub fn helper() -> i32 {\n    42\n}",
    )
    .unwrap();

    // Create a markdown file with frontmatter
    fs::write(
        tmp.path().join("notes.md"),
        r"---
title: My Notes
tags: [test, notes]
---

# Notes

Some [[wiki-link]] content here.

## Section 1

More content.
",
    )
    .unwrap();

    tmp
}

#[test]
fn test_cli_help() {
    let output = Command::new(binary_path())
        .arg("--help")
        .output()
        .expect("Failed to run binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("knowledge-index"));
    assert!(stdout.contains("index"));
    assert!(stdout.contains("search"));
}

#[test]
fn test_cli_version() {
    let output = Command::new(binary_path())
        .arg("--version")
        .output()
        .expect("Failed to run binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("knowledge-index"));
}

#[test]
fn test_cli_config_show() {
    let config_dir = tempfile::tempdir().unwrap();
    let output = test_command(config_dir.path())
        .arg("config")
        .output()
        .expect("Failed to run binary");

    // Config should work (might be first run)
    assert!(output.status.success() || output.status.code() == Some(0));
}

#[test]
fn test_cli_list_empty() {
    let config_dir = tempfile::tempdir().unwrap();
    let output = test_command(config_dir.path())
        .arg("list")
        .arg("--json")
        .output()
        .expect("Failed to run binary");

    assert!(
        output.status.success(),
        "list --json failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("repositories") || stdout.contains("[]"));
}

#[test]
fn test_cli_search_no_results() {
    let config_dir = tempfile::tempdir().unwrap();
    let output = test_command(config_dir.path())
        .args(["search", "nonexistent_term_12345", "--json"])
        .output()
        .expect("Failed to run binary");

    assert!(
        output.status.success(),
        "search --json failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should return empty results
    assert!(stdout.contains("results") || stdout.contains("[]"));
}

#[test]
#[ignore = "Requires full index cycle, run with --ignored"]
fn test_full_index_search_cycle() {
    let config_dir = tempfile::tempdir().unwrap();
    let test_dir = create_test_repo();
    let test_path = test_dir.path().to_string_lossy().to_string();

    // Index the test directory
    let output = test_command(config_dir.path())
        .args(["index", &test_path, "--quiet"])
        .output()
        .expect("Failed to run index");

    assert!(
        output.status.success(),
        "Index failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Search for known content
    let output = test_command(config_dir.path())
        .args(["search", "greet", "--json"])
        .output()
        .expect("Failed to run search");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("lib.rs") || stdout.contains("greet"));

    // Search for markdown content
    let output = test_command(config_dir.path())
        .args(["search", "wiki-link", "--json"])
        .output()
        .expect("Failed to run search");

    assert!(output.status.success());

    // Clean up - remove the indexed repo
    let output = test_command(config_dir.path())
        .args(["remove", &test_path, "--force"])
        .output()
        .expect("Failed to run remove");

    assert!(output.status.success());
}
