//! Real missing-provider failures must not become successful requested checks.
use super::RULE_STRICT_BOOLEAN;
use crate::{LintPreset, Linter, Severity};

const SOURCE: &str = include_str!(
    "../../../../../tests/_fixtures/differential/linter/type-aware-runtime-initialization-11147/App.vue.txt"
);

#[test]
fn explicit_missing_runtime_reports_one_complete_initialization_error() {
    let project = tempfile::tempdir().unwrap();
    let missing = project.path().join("missing-native-runtime");
    assert!(!missing.exists());
    let linter = Linter::with_preset(LintPreset::Opinionated)
        .with_type_aware_lint(true)
        .with_enabled_rules(Some(vec![RULE_STRICT_BOOLEAN.into()]))
        .with_corsa_path(Some(missing.clone()));
    let result = linter.lint_sfc(SOURCE, "RuntimeRequired.vue");
    assert_eq!(result.filename.as_str(), "RuntimeRequired.vue");
    assert_eq!((result.error_count, result.warning_count), (1, 0));
    let [diagnostic] = result.diagnostics.as_slice() else {
        panic!("expected one complete runtime failure: {result:?}");
    };
    let expected = vize_l0::cstr!(
        "Configured Corsa executable does not exist: {} (from configuration)",
        missing.display()
    );
    assert_eq!(
        (
            diagnostic.rule_name,
            diagnostic.severity,
            diagnostic.message.as_str(),
            diagnostic.start,
            diagnostic.end,
            diagnostic.help.as_deref(),
            diagnostic.labels.len(),
            diagnostic.fix.is_none(),
        ),
        (
            "type/corsa-runtime",
            Severity::Error,
            expected.as_str(),
            0,
            0,
            Some(
                "Requested type-aware lint could not initialize its native runtime. Check the reported failure, install a compatible native TypeScript runtime, or configure `typeChecker.corsaPath`, `CORSA_PATH`, or `TSGO_PATH` to its executable."
            ),
            0,
            true,
        )
    );
}
