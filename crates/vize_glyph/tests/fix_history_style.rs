//! Whole public CSS outputs for comment, number and stabilization fixes.

use vize_glyph::{FormatError, FormatOptions, format_style};

#[expect(clippy::unwrap_used, reason = "fixture assertions fail by panicking")]
fn check(name: &str, source: &str) {
    let options = FormatOptions::default();
    let first = format_style(source, &options).unwrap();
    insta::with_settings!({prepend_module_to_snapshot => false}, {
        insta::assert_binary_snapshot!(name, first.as_bytes().to_vec());
    });
    let second = format_style(&first, &options).unwrap();
    let third = format_style(&second, &options).unwrap();
    assert_eq!(first, second);
    assert_eq!(second, third);
}

#[test]
fn top_level_comment_outputs_are_complete() {
    check(
        "history_style_comment_before_rule.txt",
        "/* stylelint-disable-next-line selector-id-pattern */\n#legacy-id { display: grid; }\n",
    );
    check(
        "history_style_comment_after_rule.txt",
        "/* NOTE: Avoid using kebab-case for better readability. */\n.foo { color: red; }\n/* trailing note */\n",
    );
    check(
        "history_style_comment_in_string.txt",
        ".x { content: \"/* not a comment */\"; }",
    );
    check(
        "history_style_comment_in_url.txt",
        ".asset{background:url(https://example.test/a/*/icon.svg);color:red}\n/* after */",
    );
    check(
        "history_style_comment_after_import.txt",
        "@import url(https://example.test/a/*/reset.css);\n/* import note */\n.asset{color:red}",
    );
    check(
        "history_style_charset_comment.txt",
        "@charset \"UTF-8\";\n/* comment */\n.a {\n  color: red;\n}",
    );
}

#[test]
fn fractional_numbers_use_public_css_formatting() {
    check(
        "history_style_number_tokens.txt",
        r#".foo { opacity: .5; margin: -.25em .75px; transition: opacity .2s cubic-bezier(.4, 0, .2, 1); content: ".5"; background: url(https://example.test/.5/icon.svg); } /* .5 */"#,
    );
    check(
        "history_style_number_after_unicode.txt",
        ".foo { --élément: .5; }",
    );
}

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "Insta serializes the complete formatter error with Debug"
)]
fn helper_only_invalid_identifier_input_keeps_public_error() {
    // The leading-zero helper accepts this token sequence, but CSS parsing
    // rejects the period in the custom property name. Keep that distinction.
    let source = r#".foo\.5 { --data: .5; --élément.5: x; width: 0.5px; content: '.5'; }"#;
    let error = format_style(source, &FormatOptions::default()).unwrap_err();
    assert!(matches!(error, FormatError::StyleFormatError(_)));
    insta::with_settings!({prepend_module_to_snapshot => false}, {
        insta::assert_debug_snapshot!("history_style_invalid_identifier_error", error);
    });
}

#[test]
fn legacy_keyframes_have_complete_stable_output() {
    check(
        "history_style_keyframes_lowercase.txt",
        concat!(
            "@-moz-keyframes orbit { 0% { transform: rotate(0deg); } }\n",
            "@-ms-keyframes orbit { 0% { transform: rotate(0deg); } }\n",
            "@keyframes orbit { 0% { transform: rotate(0deg); } }",
        ),
    );
    check(
        "history_style_keyframes_uppercase.txt",
        concat!(
            "@-moz-keyframes orbit { 0% { transform: rotate(0deg); } }\n",
            "@-MS-keyframes orbit { 0% { transform: rotate(0deg); } }\n",
            "@keyframes orbit { 0% { transform: rotate(0deg); } }",
        ),
    );
}

#[test]
fn large_floats_have_complete_stable_output() {
    check(
        "history_style_float_max.txt",
        ".a { max-height: 3.40282e38px; }",
    );
    check(
        "history_style_infinity.txt",
        concat!(
            ".group\\/item.group\\/nested-items-open > * > ",
            ".group\\/items.translate-x-0 .group\\/button { ",
            "max-height: calc(infinity * 1px); display: flex; }",
        ),
    );
}
