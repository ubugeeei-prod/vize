//! Original #7979 source and complete physical tag-name diagnostics.

use vize_patina::{LintPreset, LintResult, Linter, Severity};

const RULE: &str = "vue/require-component-registration";
const ORIGINAL: &str = include_str!("fixtures/issue-7979/my-panel.vue.txt");
const CRLF: &str = include_str!("fixtures/issue-7979/crlf-panel.vue.txt");
const UNICODE: &str = include_str!("fixtures/issue-7979/UnicodePanel.vue.txt");
const REGISTERED: &str = include_str!("fixtures/issue-7979/RegisteredPanel.vue.txt");

fn linter() -> Linter {
    Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![RULE.into()]))
}

fn check(result: &LintResult, source: &str, filename: &str, tags: &[&str]) {
    assert_eq!(result.filename.as_str(), filename);
    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, tags.len());
    assert_eq!(result.diagnostics.len(), tags.len());
    let mut cursor = 0;
    for (diagnostic, tag) in result.diagnostics.iter().zip(tags) {
        let rest = source.get(cursor..).unwrap();
        let opening = rest
            .match_indices('<')
            .find(|(start, _)| rest.get(start + 1..).unwrap().starts_with(tag))
            .unwrap()
            .0;
        let start = cursor + opening + 1;
        let end = start + tag.len();
        assert_eq!(
            (diagnostic.start, diagnostic.end),
            (start as u32, end as u32)
        );
        assert_eq!(source.get(start..end), Some(*tag));
        assert_eq!(diagnostic.rule_name, RULE);
        assert_eq!(diagnostic.severity, Severity::Warning);
        assert_eq!(
            diagnostic.message.as_str(),
            "Component is used but not explicitly imported"
        );
        assert_eq!(
            diagnostic.help.as_deref(),
            Some("Import the component in <script setup> or register it in components option")
        );
        assert!(diagnostic.labels.is_empty());
        assert!(diagnostic.fix.is_none());
        cursor = end;
    }
}

#[test]
fn original_whole_sfc_reports_only_the_complete_tag_name() {
    let result = linter().lint_sfc(ORIGINAL, "my-panel.vue");
    check(&result, ORIGINAL, "my-panel.vue", &["MyButton"]);
    let diagnostic = result.diagnostics.first().unwrap();
    assert_eq!((diagnostic.start, diagnostic.end), (24, 32));
}

#[test]
fn crlf_preserves_every_original_byte_and_shifts_only_the_physical_start() {
    assert_eq!(CRLF, ORIGINAL.replace('\n', "\r\n"));
    check(
        &linter().lint_sfc(CRLF, "my-panel.vue"),
        CRLF,
        "my-panel.vue",
        &["MyButton"],
    );
}

#[test]
fn unicode_prefix_self_closing_and_nested_kebab_tags_keep_byte_ranges() {
    check(
        &linter().lint_sfc(UNICODE, "UnicodePanel.vue"),
        UNICODE,
        "UnicodePanel.vue",
        &["MissingPanel", "my-button"],
    );
}

#[test]
fn imported_self_builtin_configured_framework_and_native_names_remain_registered() {
    check(
        &linter()
            .with_component_registration_globals(vec!["RouterLink".into()])
            .lint_sfc(REGISTERED, "RegisteredPanel.vue"),
        REGISTERED,
        "RegisteredPanel.vue",
        &[],
    );
}

#[test]
fn standalone_template_without_sfc_offsets_uses_the_same_tag_name_span() {
    let source = "<div>界😀<MissingPanel/><MissingPanel>text</MissingPanel></div>";
    check(
        &linter().lint_template(source, "Standalone.vue"),
        source,
        "Standalone.vue",
        &["MissingPanel", "MissingPanel"],
    );
}
