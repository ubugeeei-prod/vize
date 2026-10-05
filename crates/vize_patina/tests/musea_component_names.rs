//! Original Art filename regressions and unchanged complete template diagnostics.
#![expect(clippy::disallowed_macros, reason = "compare complete fixture results")]

use vize_patina::{LintPreset, LintResult, Linter, RuleRegistry, Severity};

const BUTTON: &str = include_str!("fixtures/musea-component-names/my-button.art.vue.txt");
const BADGE: &str = include_str!("fixtures/musea-component-names/Badge.art.vue.txt");
const CASING: &str = "vue/component-definition-name-casing";
const MULTI_WORD: &str = "vue/multi-word-component-names";

fn names() -> Linter {
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![CASING.into(), MULTI_WORD.into()]))
}

fn assert_empty(result: &LintResult, filename: &str) {
    let expected = LintResult {
        filename: filename.into(),
        diagnostics: vec![],
        error_count: 0,
        warning_count: 0,
    };
    assert_eq!(format!("{result:#?}"), format!("{expected:#?}"));
}

#[test]
fn both_unmodified_reporter_sources_are_clean_in_default_and_explicit_name_rules() {
    for (source, filename) in [(BUTTON, "my-button.art.vue"), (BADGE, "Badge.art.vue")] {
        assert_empty(&Linter::new().lint_sfc(source, filename), filename);
        assert_empty(&names().lint_sfc(source, filename), filename);
    }
}

#[test]
fn art_file_identity_is_retained_with_root_posix_and_windows_paths() {
    for filename in [
        "Bad_File.art.vue",
        "/元/components/Badge.art.vue",
        r"C:\元\components\Badge.art.vue",
    ] {
        assert_empty(&names().lint_sfc(BADGE, filename), filename);
        assert_empty(&names().lint_template("<div />", filename), filename);
    }
}

#[test]
fn ordinary_component_names_and_case_sensitive_suffix_keep_their_complete_findings() {
    for (filename, rules, counts) in [
        ("myButton.vue", vec![CASING], (0, 1)),
        ("Badge.vue", vec![MULTI_WORD], (1, 0)),
        ("Bad_file.vue", vec![CASING, MULTI_WORD], (1, 1)),
        ("Badge.Art.vue", vec![CASING], (0, 1)),
        ("src/.art.vue/myButton.vue", vec![CASING], (0, 1)),
    ] {
        let result = names().lint_template("<div />", filename);
        assert_eq!(result.filename.as_str(), filename);
        assert_eq!((result.error_count, result.warning_count), counts);
        assert_eq!(result.diagnostics.len(), rules.len());
        for (diagnostic, rule) in result.diagnostics.iter().zip(rules) {
            assert_eq!(diagnostic.rule_name, rule);
            assert_eq!(
                diagnostic.severity,
                if rule == CASING {
                    Severity::Warning
                } else {
                    Severity::Error
                }
            );
            assert!(!diagnostic.message.is_empty());
            assert!(diagnostic.help.is_some());
            assert!(diagnostic.labels.is_empty());
            assert!(diagnostic.fix.is_none());
            assert!(diagnostic.start <= diagnostic.end && diagnostic.end <= 7);
        }
    }
}

#[test]
fn every_variant_keeps_the_entire_unrelated_diagnostic_and_original_attribute_span() {
    let source = "<!--界-->\r\n<art title=\"Focus\"><variant name=\"One\"><input autofocus /></variant>\r\n<variant name=\"Two\"><input autofocus /></variant></art>";
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(vize_patina::rules::a11y::NoAutofocus));
    let baseline = Linter::with_registry(registry).lint_sfc(source, "Badge.art.vue");
    let configured = names()
        .with_enabled_rules(Some(vec![
            CASING.into(),
            MULTI_WORD.into(),
            "a11y/no-autofocus".into(),
        ]))
        .lint_sfc(source, "Badge.art.vue");
    assert_eq!(format!("{configured:#?}"), format!("{baseline:#?}"));
    assert_eq!((configured.error_count, configured.warning_count), (0, 2));
    assert_eq!(configured.diagnostics.len(), 2);
    for (diagnostic, (start, attribute)) in configured
        .diagnostics
        .iter()
        .zip(source.match_indices("autofocus"))
    {
        assert_eq!(diagnostic.rule_name, "a11y/no-autofocus");
        assert_eq!(
            (diagnostic.start, diagnostic.end),
            (start as u32, (start + attribute.len()) as u32)
        );
    }
}

#[test]
fn explicitly_enabled_musea_metadata_findings_are_not_suppressed() {
    let source = "<art component=\"./Badge.vue\"><variant name=\"Empty\"></variant></art>";
    let baseline = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![
            "musea/require-title".into(),
            "musea/no-empty-variant".into(),
        ]))
        .lint_sfc(source, "Badge.art.vue");
    let configured = names()
        .with_enabled_rules(Some(vec![
            CASING.into(),
            MULTI_WORD.into(),
            "musea/require-title".into(),
            "musea/no-empty-variant".into(),
        ]))
        .lint_sfc(source, "Badge.art.vue");
    assert_eq!(format!("{configured:#?}"), format!("{baseline:#?}"));
    assert_eq!((configured.error_count, configured.warning_count), (1, 1));
    assert_eq!(configured.diagnostics.len(), 2);
}
