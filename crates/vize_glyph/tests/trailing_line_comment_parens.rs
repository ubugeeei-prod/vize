//! #7929: unary comment grouping supplies one pair of parentheses.
use vize_glyph::{EndOfLine, FormatOptions, format_script, format_sfc};

const A: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/script-trailing-line-comment-parens/a.ts.txt"
);
const B: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/script-trailing-line-comment-parens/b.ts.txt"
);
const A_CRLF: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/script-trailing-line-comment-parens/a.ts.crlf.txt"
);
const B_CRLF: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/script-trailing-line-comment-parens/b.ts.crlf.txt"
);
const SFC: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/script-trailing-line-comment-parens/App.vue.txt"
);

fn assert_script_fixed_point(source: &str, expected: &str, options: &FormatOptions) {
    let mut current = format_script(source, options).unwrap();
    assert_eq!(current.as_str(), expected);
    for _ in 0..2 {
        current = format_script(&current, options).unwrap();
        assert_eq!(current.as_str(), expected);
    }
}

#[test]
fn both_original_conditions_keep_every_byte_through_three_public_passes() {
    assert_eq!(A.len(), 64);
    assert_eq!(B.len(), 97);
    for source in [A, B] {
        assert_script_fixed_point(source, source, &FormatOptions::default());
    }
    for (source, expected) in [(A_CRLF, A), (B_CRLF, B)] {
        assert_script_fixed_point(source, expected, &FormatOptions::default());
        for end_of_line in [EndOfLine::Crlf, EndOfLine::Auto] {
            let options = FormatOptions {
                end_of_line,
                ..FormatOptions::default()
            };
            assert_script_fixed_point(source, source, &options);
        }
    }
}

#[test]
fn script_block_keeps_original_comment_grouping_and_changed_flag() {
    let options = FormatOptions::default();
    let mut current = format_sfc(SFC, &options).unwrap();
    assert_eq!(current.code.as_str(), SFC);
    assert!(!current.changed);
    for _ in 0..2 {
        current = format_sfc(&current.code, &options).unwrap();
        assert_eq!(current.code.as_str(), SFC);
        assert!(!current.changed);
    }
}

#[test]
fn comment_tokens_precedence_and_required_inner_groups_remain_owned() {
    for source in [
        "const ok = !(\n  left && right\n  // trailing\n);\n",
        "const ok = !(\n  left || right // trailing\n);\n",
        "const ok = !(/* leading */ left && right);\n",
        "const ok = !(left && right /* trailing */);\n",
        "const ok = !(\n  (left || right) && other\n  // trailing\n);\n",
        "const ok = !(\n  left === \"// literal )\"\n  // trailing\n);\n",
        "const ok = !(\n  left === /[()]/\n  // trailing\n);\n",
        "const ok = !(left && right);\n",
        "const ok = !((left, right) /* trailing */);\n",
        "const text = \"!(value) // not a comment\";\n",
    ] {
        assert_script_fixed_point(source, source, &FormatOptions::default());
    }
}
