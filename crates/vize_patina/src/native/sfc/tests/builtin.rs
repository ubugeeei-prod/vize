use super::super::*;
use super::support::*;
use crate::{HelpLevel, LintPreset, Linter, Severity};
use vize_l0::{Allocator, config::VueVersion};

fn original(locale: crate::Locale, help: HelpLevel) -> Linter {
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_locale(locale)
        .with_help_level(help)
}

#[test]
fn actual_builtin_strings_preserve_whole_zero_output_and_fresh_requeries_with_exact_options() {
    let typed = SOURCE.replace("ref(null)", "ref<HTMLInputElement | null>(null)");
    for source in [SOURCE, typed.as_str()] {
        for locale in LOCALES {
            for (vue, vapor, severity, help) in [
                (None, None, Severity::Warning, HelpLevel::Full),
                (
                    Some(VueVersion::V3),
                    Some(false),
                    Severity::Error,
                    HelpLevel::None,
                ),
                (None, Some(false), Severity::Warning, HelpLevel::Short),
            ] {
                let configured = original(locale, help)
                    .with_vue_version(vue)
                    .with_vapor_mode(vapor)
                    .with_rule_severity_overrides(vec![(RULE.into(), severity)]);
                let expected = empty("History.vue");
                assert_eq!(
                    complete(&configured.lint_sfc(source, "History.vue")),
                    expected
                );
                assert_eq!(
                    complete(&configured.lint_native_sfc(source, "History.vue").unwrap()),
                    expected
                );
                assert_eq!(
                    complete(&configured.lint_native_sfc(source, "History.vue").unwrap()),
                    expected
                );
                assert_eq!(
                    complete(&configured.lint_sfc(source, "History.vue")),
                    expected
                );
            }
        }
    }
}

#[test]
fn provided_builtin_refuses_same_buffer_foreign_setup_before_empty_output() {
    let arena = Allocator::new();
    let first = NativeSfcLintOwner::parse_in(&arena, SOURCE).unwrap();
    let foreign = NativeSfcLintOwner::parse_in(&arena, SOURCE).unwrap();
    assert!(core::ptr::eq(first.source(), foreign.source()));
    assert!(!core::ptr::eq(&first, &foreign));
    let configured = original(crate::Locale::En, HelpLevel::Full);
    let instance = crate::linter::script_rules::native::configured(&configured, RULE).unwrap();
    let callback = instance.as_native_sfc_setup_rule().unwrap();
    let mut context = NativeSfcLintContext::new(&configured, &first, "History.vue");
    context.current_rule = RULE;
    assert_eq!(
        callback.run_on_setup(&mut context, &foreign.setup().unwrap()),
        Err(NativeSfcLintRefusal::SourceMismatch)
    );
    assert_eq!(complete(&context.finish()), empty("History.vue"));
    let mut context = NativeSfcLintContext::new(&configured, &first, "History.vue");
    context.current_rule = RULE;
    assert_eq!(
        callback.run_on_setup(&mut context, &first.setup().unwrap()),
        Ok(())
    );
    assert_eq!(complete(&context.finish()), empty("History.vue"));
}

#[test]
fn actual_call_initializers_keep_full_original_warnings_and_exact_whole_setup_refusal() {
    use vize_l2::{
        file::vue::ExposureIssueKind,
        lang::js::{SetupIssue, SetupIssueKind},
    };
    for callee in ["ref", "shallowRef"] {
        let declaration = format!("import {{ {callee} }} from 'vue'; const text = {callee}(null)");
        let source = SOURCE.replace("const text = \"ref(null)\"", &declaration);
        let call = span(&source, &format!("{callee}(null)"));
        for locale in LOCALES {
            let configured = original(locale, HelpLevel::Full);
            assert_eq!(
                configured
                    .lint_native_sfc(&source, "History.vue")
                    .unwrap_err(),
                NativeSfcLintRefusal::Setup(SetupIssue {
                    span: script_span(&source),
                    kind: SetupIssueKind::Exposure(ExposureIssueKind::ScriptCall),
                })
            );
            assert_eq!(
                complete(&configured.lint_sfc(&source, "History.vue")),
                expected(
                    "History.vue",
                    vec![serde_json::json!({
                        "rule_name": RULE, "severity": "warning",
                        "message": format!("Template ref 'text' is declared with {callee}(); use useTemplateRef() instead (Vue 3.5+)."),
                        "start": call.start, "end": call.end,
                        "help": "Replace with: `const text = useTemplateRef('text')`",
                        "labels": [{ "message": "declared as a template ref", "start": call.start, "end": call.end }],
                        "fix": null,
                    })]
                )
            );
        }
    }
}

