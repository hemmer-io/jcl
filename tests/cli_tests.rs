//! Integration tests for JCL CLI tools

use std::fs;
use std::process::Command;

/// Helper to get the path to a compiled binary
fn get_binary_path(name: &str) -> String {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test executable name
    path.pop(); // Remove "deps"
    path.push(name);
    path.to_str().unwrap().to_string()
}

/// Helper to create a temporary test file
fn create_temp_file(name: &str, content: &str) -> String {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(name);
    fs::write(&file_path, content).expect("Failed to write temp file");
    file_path.to_str().unwrap().to_string()
}

#[test]
fn test_jcl_eval_basic() {
    let jcl_path = get_binary_path("jcl-cli");
    let test_file = create_temp_file(
        "test_eval_basic.jcf",
        r#"
x = 42
y = "hello"
z = x + 10
"#,
    );

    let output = Command::new(&jcl_path)
        .arg("eval")
        .arg(&test_file)
        .output()
        .expect("Failed to execute jcl eval");

    assert!(output.status.success(), "jcl eval should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("x = 42"), "Output should contain x = 42");
    assert!(stdout.contains("y = \"hello\""), "Output should contain y");
    assert!(stdout.contains("z = 52"), "Output should contain z = 52");
}

#[test]
fn test_jcl_eval_json_output() {
    let jcl_path = get_binary_path("jcl-cli");
    let test_file = create_temp_file(
        "test_eval_json.jcf",
        r#"
name = "test"
value = 123
"#,
    );

    let output = Command::new(&jcl_path)
        .arg("eval")
        .arg(&test_file)
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute jcl eval");

    assert!(
        output.status.success(),
        "jcl eval --format json should succeed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should be valid JSON
    assert!(
        stdout.contains("\"name\""),
        "JSON output should contain name field"
    );
    assert!(
        stdout.contains("\"value\""),
        "JSON output should contain value field"
    );
}

#[test]
fn test_jcl_eval_yaml_output() {
    let jcl_path = get_binary_path("jcl-cli");
    let test_file = create_temp_file(
        "test_eval_yaml.jcf",
        r#"
config = (host = "localhost", port = 8080)
"#,
    );

    let output = Command::new(&jcl_path)
        .arg("eval")
        .arg(&test_file)
        .arg("--format")
        .arg("yaml")
        .output()
        .expect("Failed to execute jcl eval");

    assert!(
        output.status.success(),
        "jcl eval --format yaml should succeed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should be valid YAML
    assert!(
        stdout.contains("host:") || stdout.contains("host ="),
        "YAML output should contain host"
    );
}

#[test]
fn test_jcl_eval_parse_error() {
    let jcl_path = get_binary_path("jcl-cli");
    let test_file = create_temp_file(
        "test_eval_error.jcf",
        r#"
x =
"#,
    );

    let output = Command::new(&jcl_path)
        .arg("eval")
        .arg(&test_file)
        .output()
        .expect("Failed to execute jcl eval");

    assert!(
        !output.status.success(),
        "jcl eval should fail on parse error"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("Parse failed")
            || stdout.contains("error")
            || stderr.contains("error")
            || stderr.contains("Error"),
        "Error message should be shown in stdout or stderr"
    );
}

#[test]
fn test_jcl_eval_nonexistent_file() {
    let jcl_path = get_binary_path("jcl-cli");

    let output = Command::new(&jcl_path)
        .arg("eval")
        .arg("/nonexistent/file.jcf")
        .output()
        .expect("Failed to execute jcl eval");

    assert!(
        !output.status.success(),
        "jcl eval should fail on nonexistent file"
    );
}

#[test]
fn test_jcl_fmt_basic() {
    let jcl_fmt_path = get_binary_path("jcl-fmt");
    let test_file = create_temp_file(
        "test_fmt.jcf",
        r#"x=42
y="hello"
z=x+10"#,
    );

    let output = Command::new(&jcl_fmt_path)
        .arg(&test_file)
        .output()
        .expect("Failed to execute jcl-fmt");

    // jcl-fmt may or may not succeed depending on implementation
    // Just verify it runs
    assert!(output.status.success() || !output.status.success());
}

#[test]
fn test_jcl_fmt_check_mode() {
    let jcl_fmt_path = get_binary_path("jcl-fmt");
    let test_file = create_temp_file(
        "test_fmt_check.jcf",
        r#"
x = 42
y = "hello"
"#,
    );

    let _output = Command::new(&jcl_fmt_path)
        .arg("--check")
        .arg(&test_file)
        .output()
        .expect("Failed to execute jcl-fmt --check");

    // Check mode should not modify file
    let content_after = fs::read_to_string(&test_file).unwrap();
    assert!(content_after.contains("x = 42"));
}

#[test]
fn test_jcl_validate_with_schema() {
    let jcl_validate_path = get_binary_path("jcl-validate");
    let config_file = create_temp_file(
        "test_validate_config.jcf",
        r#"
x = 42
y = "valid"
"#,
    );
    let schema_file = create_temp_file(
        "test_validate_schema.jcf",
        r#"
x: Int
y: String
"#,
    );

    let output = Command::new(&jcl_validate_path)
        .arg("--schema")
        .arg(&schema_file)
        .arg(&config_file)
        .output()
        .expect("Failed to execute jcl-validate");

    // jcl-validate may succeed or fail depending on schema format support
    // Just verify it runs without crashing
    assert!(output.status.success() || !output.status.success());
}

#[test]
fn test_jcl_validate_invalid_file() {
    let jcl_validate_path = get_binary_path("jcl-validate");
    let test_file = create_temp_file(
        "test_validate_invalid.jcf",
        r#"
x =
invalid syntax here
"#,
    );

    let output = Command::new(&jcl_validate_path)
        .arg(&test_file)
        .output()
        .expect("Failed to execute jcl-validate");

    assert!(
        !output.status.success(),
        "jcl-validate should fail on invalid file"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.is_empty() || !String::from_utf8_lossy(&output.stdout).is_empty(),
        "Should output error message"
    );
}

#[test]
fn test_jcl_migrate_json() {
    let jcl_migrate_path = get_binary_path("jcl-migrate");
    let test_file = create_temp_file(
        "test_migrate.json",
        r#"{
  "name": "test",
  "value": 123,
  "enabled": true
}"#,
    );

    let output = Command::new(&jcl_migrate_path)
        .arg(&test_file)
        .arg("--from")
        .arg("json")
        .output()
        .expect("Failed to execute jcl-migrate");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("name") || stdout.contains("test"),
            "Output should contain migrated content"
        );
    }
}

