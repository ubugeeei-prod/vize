use super::support::*;
use vize_l0::Span;
use vize_l1::markup::NativeLintTagRefusal;
use vize_patina::{
    HelpLevel, Severity,
    native::{NativeLintRefusal, template::NativeTemplateLintRefusal as Refusal},
};

#[test]
fn late_comments_true_suppression_and_interpolation_never_publish_pending_attribute_findings() {
    for (source, authored, refusal, suppressed) in [
        (
            "<div autofocus/><!-- ordinary -->",
            "<!-- ordinary -->",
            "comment",
            false,
        ),
        (
            "<div autofocus/><!-- eslint-disable -->",
            "<!-- eslint-disable -->",
            "comment",
            true,
        ),
        (
            "<div autofocus/><span>{{ value }}</span>",
            "{{ value }}",
            "interpolation",
            false,
        ),
        (
            "<div autofocus/><span>{{}}</span>",
            "{{}}",
            "interpolation",
            false,
        ),
    ] {
        for locale in LOCALES {
            let configured = linter(false, locale, HelpLevel::Full);
            let findings = if suppressed {
                vec![]
            } else {
                vec![finding(
                    AUTO,
                    locale,
                    HelpLevel::Full,
                    Span::new(5, 14),
                    Severity::Warning,
                )]
            };
            original(&configured, source, expected(findings));
            let expected = if refusal == "comment" {
                Refusal::Comment {
                    span: span(source, authored),
                }
            } else {
                Refusal::Interpolation {
                    span: span(source, authored),
                }
            };
            assert_eq!(
                configured.lint_native_template(source, FILE).unwrap_err(),
                expected
            );
        }
    }
}

#[test]
fn late_event_structural_and_object_binding_preserve_whole_original_warning_but_refuse() {
    for (late, head, unresolved) in [
        ("<span @click='handler'/>", "@click='handler'", false),
        ("<span v-show='opaque'/>", "v-show='opaque'", false),
        ("<span v-bind='attrs'/>", "v-bind", true),
    ] {
        let source = format!("<div autofocus/>{late}");
        for locale in LOCALES {
            let configured = linter(false, locale, HelpLevel::Full);
            original(
                &configured,
                &source,
                expected(vec![finding(
                    AUTO,
                    locale,
                    HelpLevel::Full,
                    Span::new(5, 14),
                    Severity::Warning,
                )]),
            );
            let expected = if unresolved {
                Refusal::Header(NativeLintRefusal::UnresolvedBinding {
                    span: span(&source, head),
                })
            } else {
                Refusal::UnsupportedAttribute {
                    span: span(&source, head),
                }
            };
            assert_eq!(
                configured.lint_native_template(&source, FILE).unwrap_err(),
                expected
            );
        }
    }
}

#[test]
fn empty_dynamic_and_clean_lexer_overwrite_heads_keep_precise_refusal_not_absence_credit() {
    for head in [
        ":[]",
        ".[]",
        "v-bind:[]",
        ":[name]tail",
        ".[name]tail",
        "v-bind:[name]tail",
        ":pre[name]",
        ":[name][other]",
    ] {
        let source = format!("<div autofocus/><span {head}='opaque'/>");
        for locale in LOCALES {
            let configured = linter(false, locale, HelpLevel::Full);
            original(
                &configured,
                &source,
                expected(vec![finding(
                    AUTO,
                    locale,
                    HelpLevel::Full,
                    Span::new(5, 14),
                    Severity::Warning,
                )]),
            );
            assert_eq!(
                configured.lint_native_template(&source, FILE).unwrap_err(),
                Refusal::Header(NativeLintRefusal::UnresolvedBinding {
                    span: span(&source, head)
                })
            );
        }
    }
}

#[test]
fn actual_component_exemption_is_retained_as_whole_original_empty_and_native_context_refusal() {
    for source in [
        "<MyInput autofocus accesskey='h'/>",
        "<Input :autofocus='opaque' :accesskey='opaque'/>",
    ] {
        for locale in LOCALES {
            let configured = linter(false, locale, HelpLevel::Full);
            original(&configured, source, expected(vec![]));
            assert_eq!(
                configured.lint_native_template(source, FILE).unwrap_err(),
                Refusal::UnsupportedContext {
                    span: Span::new(0, source.len() as u32)
                }
            );
        }
    }
}

#[test]
fn foreign_and_table_raw_or_recovery_owners_refuse_without_discarding_original_full_findings() {
    for (source, opening) in [
        (
            "<svg autofocus accesskey='h'><path/></svg>",
            "<svg autofocus accesskey='h'>",
        ),
        (
            "<math autofocus accesskey='h'><mi>x</mi></math>",
            "<math autofocus accesskey='h'>",
        ),
        (
            "<table autofocus accesskey='h'><tr><td>x</td></tr></table>",
            "<table autofocus accesskey='h'>",
        ),
        (
            "<textarea autofocus accesskey='h'>text</textarea>",
            "<textarea autofocus accesskey='h'>",
        ),
        (
            "<p autofocus accesskey='h'>text</p>",
            "<p autofocus accesskey='h'>",
        ),
    ] {
        for locale in LOCALES {
            let configured = linter(false, locale, HelpLevel::Full);
            original(
                &configured,
                source,
                expected(vec![
                    finding(
                        AUTO,
                        locale,
                        HelpLevel::Full,
                        span(source, "autofocus"),
                        Severity::Warning,
                    ),
                    finding(
                        KEY,
                        locale,
                        HelpLevel::Full,
                        span(source, "accesskey='h'"),
                        Severity::Warning,
                    ),
                ]),
            );
            assert_eq!(
                configured.lint_native_template(source, FILE).unwrap_err(),
                Refusal::UnsupportedContext {
                    span: span(source, opening)
                }
            );
        }
    }
}

#[test]
fn exact_and_modified_verbatim_headers_cannot_grant_whole_body_clean_credit() {
    for (source, refusal) in [
        (
            "<div autofocus v-pre></div>",
            Refusal::UnsupportedContext {
                span: Span::new(0, 21),
            },
        ),
        (
            "<div v-pre.camel autofocus></div>",
            Refusal::Header(NativeLintRefusal::LintTag {
                reason: NativeLintTagRefusal::AmbiguousVerbatim,
            }),
        ),
    ] {
        for locale in LOCALES {
            let configured = linter(false, locale, HelpLevel::Full);
            original(
                &configured,
                source,
                expected(vec![finding(
                    AUTO,
                    locale,
                    HelpLevel::Full,
                    span(source, "autofocus"),
                    Severity::Warning,
                )]),
            );
            assert_eq!(
                configured.lint_native_template(source, FILE).unwrap_err(),
                refusal
            );
        }
    }
}
