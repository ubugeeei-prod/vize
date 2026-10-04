//! #7697: SFC template indentation must not duplicate CRLF terminators.
#![expect(
    clippy::disallowed_macros,
    clippy::unwrap_used,
    reason = "regression fixtures construct source strings and assert by panicking"
)]

use vize_glyph::{EndOfLine, FormatOptions, format_sfc};

fn crlf_options() -> FormatOptions {
    FormatOptions {
        end_of_line: EndOfLine::Crlf,
        ..FormatOptions::default()
    }
}

#[track_caller]
fn assert_fixed_point(source: &str, expected: &str) {
    let options = crlf_options();
    let first = format_sfc(source, &options).unwrap();
    assert_eq!(first.code.as_str(), expected, "complete first-pass output");
    assert!(!first.code.contains("\r\r\n"), "redundant CR terminator");
    let second = format_sfc(&first.code, &options).unwrap();
    let third = format_sfc(&second.code, &options).unwrap();
    assert_eq!(
        second.code.as_str(),
        expected,
        "complete second-pass output"
    );
    assert_eq!(third.code.as_str(), expected, "complete third-pass output");
    assert!(!second.changed && !third.changed);
}

#[test]
fn crlf_template_corpus_has_exact_bytes_and_fixed_point() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-crlf-template-layout/App.vue.txt"
    );
    let expected = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-crlf-template-layout/reference.expected.txt"
    );
    assert_fixed_point(source, expected);
    assert!(!format_sfc(source, &crlf_options()).unwrap().changed);
    assert_fixed_point(&source.replace("\r\n", "\n"), expected);
}

#[test]
fn raw_template_lines_keep_their_content_without_carriage_return_drift() {
    for tag in ["pre", "textarea", "listing", "code v-pre"] {
        let close = tag.split(' ').next().unwrap();
        let source = format!(
            "<template>\r\n  <{tag}>\r\nfirst\r\n  second\r\n  </{close}>\r\n</template>\r\n"
        );
        assert_fixed_point(&source, &source);
    }
}

#[test]
fn multiline_literal_attributes_keep_their_authored_lines() {
    let source = "<template>\r\n  <div title=\"first\r\nsecond\">text</div>\r\n</template>\r\n";
    assert_fixed_point(source, source);
}

#[test]
fn multiline_template_literals_keep_their_authored_lines() {
    let source = "<template>\r\n  <div>{{ `first\r\nsecond` }}</div>\r\n</template>\r\n";
    let first = format_sfc(source, &crlf_options()).unwrap();
    assert!(first.code.contains("`first\r\nsecond`"));
    assert_fixed_point(source, first.code.as_str());
}
