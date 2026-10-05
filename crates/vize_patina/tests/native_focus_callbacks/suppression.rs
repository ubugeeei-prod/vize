use super::support::*;
use vize_l0::Span;
use vize_patina::{HelpLevel, Severity, native::template::NativeTemplateLintRefusal as Refusal};

#[test]
fn actual_next_line_and_multiline_opening_geometry_remain_whole_original_controls_only() {
    let comment = "<!-- eslint-disable-next-line a11y/no-autofocus -->";
    for (source, suppressed, authored) in [
        (format!("{comment}\n<div autofocus/>"), true, "autofocus"),
        (format!("{comment}\n\n<div autofocus/>"), false, "autofocus"),
        (format!("{comment}\n<div\n autofocus/>"), false, "autofocus"),
    ] {
        for locale in LOCALES {
            let configured = linter(false, locale, HelpLevel::Full);
            let start = source.rfind(authored).unwrap() as u32;
            let findings = if suppressed {
                vec![]
            } else {
                vec![finding(
                    AUTO,
                    locale,
                    HelpLevel::Full,
                    Span::new(start, start + 9),
                    Severity::Warning,
                )]
            };
            original(&configured, &source, expected(findings));
            assert_eq!(
                configured.lint_native_template(&source, FILE).unwrap_err(),
                Refusal::Comment {
                    span: Span::new(0, comment.len() as u32)
                }
            );
        }
    }
}
