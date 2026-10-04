use super::*;

#[test]
fn missing_original_close_keeps_complete_parser_error_and_refuses_independently() {
    let source = "<div>";
    for locale in LOCALES {
        let linter = configured(locale, HelpLevel::Full);
        assert_eq!(
            linter.lint_native_template(source, FILE).unwrap_err(),
            Refusal::Header(NativeLintRefusal::Hole)
        );
        assert_eq!(
            complete(&linter.lint_template(source, FILE)),
            expected(
                FILE,
                vec![
                    root_warning(locale),
                    parser("Element is missing end tag.", 0, 5)
                ]
            )
        );
    }
}

#[test]
fn late_missing_close_discards_pending_root_result_and_retains_complete_original_order() {
    let source = "<div>Content</div><span>";
    let missing = span(source, "<span>");
    for locale in LOCALES {
        let linter = configured(locale, HelpLevel::Full);
        assert_eq!(
            linter.lint_native_template(source, FILE).unwrap_err(),
            Refusal::Header(NativeLintRefusal::Hole)
        );
        assert_eq!(
            complete(&linter.lint_template(source, FILE)),
            expected(
                FILE,
                vec![
                    root_warning(locale),
                    parser("Element is missing end tag.", missing.start, missing.end)
                ]
            )
        );
    }
}

#[test]
fn duplicate_static_headers_preserve_whole_parser_warning_and_root_metadata() {
    let source = "<div id='one' ID='two'>Content</div>";
    let repeated = span(source, "ID");
    for locale in LOCALES {
        let linter = configured(locale, HelpLevel::Full);
        assert_eq!(
            linter.lint_native_template(source, FILE).unwrap_err(),
            Refusal::Header(NativeLintRefusal::DuplicateAttribute { span: repeated })
        );
        let mut duplicate = parser(
            "Duplicate attribute `ID`. Keeping the repeated attribute so parsing can continue.",
            repeated.start,
            repeated.end,
        );
        duplicate["severity"] = "warning".into();
        assert_eq!(
            complete(&linter.lint_template(source, FILE)),
            expected(FILE, vec![root_warning(locale), duplicate])
        );
    }
}

#[test]
fn late_stray_close_keeps_complete_original_parser_output_and_cannot_be_clean() {
    let source = "<div>Content</div></stray>";
    let stray = span(source, "</stray>");
    for locale in LOCALES {
        let linter = configured(locale, HelpLevel::Full);
        assert_eq!(
            linter.lint_native_template(source, FILE).unwrap_err(),
            Refusal::UnexpectedChild { span: stray }
        );
        assert_eq!(
            complete(&linter.lint_template(source, FILE)),
            expected(
                FILE,
                vec![
                    root_warning(locale),
                    parser("Invalid end tag.", stray.start, stray.end)
                ]
            )
        );
    }
}

#[test]
fn original_nonempty_lexer_errors_refuse_before_root_callback_with_full_legacy_vector() {
    let source = "<div";
    let file = "my-component.vue";
    for locale in LOCALES {
        let linter = configured(locale, HelpLevel::Full);
        assert_eq!(
            linter.lint_native_template(source, file).unwrap_err(),
            Refusal::Recovered { offset: 4 }
        );
        assert_eq!(
            complete(&linter.lint_template(source, file)),
            expected(
                file,
                vec![
                    parser(
                        "Unexpected end of input inside a tag; inferred the missing tag close so parsing can continue.",
                        3,
                        4
                    ),
                    parser("Element is missing end tag.", 0, 4)
                ]
            )
        );
    }
}