#[test]
fn primitive_builtin_does_not_skip_late_template_comment_binding_or_parser_vector() {
    use crate::native::{NativeLintRefusal, template::NativeTemplateLintRefusal as Template};
    for locale in LOCALES {
        let configured = original(locale, HelpLevel::Full);
        for (old, new, error) in [
            (
                "<div ref=\"text\" />",
                "<div ref=\"text\" /><!-- control -->",
                "<!-- control -->",
            ),
            ("ref=\"text\"", ":ref=\"text\"", ":ref=\"text\""),
        ] {
            let source = SOURCE.replace(old, new);
            let range = span(&source, error);
            let refusal = if error.starts_with("<!--") {
                Template::Comment { span: range }
            } else {
                Template::UnsupportedAttribute { span: range }
            };
            assert_eq!(
                configured
                    .lint_native_sfc(&source, "History.vue")
                    .unwrap_err(),
                NativeSfcLintRefusal::Template(refusal)
            );
            assert_eq!(
                complete(&configured.lint_sfc(&source, "History.vue")),
                empty("History.vue")
            );
        }
        let source = SOURCE.replace("<div ref=\"text\" />", "<div ref=\"text\"/><span>");
        assert_eq!(
            configured
                .lint_native_sfc(&source, "History.vue")
                .unwrap_err(),
            NativeSfcLintRefusal::Template(Template::Header(NativeLintRefusal::Hole))
        );
        let opening = span(&source, "<span>");
        assert_eq!(
            complete(&configured.lint_sfc(&source, "History.vue")),
            expected(
                "History.vue",
                vec![serde_json::json!({
                    "rule_name": "parser/template", "severity": "error",
                    "message": "Element is missing end tag.", "start": opening.start, "end": opening.end,
                    "help": null, "labels": [], "fix": null,
                })]
            )
        );
    }
}

#[test]
fn builtin_script_comment_remains_typed_refusal_beside_complete_original_empty_output() {
    let source = SOURCE.replace("const text", "/* oxlint-disable */const text");
    for locale in LOCALES {
        let configured = original(locale, HelpLevel::Full);
        assert_eq!(
            configured
                .lint_native_sfc(&source, "History.vue")
                .unwrap_err(),
            NativeSfcLintRefusal::ScriptComment {
                span: span(&source, "/* oxlint-disable */")
            }
        );
        assert_eq!(
            complete(&configured.lint_sfc(&source, "History.vue")),
            empty("History.vue")
        );
    }
}

#[test]
fn actual_setup_method_candidate_keeps_original_warning_and_full_unsupported_method_refusal() {
    let declaration =
        "import { ref } from 'vue'; const options = { setup() { const text = ref(null) } }";
    let source = SOURCE.replace("const text = \"ref(null)\"", declaration);
    let method = span(&source, "setup() { const text = ref(null) }");
    let call = span(&source, "ref(null)");
    for locale in LOCALES {
        let configured = original(locale, HelpLevel::Full);
        let error = configured
            .lint_native_sfc(&source, "History.vue")
            .unwrap_err();
        assert_eq!(
            format!("{error:?}"),
            format!(
                "FileIssues {{ issues: [FileIssue {{ unit: ScriptUnitId(0), span: Span {{ start: {}, end: {} }}, kind: UnsupportedSyntax }}], interruptions: [] }}",
                method.start, method.end
            )
        );
        assert_eq!(
            complete(&configured.lint_sfc(&source, "History.vue")),
            expected(
                "History.vue",
                vec![serde_json::json!({
                    "rule_name": RULE, "severity": "warning",
                    "message": "Template ref 'text' is declared with ref(); use useTemplateRef() instead (Vue 3.5+).",
                    "start": call.start, "end": call.end,
                    "help": "Replace with: `const text = useTemplateRef('text')`",
                    "labels": [{ "message": "declared as a template ref", "start": call.start, "end": call.end }],
                    "fix": null,
                })]
            )
        );
    }
}
