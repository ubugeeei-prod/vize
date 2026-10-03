//! Original BigInt atoms preserve spelling without numeric conversion.

use vize_glyph::native_doc::{LineEnding, PrintOptions};
use vize_l1::embed::Lang;

use super::{format, preservation::assert_preserved as assert_semantics, retained};

#[path = "bigint/custody.rs"]
mod custody;
#[path = "bigint/refusals.rs"]
mod refusals;

fn options(width: usize, line_ending: LineEnding) -> PrintOptions {
    PrintOptions {
        width,
        line_ending,
        ..PrintOptions::default()
    }
}

fn assert_preserved(source: &str, decode: bool) {
    assert_semantics(source, decode);
    for lang in [Lang::Js, Lang::Ts] {
        for width in [0, 1, 7, 80, 200] {
            for ending in [LineEnding::Lf, LineEnding::CrLf] {
                let options = options(width, ending);
                let output = format(source, lang, decode, options);
                assert_eq!(format(&output, lang, decode, options), output, "{source}");
            }
        }
    }
}

#[test]
fn four_original_bigint_bases_and_separators_have_complete_unchanged_atom_outputs() {
    for atom in [
        "0n",
        "1n",
        "1_000n",
        "0b10_10n",
        "0B1_0n",
        "0o7_7n",
        "0O1_0n",
        "0xA_Fn",
        "0Xf_Fn",
        "123456789012345678901234567890n",
    ] {
        let source = std::format!(" \t{atom}\r\n ");
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 1, 7, 80, 200] {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    assert_eq!(format(&source, lang, false, options(width, ending)), atom);
                }
            }
        }
        assert_preserved(&source, false);
    }
}

#[test]
fn bigint_children_keep_original_unary_reference_and_container_shapes() {
    for (source, expected) in [
        ("-1n", "- 1n"),
        ("~0xFn", "~ 0xFn"),
        ("!0n", "! 0n"),
        ("typeof 1n", "typeof 1n"),
        ("void 1n", "void 1n"),
        ("delete 0n", "delete 0n"),
        ("((1n))", "((1n))"),
        ("1n.value", "1n . value"),
        ("(1n).value", "(1n) . value"),
        ("1n['x']", "1n [ 'x' ]"),
        ("obj[1n]", "obj [ 1n ]"),
        ("f(1n,0b10n,)", "f ( 1n, 0b10n, )"),
        ("f((0n,1n))", "f ( (0n, 1n) )"),
        ("[1n,,0o7n,]", "[ 1n, , 0o7n, ]"),
        ("{a:1n,'b':0xFn}", "{ a: 1n, 'b': 0xFn }"),
        ("1n,0b1n", "1n, 0b1n"),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 1, 7, 80, 200] {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    assert_eq!(
                        format(source, lang, false, options(width, ending)),
                        expected
                    );
                }
            }
        }
        assert_preserved(source, false);
    }
}

#[test]
fn bigint_infix_conditional_and_exponentiation_parentheses_keep_real_child_layouts() {
    for (source, flat, narrow, seven) in [
        ("1n+2n", "1n + 2n", "1n +\n  2n", "1n + 2n"),
        ("1n||2n", "1n || 2n", "1n ||\n  2n", "1n ||\n  2n"),
        (
            "a?1n:2n",
            "a ? 1n : 2n",
            "a ?\n  1n :\n  2n",
            "a ?\n  1n :\n  2n",
        ),
        (
            "(-1n)**2n",
            "(- 1n) ** 2n",
            "(- 1n) **\n  2n",
            "(- 1n) **\n  2n",
        ),
        (
            "-(1n**2n)",
            "- (1n ** 2n)",
            "- (1n **\n  2n)",
            "- (1n **\n  2n)",
        ),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for (width, expected) in [
                (0, narrow),
                (1, narrow),
                (7, seven),
                (80, flat),
                (200, flat),
            ] {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    let expected = if ending == LineEnding::CrLf {
                        expected.replace('\n', "\r\n")
                    } else {
                        expected.to_owned()
                    };
                    assert_eq!(
                        format(source, lang, false, options(width, ending)),
                        expected
                    );
                }
            }
        }
        assert_preserved(source, false);
    }
}

#[test]
fn bigint_entities_comments_and_mapped_physical_line_endings_stay_authored() {
    for (source, decode, expected) in [
        ("/*l*/0xA_Fn//t", false, "/*l*/0xA_Fn//t"),
        ("1n/*x*/+/*y*/2n", false, "1n/*x*/+/*y*/2n"),
        ("!/*x*/1n", false, "!/*x*/1n"),
        ("(/*x*/1n/*y*/)", false, "(/*x*/1n/*y*/)"),
        ("&#49;&#110;", true, "&#49;&#110;"),
        ("&#48;x&#65;_&#70;&#110;", true, "&#48;x&#65;_&#70;&#110;"),
        ("&#45;&#32;1&#110;", true, "&#45;&#32;1&#110;"),
        ("(1&#110;\n)", true, "(1&#110;\n)"),
        ("(1&#110;\r\n)", true, "(1&#110;\r\n)"),
        ("1n&#32;+&#9;2&#110;", true, "1n&#32;+&#9;2&#110;"),
        ("1&#110;&#47;*x*&#47;", true, "1&#110;&#47;*x*&#47;"),
        (
            "f&#40;1&#110;&#44;0b1n&#41;",
            true,
            "f &#40; 1&#110;&#44; 0b1n &#41;",
        ),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 1, 7, 80, 200] {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    assert_eq!(
                        format(source, lang, decode, options(width, ending)),
                        expected
                    );
                }
            }
        }
        assert_preserved(source, decode);
    }
}