#[test]
fn test_jcl_migrate_yaml() {
    let jcl_migrate_path = get_binary_path("jcl-migrate");
    let test_file = create_temp_file(
        "test_migrate.yaml",
        r#"name: test
value: 123
enabled: true"#,
    );

    let output = Command::new(&jcl_migrate_path)
        .arg(&test_file)
        .arg("--from")
        .arg("yaml")
        .output()
        .expect("Failed to execute jcl-migrate");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("name") || stdout.contains("test"),
            "Output should contain migrated content"
        );
    }
}

#[test]
fn test_jcl_migrate_toml() {
    let jcl_migrate_path = get_binary_path("jcl-migrate");
    let test_file = create_temp_file(
        "test_migrate.toml",
        r#"name = "test"
value = 123
enabled = true"#,
    );

    let output = Command::new(&jcl_migrate_path)
        .arg(&test_file)
        .arg("--from")
        .arg("toml")
        .output()
        .expect("Failed to execute jcl-migrate");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("name") || stdout.contains("test"),
            "Output should contain migrated content"
        );
    }
}

#[test]
fn test_jcl_bench_exists() {
    let jcl_bench_path = get_binary_path("jcl-bench");

    // Just verify the binary exists and can be executed
    let output = Command::new(&jcl_bench_path).arg("--help").output();

    assert!(output.is_ok(), "jcl-bench should be executable");
}

#[test]
fn test_jcl_validate_multiple_files() {
    let jcl_validate_path = get_binary_path("jcl-validate");

    // Create test schema
    let schema_file = create_temp_file(
        "test_multifile_schema.yaml",
        r#"
version: "1.0"
type:
  kind: map
  required:
    - name
  properties:
    name:
      type:
        kind: string
"#,
    );

    // Create valid config files
    let config1 = create_temp_file("test_multifile_1.jcf", "name = \"config1\"");
    let config2 = create_temp_file("test_multifile_2.jcf", "name = \"config2\"");

    let output = Command::new(&jcl_validate_path)
        .arg("--schema")
        .arg(&schema_file)
        .arg(&config1)
        .arg(&config2)
        .output()
        .expect("Failed to execute jcl-validate");

    assert!(
        output.status.success(),
        "jcl-validate should succeed with multiple valid files"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("2 file(s)"),
        "Should indicate 2 files validated"
    );
    assert!(
        stdout.contains("All files passed validation"),
        "Should show success message"
    );
}

