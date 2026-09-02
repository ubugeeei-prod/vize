use super::{MuseaLinter, extract_name_attr_bytes};

#[test]
fn test_lint_valid_art_file() {
    let source = r#"
<art title="Button" component="./Button.vue">
  <variant name="default">
    <Button>Click me</Button>
  </variant>
</art>
"#;
    let linter = MuseaLinter::new();
    let result = linter.lint(source);
    assert!(!result.has_errors());
}

#[test]
fn test_lint_missing_title() {
    let source = r#"
<art component="./Button.vue">
  <variant name="default">
    <Button>Click me</Button>
  </variant>
</art>
"#;
    let linter = MuseaLinter::new();
    let result = linter.lint(source);
    assert!(result.has_errors());
}

#[test]
fn test_lint_duplicate_variant_names() {
    let source = r#"
<art title="Button" component="./Button.vue">
  <variant name="same">
    <Button>One</Button>
  </variant>
  <variant name="same">
    <Button>Two</Button>
  </variant>
</art>
"#;
    let linter = MuseaLinter::new();
    let result = linter.lint(source);
    assert!(result.has_errors());
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_lint_empty_variant() {
    let source = r#"
<art title="Button" component="./Button.vue">
  <variant name="empty"></variant>
</art>
"#;
    let linter = MuseaLinter::new();
    let result = linter.lint(source);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_extract_name_attr() {
    assert_eq!(
        extract_name_attr_bytes(b"<variant name=\"test\""),
        Some(b"test".as_slice())
    );
    assert_eq!(
        extract_name_attr_bytes(b"<variant name='test'"),
        Some(b"test".as_slice())
    );
    assert_eq!(extract_name_attr_bytes(b"<variant "), None);
}
