use super::super::{NativeSfcLintOwner, NativeSfcLintRefusal as Refusal};
use super::support::*;
use crate::{
    HelpLevel, Linter, Severity,
    native::{NativeLintRefusal, template::NativeTemplateLintRefusal as Template},
};
use vize_l0::Allocator;
use vize_l1::embed::syntax::EmbedHole;
use vize_l2::file::vue::ExposureIssueKind;
use vize_l2::lang::js::{SetupIssue, SetupIssueKind};

#[test]
fn actual_parser_hole_and_retained_script_comment_cannot_supply_completed_setup() {
    for locale in LOCALES {
        let events = log();
        let configured = single(&events, locale);
        let source = SOURCE.replace("const text = \"ref(null)\"", "const text =");
        assert_eq!(
            configured.lint_native_sfc(&source, FILE).unwrap_err(),
            Refusal::ProgramSyntax {
                span: script_span(&source),
                hole: Some(EmbedHole::Syntax)
            }
        );
        let source = SOURCE.replace("const text", "/* oxlint-disable */const text");
        assert_eq!(
            configured.lint_native_sfc(&source, FILE).unwrap_err(),
            Refusal::ScriptComment {
                span: span(&source, "/* oxlint-disable */")
            }
        );
        assert_eq!(*events.lock().unwrap(), Vec::<serde_json::Value>::new());
    }
}

#[test]
fn completed_primitive_setup_does_not_grant_clean_credit_to_late_template_comment_or_binding() {
    for locale in LOCALES {
        for (old, new, error_text, comment) in [
            (
                "<div ref=\"text\" />",
                "<div ref=\"text\" /><!-- control -->",
                "<!-- control -->",
                true,
            ),
            ("ref=\"text\"", ":ref=\"text\"", ":ref=\"text\"", false),
            ("ref=\"text\"", "v-if=\"text\"", "v-if=\"text\"", false),
        ] {
            let source = SOURCE.replace(old, new);
            let events = log();
            let configured = single(&events, locale);
            let range = span(&source, error_text);
            let error = if comment {
                Template::Comment { span: range }
            } else {
                Template::UnsupportedAttribute { span: range }
            };
            assert_eq!(
                configured.lint_native_sfc(&source, FILE).unwrap_err(),
                Refusal::Template(error)
            );
            assert_eq!(
                *events.lock().unwrap(),
                [expected_event(
                    &source,
                    FILE,
                    &FIRST,
                    "actual",
                    locale,
                    HelpLevel::Full,
                    None,
                    None
                )]
            );
            assert_eq!(
                complete(&configured.lint_sfc(&source, FILE)),
                expected(
                    FILE,
                    vec![diagnostic(
                        RULE,
                        locale,
                        HelpLevel::Full,
                        "actual",
                        script_span(&source).start,
                        Severity::Warning
                    )]
                )
            );
        }
    }
}

#[test]
fn late_missing_close_discards_findings_and_preserves_entire_original_parser_vector() {
    let source = SOURCE.replace("<div ref=\"text\" />", "<div ref=\"text\"/><span>");
    for locale in LOCALES {
        let events = log();
        let configured = single(&events, locale);
        assert_eq!(
            configured.lint_native_sfc(&source, FILE).unwrap_err(),
            Refusal::Template(Template::Header(NativeLintRefusal::Hole))
        );
        assert_eq!(
            *events.lock().unwrap(),
            [expected_event(
                &source,
                FILE,
                &FIRST,
                "actual",
                locale,
                HelpLevel::Full,
                None,
                None
            )]
        );
        let opening = span(&source, "<span>");
        let parser = serde_json::json!({ "rule_name": "parser/template", "severity": "error",
            "message": "Element is missing end tag.", "start": opening.start, "end": opening.end,
            "help": null, "labels": [], "fix": null });
        // Original SFC parser output precedes its subsequently appended script output.
        assert_eq!(
            complete(&configured.lint_sfc(&source, FILE)),
            expected(
                FILE,
                vec![
                    parser,
                    diagnostic(
                        RULE,
                        locale,
                        HelpLevel::Full,
                        "actual",
                        script_span(&source).start,
                        Severity::Warning
                    )
                ]
            )
        );
    }
}

#[test]
fn duplicate_original_header_refuses_with_absolute_geometry_and_full_legacy_warning() {
    let source = SOURCE.replace("ref=\"text\"", "ref=\"text\" REF=\"text\"");
    for locale in LOCALES {
        let events = log();
        let configured = single(&events, locale);
        let repeated = span(&source, "REF");
        assert_eq!(
            configured.lint_native_sfc(&source, FILE).unwrap_err(),
            Refusal::Template(Template::Header(NativeLintRefusal::DuplicateAttribute {
                span: repeated
            }))
        );
        assert_eq!(
            *events.lock().unwrap(),
            [expected_event(
                &source,
                FILE,
                &FIRST,
                "actual",
                locale,
                HelpLevel::Full,
                None,
                None
            )]
        );
        let parser = serde_json::json!({ "rule_name": "parser/template", "severity": "warning",
            "message": "Duplicate attribute `REF`. Keeping the repeated attribute so parsing can continue.",
            "start": repeated.start, "end": repeated.end, "help": null, "labels": [], "fix": null });
        assert_eq!(
            complete(&configured.lint_sfc(&source, FILE)),
            expected(
                FILE,
                vec![
                    parser,
                    diagnostic(
                        RULE,
                        locale,
                        HelpLevel::Full,
                        "actual",
                        script_span(&source).start,
                        Severity::Warning
                    )
                ]
            )
        );
    }
}

