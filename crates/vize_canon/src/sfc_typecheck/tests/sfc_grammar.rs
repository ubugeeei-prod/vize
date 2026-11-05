use super::{SfcTypeCheckOptions, SfcTypeSeverity, type_check_sfc};

#[test]
fn adjacent_quoted_attributes_keep_template_analysis_and_virtual_ts() {
    let options = SfcTypeCheckOptions::new("SfcGrammar.vue").with_virtual_ts();
    let original = include_str!(
        "../../../../../tests/_fixtures/differential/typechecker/sfc-entry-grammar-11869/Adjacent.vue.txt"
    );
    for source in [
        original.to_owned(),
        original.replace(r#"id="a"class="b""#, "id='a'class='b'"),
        original.replace(r#"id="a"class="b""#, "id=\"a\"class='b'"),
    ] {
        let result = type_check_sfc(&source, &options);
        let start = source.find("missing").unwrap() as u32;
        let actual = result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                (
                    diagnostic.severity,
                    diagnostic.code.as_deref(),
                    diagnostic.message.as_str(),
                    diagnostic.start,
                    diagnostic.end,
                    diagnostic.help.as_deref(),
                    diagnostic.related.len(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            actual,
            vec![(
                SfcTypeSeverity::Error,
                Some("undefined-binding"),
                "Undefined reference 'missing' in template expression",
                start,
                start + 7,
                Some("Make sure 'missing' is defined in script setup or imported"),
                0,
            )],
            "{source}"
        );
        assert_eq!((result.error_count, result.warning_count), (1, 0));
        assert!(result.virtual_ts.is_some());
    }
}

#[test]
fn adjacent_quoted_attributes_do_not_suppress_malformed_template_diagnostics() {
    let source = include_str!(
        "../../../../../tests/_fixtures/differential/typechecker/sfc-entry-grammar-11869/Malformed.vue.txt"
    );
    let result = type_check_sfc(
        source,
        &SfcTypeCheckOptions::new("SfcGrammar.vue").with_virtual_ts(),
    );

    assert_eq!((result.error_count, result.warning_count), (1, 0));
    assert!(result.virtual_ts.is_none());
    assert_eq!(result.diagnostics.len(), 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.code.as_deref(), Some("template-parse-error"));
    assert_eq!(diagnostic.severity, SfcTypeSeverity::Error);
    assert_eq!(
        diagnostic.message.as_str(),
        "Template parse error: Element is missing end tag."
    );
    assert_eq!((diagnostic.start, diagnostic.end), (10, 31));
    assert!(diagnostic.help.is_none());
    assert!(diagnostic.related.is_empty());
}

#[test]
fn sfc_selection_keeps_the_complete_base_eof_diagnostics() {
    use vize_atelier_core::{
        Allocator, ErrorCode, ParseMode, ParserOptions, TemplateSyntaxMode,
        parser::parse_with_options_and_template_syntax,
    };

    let allocator = Allocator::default();
    let source = r#"<div class="a""#;
    for syntax in [TemplateSyntaxMode::Standard, TemplateSyntaxMode::Strict] {
        let (base, expected) = parse_with_options_and_template_syntax(
            &allocator,
            source,
            ParserOptions::default(),
            syntax,
        );
        assert_eq!(
            expected.iter().map(|error| error.code).collect::<Vec<_>>(),
            vec![ErrorCode::EofInTag, ErrorCode::MissingEndTag]
        );
        let (sfc, actual) = parse_with_options_and_template_syntax(
            &allocator,
            source,
            ParserOptions {
                mode: ParseMode::Sfc,
                ..Default::default()
            },
            syntax,
        );
        assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
        assert_eq!(format!("{sfc:?}"), format!("{base:?}"));
    }
}
