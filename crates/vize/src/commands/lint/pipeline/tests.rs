use super::*;

static JSX_FIX_META: vize_patina::RuleMeta = vize_patina::RuleMeta {
    name: "test/remove-autofocus",
    description: "test-only JSX fix",
    category: vize_patina::RuleCategory::Accessibility,
    fixable: true,
    default_severity: vize_patina::Severity::Warning,
};

struct RemoveAutofocus;

impl vize_patina::MarkupRule for RemoveAutofocus {
    fn name(&self) -> &'static str {
        JSX_FIX_META.name
    }

    fn enter_binding<'a>(
        &self,
        context: &mut vize_patina::MarkupContext<'_, 'a>,
        _element: &vize_patina::MarkupElement<'a>,
        binding: &vize_patina::MarkupBinding<'a>,
    ) {
        if !binding.arg_name_eq("autofocus") {
            return;
        }
        let range = binding.range();
        context.lint().report(
            vize_patina::LintDiagnostic::warn(
                JSX_FIX_META.name,
                "remove autofocus",
                range.start,
                range.end,
            )
            .with_fix(vize_patina::Fix::new(
                "Remove autofocus",
                vize_patina::TextEdit::delete(range.start, range.end),
            )),
        );
    }
}

impl vize_patina::Rule for RemoveAutofocus {
    fn meta(&self) -> &'static vize_patina::RuleMeta {
        &JSX_FIX_META
    }

    fn as_markup_rule(&self) -> Option<&dyn vize_patina::MarkupRule> {
        Some(self)
    }
}

#[test]
fn vue_pipeline_uses_artifact_graph_and_preserves_direct_result() {
    let project = tempfile::tempdir().unwrap();
    let path = project.path().join("Component.vue");
    let source = "<template><div v-if=\"ok\">{{ missing }}</div></template>";
    fs::write(&path, source).unwrap();
    let expected = Linter::new().lint_sfc(source, path.to_string_lossy().as_ref());

    let files = lint_inputs(
        read_lint_inputs(std::slice::from_ref(&path), false),
        Shared::new(Linter::new()),
        VueVersion::V3,
        false,
        false,
    );

    assert!(files[0].artifact_backed);
    assert!(files[0].semantics.is_some());
    assert_eq!(files[0].result.error_count, expected.error_count);
    assert_eq!(files[0].result.warning_count, expected.warning_count);
    let actual: Vec<_> = files[0]
        .result
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.rule_name, diagnostic.start, diagnostic.end))
        .collect();
    let expected: Vec<_> = expected
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.rule_name, diagnostic.start, diagnostic.end))
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn autofix_revalidates_fixed_vue_through_artifact_graph() {
    let project = tempfile::tempdir().unwrap();
    let path = project.path().join("Fix.vue");
    fs::write(
        &path,
        r#"<template><button v-on:click="save">Save</button></template>"#,
    )
    .unwrap();

    let files = lint_inputs(
        read_lint_inputs(std::slice::from_ref(&path), false),
        Shared::new(Linter::new()),
        VueVersion::V3,
        true,
        false,
    );

    assert!(files[0].fixed);
    assert!(files[0].artifact_backed);
    assert!(files[0].semantics.is_some());
    assert_eq!(
        fs::read_to_string(path).unwrap(),
        r#"<template><button @click="save">Save</button></template>"#
    );
}

#[test]
fn malformed_vue_stays_on_atlas_and_matches_legacy_diagnostics() {
    let project = tempfile::tempdir().unwrap();
    let path = project.path().join("Malformed.vue");
    let source = "<template><div /></template><template><span /></template>";
    fs::write(&path, source).unwrap();
    let expected = Linter::new().lint_sfc(source, path.to_string_lossy().as_ref());

    let files = lint_inputs(
        read_lint_inputs(std::slice::from_ref(&path), false),
        Shared::new(Linter::new()),
        VueVersion::V3,
        false,
        false,
    );

    assert!(files[0].artifact_backed);
    assert!(files[0].semantics.is_none());
    assert_eq!(
        vize_carton::cstr!("{:?}", files[0].result.diagnostics),
        vize_carton::cstr!("{:?}", expected.diagnostics)
    );
}

#[test]
fn malformed_vue_preserves_incremental_preset_lazy_parse_behavior() {
    let project = tempfile::tempdir().unwrap();
    let path = project.path().join("Incremental.vue");
    let source = "<template><div /></template><template><span /></template>";
    fs::write(&path, source).unwrap();
    let expected = Linter::with_preset(vize_patina::LintPreset::Incremental)
        .lint_sfc(source, path.to_string_lossy().as_ref());

    let files = lint_inputs(
        read_lint_inputs(std::slice::from_ref(&path), false),
        Shared::new(Linter::with_preset(vize_patina::LintPreset::Incremental)),
        VueVersion::V3,
        false,
        false,
    );

    assert!(files[0].artifact_backed);
    assert_eq!(
        vize_carton::cstr!("{:?}", files[0].result.diagnostics),
        vize_carton::cstr!("{:?}", expected.diagnostics)
    );
}

#[test]
fn jsx_and_tsx_pipeline_are_atlas_backed() {
    let project = tempfile::tempdir().unwrap();
    let jsx = project.path().join("View.jsx");
    let tsx = project.path().join("Typed.tsx");
    fs::write(&jsx, "const View = () => <img />;").unwrap();
    fs::write(
        &tsx,
        "const Typed = (p: Props): JSX.Element => <img src={p.src} />;",
    )
    .unwrap();

    let files = lint_inputs(
        read_lint_inputs(&[jsx, tsx], false),
        Shared::new(Linter::new()),
        VueVersion::V3,
        false,
        false,
    );

    assert_eq!(files.len(), 2);
    assert!(files.iter().all(|file| file.artifact_backed));
    assert!(files.iter().all(|file| file.semantics.is_some()));
}

#[test]
fn jsx_autofix_revalidates_through_atlas() {
    let project = tempfile::tempdir().unwrap();
    let path = project.path().join("Fix.jsx");
    fs::write(&path, "const View = () => <input autofocus />;").unwrap();
    let mut rules = vize_patina::RuleRegistry::new();
    rules.register(Box::new(RemoveAutofocus));

    let files = lint_inputs(
        read_lint_inputs(std::slice::from_ref(&path), false),
        Shared::new(Linter::with_registry(rules)),
        VueVersion::V3,
        true,
        false,
    );

    assert!(files[0].fixed);
    assert!(files[0].artifact_backed);
    assert!(files[0].semantics.is_some());
    assert_eq!(files[0].result.warning_count, 0);
    assert_eq!(
        fs::read_to_string(path).unwrap(),
        "const View = () => <input  />;"
    );
}
