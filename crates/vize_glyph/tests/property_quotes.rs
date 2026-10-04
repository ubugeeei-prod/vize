//! #7753: shared script options honor the existing property-quote policy.
#![expect(
    clippy::disallowed_macros,
    reason = "regressions construct complete independent expected output"
)]
use vize_glyph::{
    Allocator, FormatOptions, QuoteProps, VueVersion, format_script,
    format_script_with_source_type, format_sfc_with_vue_version,
};

#[test]
fn configured_property_quotes_preserve_complete_corpus_output_and_fixed_points() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-preserve-property-quotes/App.vue.txt"
    );
    let expected = include_bytes!(
        "../../../tests/_fixtures/differential/formatter/sfc-preserve-property-quotes/reference.expected.txt"
    );
    let options = FormatOptions {
        quote_props: QuoteProps::Preserve,
        ..FormatOptions::default()
    };
    for version in [VueVersion::V2, VueVersion::V2_7, VueVersion::V3] {
        let first = format_sfc_with_vue_version(source, &options, version).unwrap();
        assert_eq!(first.code.as_bytes(), expected);
        assert!(first.changed);
        let second = format_sfc_with_vue_version(&first.code, &options, version).unwrap();
        let third = format_sfc_with_vue_version(&second.code, &options, version).unwrap();
        assert_eq!(second.code.as_bytes(), expected);
        assert_eq!(third.code.as_bytes(), expected);
        assert!(!second.changed && !third.changed);
    }
    assert_eq!(options.quote_props, QuoteProps::Preserve);
}

#[test]
fn every_property_quote_policy_has_independent_full_output_controls() {
    let source = "const value={\"name\":1,\"needs-dash\":2}\nconst plain={\"name\":1,plain:2}\n";
    let as_needed =
        "const value = { name: 1, \"needs-dash\": 2 };\nconst plain = { name: 1, plain: 2 };\n";
    let consistent =
        "const value = { \"name\": 1, \"needs-dash\": 2 };\nconst plain = { name: 1, plain: 2 };\n";
    let preserve = "const value = { \"name\": 1, \"needs-dash\": 2 };\nconst plain = { \"name\": 1, plain: 2 };\n";
    assert_eq!(
        format_script(source, &FormatOptions::default())
            .unwrap()
            .as_bytes(),
        as_needed.as_bytes()
    );
    let allocator = Allocator::default();
    for (quote_props, expected) in [
        (QuoteProps::AsNeeded, as_needed),
        (QuoteProps::Consistent, consistent),
        (QuoteProps::Preserve, preserve),
    ] {
        let options = FormatOptions {
            quote_props,
            ..FormatOptions::default()
        };
        let first = format_script(source, &options).unwrap();
        assert_eq!(first.as_bytes(), expected.as_bytes());
        let second = format_script(&first, &options).unwrap();
        let third = format_script(&second, &options).unwrap();
        assert_eq!(second.as_bytes(), expected.as_bytes());
        assert_eq!(third.as_bytes(), expected.as_bytes());
        for source_type in [
            oxc_span::SourceType::ts(),
            oxc_span::SourceType::tsx(),
            oxc_span::SourceType::jsx(),
        ] {
            assert_eq!(
                format_script_with_source_type(source, &options, &allocator, source_type)
                    .unwrap()
                    .as_bytes(),
                expected.as_bytes()
            );
        }
        let single_quotes = FormatOptions {
            single_quote: true,
            ..options.clone()
        };
        let expected_single = expected.replace('"', "'");
        assert_eq!(
            format_script(source, &single_quotes).unwrap().as_bytes(),
            expected_single.as_bytes()
        );
        assert_eq!(options.quote_props, quote_props);
    }
}
