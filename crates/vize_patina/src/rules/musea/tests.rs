use super::{MuseaLinter, MuseaRule, extract_name_attr_bytes};

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

/// The `musea/require-*` reports of the full linter on an Art file.
fn musea_target_reports(source: &str) -> Vec<(&'static str, vize_l0::String)> {
    crate::Linter::with_all()
        .lint_sfc(source, "src/Counter.art.vue")
        .diagnostics
        .into_iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic.rule_name,
                "musea/require-title" | "musea/require-component"
            )
        })
        .map(|diagnostic| (diagnostic.rule_name, diagnostic.message))
        .collect()
}

// Musea renders the one imported tag every variant uses, so a second
// import inside a variant leaves the target and its title inferred.
#[test]
fn the_shared_imported_tag_is_the_target_beside_other_imports() {
    let source = include_str!(
        "../../../../../tests/_fixtures/differential/linter/musea-inferred-target-several-imports/Counter.art.vue.txt"
    );
    assert_eq!(musea_target_reports(source), []);
    let mut linter_result = super::MuseaLintResult::default();
    super::RequireComponent.check(source, &mut linter_result);
    super::RequireTitle.check(source, &mut linter_result);
    assert_eq!(linter_result.diagnostics.len(), 0);
}

// Two imports rendered side by side are ambiguous for Musea as well, so both
// rules still ask for an explicit target.
#[test]
fn several_target_candidates_stay_reported() {
    let source = include_str!(
        "../../../../../tests/_fixtures/differential/linter/musea-inferred-target-several-imports/Composition.art.vue.txt"
    );
    assert_eq!(
        musea_target_reports(source),
        [
            (
                "musea/require-title",
                vize_l0::String::from("Missing required 'title' attribute in <art> block")
            ),
            (
                "musea/require-component",
                vize_l0::String::from("Missing 'component' attribute in <art> block")
            ),
        ]
    );
}
