use super::super::NativeSfcLintRefusal as Refusal;
use super::support::*;
use crate::{
    HelpLevel, Severity,
    native::{NativeLintRefusal, template::NativeTemplateLintRefusal as Template},
};
use vize_l1::markup::NativeLintTagRefusal;

#[test]
fn exact_and_modified_pre_never_turn_whole_selected_literal_body_into_clean_credit() {
    for (body, refusal) in [
        ("<div v-pre><span>{{ raw }}</span></div>", None),
        (
            "<div v-pre.camel><textarea>{{ raw }}</textarea></div>",
            Some(NativeLintTagRefusal::AmbiguousVerbatim),
        ),
        (
            "<div v-pre:argument><span>{{ raw }}</span></div>",
            Some(NativeLintTagRefusal::AmbiguousVerbatim),
        ),
    ] {
        let source = SOURCE.replace("<div ref=\"text\" />", body);
        for locale in LOCALES {
            let events = log();
            let configured = single(&events, locale);
            let expected_refusal = refusal.map_or_else(
                || Template::UnsupportedContext {
                    span: span(&source, "<div v-pre>"),
                },
                |reason| Template::Header(NativeLintRefusal::LintTag { reason }),
            );
            assert_eq!(
                configured.lint_native_sfc(&source, FILE).unwrap_err(),
                Refusal::Template(expected_refusal)
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
fn raw_foreign_component_table_and_recovery_owners_keep_the_shared_strict_profile() {
    for (body, opening) in [
        ("<script>plain</script>", "<script>"),
        ("<style>plain</style>", "<style>"),
        ("<textarea>plain</textarea>", "<textarea>"),
        ("<title>plain</title>", "<title>"),
        ("<svg><path /></svg>", "<svg>"),
        ("<math><mi>x</mi></math>", "<math>"),
        ("<Foo />", "<Foo />"),
        ("<DIV>plain</DIV>", "<DIV>"),
        ("<p>plain</p>", "<p>"),
        ("<table><tr><td>plain</td></tr></table>", "<table>"),
    ] {
        let source = SOURCE.replace("<div ref=\"text\" />", body);
        for locale in LOCALES {
            let events = log();
            let configured = single(&events, locale);
            assert_eq!(
                configured.lint_native_sfc(&source, FILE).unwrap_err(),
                Refusal::Template(Template::UnsupportedContext {
                    span: span(&source, opening)
                })
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
fn direct_and_nested_actual_interpolation_markers_refuse_with_whole_absolute_geometry() {
    for body in [
        "<div>{{ text }}</div>",
        "<div><span>{{ text }}</span></div>",
    ] {
        let source = SOURCE.replace("<div ref=\"text\" />", body);
        for locale in LOCALES {
            let events = log();
            let configured = single(&events, locale);
            assert_eq!(
                configured.lint_native_sfc(&source, FILE).unwrap_err(),
                Refusal::Template(Template::Interpolation {
                    span: span(&source, "{{ text }}")
                })
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
fn dynamic_and_object_binding_refusals_keep_original_head_range_without_expression_parse() {
    for (body, head) in [
        ("<div :[name]='opaque'/>", ":[name]"),
        ("<div v-bind='attrs'/>", "v-bind"),
    ] {
        let source = SOURCE.replace("<div ref=\"text\" />", body);
        let events = log();
        assert_eq!(
            single(&events, crate::Locale::En)
                .lint_native_sfc(&source, FILE)
                .unwrap_err(),
            Refusal::Template(Template::Header(NativeLintRefusal::UnresolvedBinding {
                span: span(&source, head)
            }))
        );
        assert_eq!(
            *events.lock().unwrap(),
            [expected_event(
                &source,
                FILE,
                &FIRST,
                "actual",
                crate::Locale::En,
                HelpLevel::Full,
                None,
                None
            )]
        );
    }
}

#[test]
fn suppression_spellings_inside_actual_string_literal_are_opaque_and_never_rescanned_as_comments() {
    let source = SOURCE.replace("ref(null)", "/* oxlint-disable */ ref(null)");
    for locale in LOCALES {
        let events = log();
        let configured = single(&events, locale);
        let full = expected(
            FILE,
            vec![diagnostic(
                RULE,
                locale,
                HelpLevel::Full,
                "actual",
                script_span(&source).start,
                Severity::Warning,
            )],
        );
        assert_eq!(
            complete(&configured.lint_native_sfc(&source, FILE).unwrap()),
            full
        );
        assert_eq!(complete(&configured.lint_sfc(&source, FILE)), full);
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
    }
}
