use super::support::*;
use vize_l0::Span;
use vize_patina::{
    HelpLevel, Severity,
    native::{NativeLintRefusal, template::NativeTemplateLintRefusal as Refusal},
};

#[test]
fn late_missing_close_keeps_full_sorted_original_parser_vector_and_atomic_native_refusal() {
    let source = "<div autofocus/><span accesskey='h'>";
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
                    Span::new(5, 14),
                    Severity::Warning,
                ),
                parser(
                    "Element is missing end tag.",
                    span(source, "<span accesskey='h'>"),
                    Severity::Error,
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
            Refusal::Header(NativeLintRefusal::Hole)
        );
    }
}

#[test]
fn static_case_duplicate_header_preserves_complete_original_parser_warning_and_exact_refusal() {
    let source = "<div autofocus AUTOFOCUS/>";
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
                    Span::new(5, 14),
                    Severity::Warning,
                ),
                parser(
                    "Duplicate attribute `AUTOFOCUS`. Keeping the repeated attribute so parsing can continue.",
                    Span::new(15, 24),
                    Severity::Warning,
                ),
            ]),
        );
        assert_eq!(
            configured.lint_native_template(source, FILE).unwrap_err(),
            Refusal::Header(NativeLintRefusal::DuplicateAttribute {
                span: Span::new(15, 24)
            })
        );
    }
}

#[test]
fn dynamic_lexer_recovery_precludes_even_root_callbacks_with_full_original_parser_metadata() {
    let source = "<div autofocus/><span :[name='opaque'/>";
    let offset = source.find('=').unwrap() as u32;
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
                    Span::new(5, 14),
                    Severity::Warning,
                ),
                parser(
                    "Dynamic directive argument is missing its closing `]`; inferred the argument end at the next tag boundary.",
                    Span::new(offset, offset + 1),
                    Severity::Error,
                ),
            ]),
        );
        assert_eq!(
            configured.lint_native_template(source, FILE).unwrap_err(),
            Refusal::Recovered { offset }
        );
    }
}
