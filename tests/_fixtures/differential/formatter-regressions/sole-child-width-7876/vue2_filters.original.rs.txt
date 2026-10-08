//! Issue #6845: Vue 2 filters must not become Vue 3 bitwise-OR expressions.

use vize_glyph::{
    Allocator, EndOfLine, FormatOptions, VueVersion, format_sfc,
    format_sfc_with_allocator_and_vue_version, format_sfc_with_vue_version, format_template,
    format_template_with_vue_version,
};

#[test]
fn configured_sfc_filter_corpus_preserves_every_byte_and_reaches_a_fixed_point() {
    let cases = [
        (
            include_str!("../../../tests/_fixtures/differential/formatter/sfc-vue2-filter-chain-crlf/App.vue.txt"),
            include_bytes!("../../../tests/_fixtures/differential/formatter/sfc-vue2-filter-chain-crlf/reference.expected.txt").as_slice(),
            VueVersion::V2,
            EndOfLine::Crlf,
        ),
        (
            include_str!("../../../tests/_fixtures/differential/formatter/sfc-vue2-7-filter-chain/App.vue.txt"),
            include_bytes!("../../../tests/_fixtures/differential/formatter/sfc-vue2-7-filter-chain/reference.expected.txt").as_slice(),
            VueVersion::V2_7,
            EndOfLine::Lf,
        ),
        (
            include_str!("../../../tests/_fixtures/differential/formatter/sfc-vue3-bitwise-or/App.vue.txt"),
            include_bytes!("../../../tests/_fixtures/differential/formatter/sfc-vue3-bitwise-or/reference.expected.txt").as_slice(),
            VueVersion::V3,
            EndOfLine::Lf,
        ),
    ];
    let allocator = Allocator::default();
    for (source, expected, version, end_of_line) in cases {
        let options = FormatOptions {
            end_of_line,
            ..FormatOptions::default()
        };
        let first = format_sfc_with_vue_version(source, &options, version).unwrap();
        assert_eq!(first.code.as_bytes(), expected, "{version:?} first pass");
        assert_eq!(first.changed, source.as_bytes() != expected);
        let second =
            format_sfc_with_allocator_and_vue_version(&first.code, &options, &allocator, version)
                .unwrap();
        let third = format_sfc_with_vue_version(&second.code, &options, version).unwrap();
        assert_eq!(second.code.as_bytes(), expected, "{version:?} second pass");
        assert_eq!(third.code.as_bytes(), expected, "{version:?} third pass");
        assert!(!second.changed && !third.changed);
        if version == VueVersion::V3 {
            assert_eq!(
                format_sfc(source, &options).unwrap().code.as_bytes(),
                expected
            );
        }
    }
}

#[test]
fn explicit_filter_version_preserves_hyphenated_names_and_default_is_vue3() {
    let source = "<p>{{ message | format-date('en') }}</p>";
    let options = FormatOptions::default();
    for version in [VueVersion::V2, VueVersion::V2_7] {
        let first = format_template_with_vue_version(source, &options, version).unwrap();
        assert_eq!(first.as_str(), "<p>{{ message | format-date(\"en\") }}</p>");
        assert_eq!(
            format_template_with_vue_version(&first, &options, version).unwrap(),
            first
        );
    }
    let expected_vue3 = "<p>{{ message | (format - date(\"en\")) }}</p>";
    assert_eq!(
        format_template(source, &options).unwrap().as_str(),
        expected_vue3
    );
    assert_eq!(
        format_template_with_vue_version(source, &options, VueVersion::V3)
            .unwrap()
            .as_str(),
        expected_vue3
    );
}

#[test]
fn pipes_in_javascript_payloads_and_events_keep_their_javascript_meaning() {
    let options = FormatOptions::default();
    for (source, expected) in [
        ("<p>{{ '日|本' | f }}</p>", "<p>{{ \"日|本\" | f }}</p>"),
        (
            "<p>{{ /a|b/.test(value) | f }}</p>",
            "<p>{{ /a|b/.test(value) | f }}</p>",
        ),
        ("<p>{{ a||b | f }}</p>", "<p>{{ a || b | f }}</p>"),
        ("<p>{{ (a|b) | f }}</p>", "<p>{{ (a | b) | f }}</p>"),
        ("<p>{{ `日|本` | f }}</p>", "<p>{{ `日|本` | f }}</p>"),
        (
            "<p @click=\"a | format-date('en')\" />",
            "<p @click=\"a | (format - date('en'))\" />",
        ),
        ("<p>{{ a | f( }}</p>", "<p>{{ a | f( }}</p>"),
    ] {
        let first = format_template_with_vue_version(source, &options, VueVersion::V2).unwrap();
        assert_eq!(first.as_str(), expected, "{source}");
        assert_eq!(
            format_template_with_vue_version(&first, &options, VueVersion::V2).unwrap(),
            first
        );
    }
}
