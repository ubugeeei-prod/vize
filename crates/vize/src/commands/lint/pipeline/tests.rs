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
        true,
    )
    .into_parts()
    .1;

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
        true,
    )
    .into_parts()
    .1;

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
        true,
    )
    .into_parts()
    .1;

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
        true,
    )
    .into_parts()
    .1;

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
        true,
    )
    .into_parts()
    .1;

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
        true,
    )
    .into_parts()
    .1;

    assert!(files[0].fixed);
    assert!(files[0].artifact_backed);
    assert!(files[0].semantics.is_some());
    assert_eq!(files[0].result.warning_count, 0);
    assert_eq!(
        fs::read_to_string(path).unwrap(),
        "const View = () => <input  />;"
    );
}

#[test]
fn raw_script_and_html_pipeline_are_atlas_backed_with_direct_parity() {
    let project = tempfile::tempdir().unwrap();
    let script = project.path().join("state.ts");
    let html = project.path().join("index.html");
    let script_source = "import { ref } from '@vue/reactivity'; export const count = ref(0);";
    let html_source = r#"<button v-on:click="save">Save</button>"#;
    fs::write(&script, script_source).unwrap();
    fs::write(&html, html_source).unwrap();
    let expected_linter = Linter::with_preset(vize_patina::LintPreset::Opinionated)
        .with_additional_rules(vec!["script/prefer-import-from-vue".into()]);
    let expected_script =
        expected_linter.lint_script(script_source, script.to_string_lossy().as_ref());
    let expected_html =
        expected_linter.lint_standalone_html(html_source, html.to_string_lossy().as_ref());

    let files = lint_inputs(
        read_lint_inputs(&[script, html], false),
        Shared::new(
            Linter::with_preset(vize_patina::LintPreset::Opinionated)
                .with_additional_rules(vec!["script/prefer-import-from-vue".into()]),
        ),
        VueVersion::V3,
        false,
        false,
        true,
    )
    .into_parts()
    .1;

    assert!(files.iter().all(|file| file.artifact_backed));
    assert_eq!(
        vize_carton::cstr!("{:?}", files[0].result.diagnostics),
        vize_carton::cstr!("{:?}", expected_script.diagnostics)
    );
    assert_eq!(
        vize_carton::cstr!("{:?}", files[1].result.diagnostics),
        vize_carton::cstr!("{:?}", expected_html.diagnostics)
    );
}

#[test]
fn raw_script_and_html_autofix_revalidate_through_atlas() {
    let project = tempfile::tempdir().unwrap();
    let script = project.path().join("state.ts");
    let html = project.path().join("index.html");
    fs::write(&script, "import { ref } from '@vue/reactivity';").unwrap();
    fs::write(&html, r#"<button v-on:click="save">Save</button>"#).unwrap();

    let files = lint_inputs(
        read_lint_inputs(&[script.clone(), html.clone()], false),
        Shared::new(
            Linter::with_preset(vize_patina::LintPreset::Opinionated)
                .with_additional_rules(vec!["script/prefer-import-from-vue".into()]),
        ),
        VueVersion::V3,
        true,
        false,
        true,
    )
    .into_parts()
    .1;

    assert!(files.iter().all(|file| file.fixed && file.artifact_backed));
    assert_eq!(
        fs::read_to_string(script).unwrap(),
        "import { ref } from 'vue';"
    );
    assert_eq!(
        fs::read_to_string(html).unwrap(),
        r#"<button @click="save">Save</button>"#
    );
    assert!(files.iter().all(|file| {
        file.result.diagnostics.iter().all(|diagnostic| {
            diagnostic.rule_name != "script/prefer-import-from-vue"
                && diagnostic.rule_name != "vue/v-on-style"
        })
    }));
}

#[test]
fn storybook_exclusion_is_an_empty_atlas_report() {
    let project = tempfile::tempdir().unwrap();
    let story = project.path().join("Button.stories.tsx");
    fs::write(
        &story,
        "export const Story = () => <button autofocus>{items.map(item => <i>{item}</i>)}</button>;",
    )
    .unwrap();

    let files = lint_inputs(
        read_lint_inputs(std::slice::from_ref(&story), false),
        Shared::new(Linter::with_preset(vize_patina::LintPreset::HappyPath)),
        VueVersion::V3,
        false,
        false,
        true,
    )
    .into_parts()
    .1;

    assert!(files[0].artifact_backed);
    assert!(files[0].semantics.is_none());
    assert!(!files[0].result.has_diagnostics());
}