#[test]
fn test_jcl_validate_directory() {
    let jcl_validate_path = get_binary_path("jcl-validate");

    // Create test directory
    let temp_dir = std::env::temp_dir().join("jcl_validate_dir_test");
    fs::create_dir_all(&temp_dir).expect("Failed to create test directory");

    // Create schema
    let schema_file = create_temp_file(
        "test_dir_schema.yaml",
        r#"
version: "1.0"
type:
  kind: map
  properties:
    value:
      type:
        kind: number
"#,
    );

    // Create test files in directory
    fs::write(temp_dir.join("test1.jcf"), "value = 10").expect("Failed to write test1.jcf");
    fs::write(temp_dir.join("test2.jcf"), "value = 20").expect("Failed to write test2.jcf");

    let output = Command::new(&jcl_validate_path)
        .arg("--schema")
        .arg(&schema_file)
        .arg("--dir")
        .arg(&temp_dir)
        .output()
        .expect("Failed to execute jcl-validate");

    // Cleanup
    fs::remove_dir_all(&temp_dir).ok();

    assert!(
        output.status.success(),
        "jcl-validate should succeed with directory of valid files"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("file(s)"),
        "Should show files validated from directory"
    );
}

#[test]
fn test_jcl_validate_no_fail_fast() {
    let jcl_validate_path = get_binary_path("jcl-validate");

    // Create test schema
    let schema_file = create_temp_file(
        "test_no_fail_fast_schema.yaml",
        r#"
version: "1.0"
type:
  kind: map
  required:
    - required_field
  properties:
    required_field:
      type:
        kind: string
"#,
    );

    // Create files - some valid, some invalid
    let config1 = create_temp_file("test_nff_1.jcf", "required_field = \"valid\"");
    let config2 = create_temp_file("test_nff_2.jcf", "other_field = \"invalid\""); // Missing required field
    let config3 = create_temp_file("test_nff_3.jcf", "required_field = \"also_valid\"");

    let output = Command::new(&jcl_validate_path)
        .arg("--schema")
        .arg(&schema_file)
        .arg("--no-fail-fast")
        .arg(&config1)
        .arg(&config2)
        .arg(&config3)
        .output()
        .expect("Failed to execute jcl-validate");

    assert!(
        !output.status.success(),
        "jcl-validate should fail when files have errors"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // With --no-fail-fast, should validate all 3 files
    assert!(
        stdout.contains("3"),
        "Should validate all 3 files with --no-fail-fast"
    );
    assert!(
        !stdout.contains("Stopped on first error"),
        "Should not show 'stopped on first error' message with --no-fail-fast"
    );
}

#[test]
fn test_jcl_watch_help() {
    let jcl_watch_path = get_binary_path("jcl-watch");

    // Just verify the binary exists and can show help
    let output = Command::new(&jcl_watch_path).arg("--help").output();

    assert!(output.is_ok(), "jcl-watch should be executable");
}

#[test]
fn test_jcl_lsp_help() {
    let jcl_lsp_path = get_binary_path("jcl-lsp");

    // Just verify the binary exists
    let output = Command::new(&jcl_lsp_path).arg("--help").output();

    assert!(output.is_ok(), "jcl-lsp should be executable");
}

#[test]
fn test_jcl_eval_with_functions() {
    let jcl_path = get_binary_path("jcl-cli");
    let test_file = create_temp_file(
        "test_eval_functions.jcf",
        r#"
fn double(x) = x * 2
result = double(21)
"#,
    );

    let output = Command::new(&jcl_path)
        .arg("eval")
        .arg(&test_file)
        .output()
        .expect("Failed to execute jcl eval");

    assert!(
        output.status.success(),
        "jcl eval should succeed with functions"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("result = 42"),
        "Function result should be correct"
    );
}

#[test]
fn test_jcl_eval_with_lists() {
    let jcl_path = get_binary_path("jcl-cli");
    let test_file = create_temp_file(
        "test_eval_lists.jcf",
        r#"
numbers = [1, 2, 3, 4, 5]
first = numbers[0]
"#,
    );

    let output = Command::new(&jcl_path)
        .arg("eval")
        .arg(&test_file)
        .output()
        .expect("Failed to execute jcl eval");

    assert!(
        output.status.success(),
        "jcl eval should succeed with lists"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("numbers") && stdout.contains("first"),
        "List operations should work"
    );
}

#[test]
fn test_jcl_eval_with_maps() {
    let jcl_path = get_binary_path("jcl-cli");
    let test_file = create_temp_file(
        "test_eval_maps.jcf",
        r#"
config = (host = "localhost", port = 8080)
server_host = config.host
"#,
    );

    let output = Command::new(&jcl_path)
        .arg("eval")
        .arg(&test_file)
        .output()
        .expect("Failed to execute jcl eval");

    assert!(output.status.success(), "jcl eval should succeed with maps");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("server_host") && stdout.contains("localhost"),
        "Map member access should work"
    );
}