#[test]
fn callback_error_discards_earlier_callback_findings_without_running_later_rules() {
    let events = log();
    let mut first_in_registry = audit(
        &super::support::SECOND,
        "stop",
        crate::Locale::En,
        HelpLevel::Full,
        &events,
    );
    first_in_registry.fail = true;
    let configured = configured(
        vec![
            audit(&FIRST, "later", crate::Locale::En, HelpLevel::Full, &events),
            first_in_registry,
        ],
        crate::Locale::En,
        HelpLevel::Full,
    );
    assert_eq!(
        configured.lint_native_sfc(SOURCE, FILE).unwrap_err(),
        Refusal::UnsupportedEnvelope {
            span: script_span(SOURCE)
        }
    );
    assert_eq!(
        *events.lock().unwrap(),
        [expected_event(
            SOURCE,
            FILE,
            &super::support::SECOND,
            "stop",
            crate::Locale::En,
            HelpLevel::Full,
            None,
            None
        )]
    );
}

#[test]
fn genuine_untyped_call_and_setup_function_cannot_be_primitive_absence_certificates() {
    for declaration in [
        "import { ref } from 'vue'; const text = ref(null)",
        "function setup() {}",
    ] {
        let source = SOURCE.replace("const text = \"ref(null)\"", declaration);
        let arena = Allocator::new();
        let error = NativeSfcLintOwner::parse_in(&arena, &source).err().unwrap();
        let kind = if declaration.starts_with("import") {
            SetupIssueKind::Exposure(ExposureIssueKind::ScriptCall)
        } else {
            SetupIssueKind::UnsupportedSyntax
        };
        assert_eq!(
            error,
            Refusal::Setup(SetupIssue {
                span: script_span(&source),
                kind
            })
        );
    }
}

#[test]
fn original_typed_ref_call_keeps_full_diagnostic_and_actual_original_file_refusal() {
    let source = "<script setup lang=\"ts\">\nimport { ref } from \"vue\"\nconst text = ref<HTMLInputElement | null>(null)\n</script>\n<template><div ref=\"text\" /></template>\n";
    let arena = Allocator::new();
    let error = NativeSfcLintOwner::parse_in(&arena, source).err().unwrap();
    assert_eq!(
        format!("{error:?}"),
        "FileIssues { issues: [FileIssue { unit: ScriptUnitId(0), span: Span { start: 64, end: 98 }, kind: UnsupportedSyntax }], interruptions: [] }"
    );
    for locale in LOCALES {
        let original = Linter::with_preset(crate::LintPreset::Incremental)
            .with_enabled_rules(Some(vec![RULE.into()]))
            .with_locale(locale);
        assert_eq!(
            complete(&original.lint_sfc(source, "History.vue")),
            expected(
                "History.vue",
                vec![serde_json::json!({
            "rule_name": RULE, "severity": "warning", "message": "Template ref 'text' is declared with ref(); use useTemplateRef() instead (Vue 3.5+).",
            "start": 64, "end": 98, "help": "Replace with: `const text = useTemplateRef('text')`",
            "labels": [{ "message": "declared as a template ref", "start": 64, "end": 98 }], "fix": null })]
            )
        );
        // Provider parent still refuses the actual unprovided builtin before parsing.
        assert_eq!(
            original.lint_native_sfc(source, "History.vue").unwrap_err(),
            Refusal::UnprovidedRule { rule: RULE.into() }
        );
    }
}

#[test]
fn the_two_original_string_witnesses_only_supply_real_provider_eligibility_in_parent() {
    let typed = SOURCE.replace("ref(null)", "ref<HTMLInputElement | null>(null)");
    for source in [SOURCE, typed.as_str()] {
        let arena = Allocator::new();
        let owner = NativeSfcLintOwner::parse_in(&arena, source).unwrap();
        let view = owner.setup().unwrap();
        let names: Vec<_> = view
            .semantic()
            .bindings()
            .map(|binding| {
                let d = binding.declaration().unwrap();
                (
                    d.name.as_str(),
                    d.span,
                    d.initializer,
                    d.is_direct_program(),
                )
            })
            .collect();
        assert_eq!(
            names,
            [(
                "text",
                span(source, "text"),
                vize_l2::file::InitializerKind::PrimitiveLiteral,
                true
            )]
        );
        for locale in LOCALES {
            let original = Linter::with_preset(crate::LintPreset::Incremental)
                .with_enabled_rules(Some(vec![RULE.into()]))
                .with_locale(locale);
            assert_eq!(
                complete(&original.lint_sfc(source, "History.vue")),
                empty("History.vue")
            );
            assert_eq!(
                original.lint_native_sfc(source, "History.vue").unwrap_err(),
                Refusal::UnprovidedRule { rule: RULE.into() }
            );
        }
    }
}
