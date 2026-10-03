//! Original ternary branches, checked punctuation and complete document layout.

use vize_glyph::native_doc::{LineEnding, PrintOptions};
use vize_l1::embed::Lang;

use super::{format, preservation::assert_preserved as assert_semantics, retained};

#[path = "conditional/custody.rs"]
mod custody;
#[path = "conditional/refusals.rs"]
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
            for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                let options = options(width, line_ending);
                let output = format(source, lang, decode, options);
                assert_eq!(format(&output, lang, decode, options), output, "{source}");
            }
        }
    }
}

#[test]
fn ternary_whole_outputs_keep_branch_order_parentheses_and_supported_children() {
    for (source, flat, narrow) in [
        ("a?b:c", "a ? b : c", "a ?\n  b :\n  c"),
        ("a?.2:0", "a ? .2 : 0", "a ?\n  .2 :\n  0"),
        ("!a?+b:-c", "! a ? + b : - c", "! a ?\n  + b :\n  - c"),
        (
            "a&&b?f(x):obj[key]",
            "a && b ? f ( x ) : obj [ key ]",
            "a &&\n  b ?\n  f ( x ) :\n  obj [ key ]",
        ),
        (
            "a?b?c:d:e",
            "a ? b ? c : d : e",
            "a ?\n  b ?\n    c :\n    d :\n  e",
        ),
        (
            "a?b:c?d:e",
            "a ? b : c ? d : e",
            "a ?\n  b :\n  c ?\n    d :\n    e",
        ),
        (
            "(a?b:c)?d:e",
            "(a ? b : c) ? d : e",
            "(a ?\n  b :\n  c) ?\n  d :\n  e",
        ),
        (
            "a?(b+c):d**e",
            "a ? (b + c) : d ** e",
            "a ?\n  (b +\n    c) :\n  d **\n    e",
        ),
        ("f(a?b:c)", "f ( a ? b : c )", "f ( a ?\n  b :\n  c )"),
        (
            "(a?b:c).key",
            "(a ? b : c) . key",
            "(a ?\n  b :\n  c) . key",
        ),
        ("obj[a?b:c]", "obj [ a ? b : c ]", "obj [ a ?\n  b :\n  c ]"),
        (
            "a?eval('x'):(eval)('y')",
            "a ? eval ( 'x' ) : (eval) ( 'y' )",
            "a ?\n  eval ( 'x' ) :\n  (eval) ( 'y' )",
        ),
        (
            "日本?\\u0061:0xCA_FE",
            "日本 ? \\u0061 : 0xCA_FE",
            "日本 ?\n  \\u0061 :\n  0xCA_FE",
        ),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                assert_eq!(format(source, lang, false, options(200, line_ending)), flat);
                let narrow = if line_ending == LineEnding::Lf {
                    narrow.to_owned()
                } else {
                    narrow.replace('\n', "\r\n")
                };
                assert_eq!(
                    format(source, lang, false, options(0, line_ending)),
                    narrow,
                    "{source}"
                );
            }
        }
        assert_preserved(source, false);
    }
}

#[test]
fn original_question_colon_entities_and_typed_comments_stay_complete_at_every_width() {
    for (source, decode, expected) in [
        (
            "a/*t*/?/*q*/b/*c*/:/*a*/c",
            false,
            "a/*t*/?/*q*/b/*c*/:/*a*/c",
        ),
        ("a&#32;?&#9;b&#32;:&#9;c", true, "a&#32;?&#9;b&#32;:&#9;c"),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 1, 7, 80, 200] {
                for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                    assert_eq!(
                        format(source, lang, decode, options(width, line_ending)),
                        expected,
                        "{source}"
                    );
                }
            }
        }
        assert_preserved(source, decode);
    }
    for lang in [Lang::Js, Lang::Ts] {
        for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
            let generated = if line_ending == LineEnding::Lf {
                "\n"
            } else {
                "\r\n"
            };
            assert_eq!(
                format("a&#63;b&#58;c", lang, true, options(200, line_ending)),
                "a &#63; b &#58; c"
            );
            assert_eq!(
                format("a&#63;b&#58;c", lang, true, options(0, line_ending)),
                std::format!("a &#63;{generated}  b &#58;{generated}  c")
            );
            for (source, flat, narrow) in [
                (
                    "f(/*t*/)/*q*/?b/*c*/:c",
                    "f (/*t*/)/*q*/? b/*c*/: c",
                    std::format!("f (/*t*/)/*q*/?{generated}  b/*c*/:{generated}  c"),
                ),
                (
                    "a?f(/*b*/)/*c*/:g(/*d*/)",
                    "a ? f (/*b*/)/*c*/: g (/*d*/)",
                    std::format!("a ?{generated}  f (/*b*/)/*c*/:{generated}  g (/*d*/)"),
                ),
            ] {
                assert_eq!(format(source, lang, false, options(200, line_ending)), flat);
                assert_eq!(format(source, lang, false, options(0, line_ending)), narrow);
            }
        }
    }
    assert_preserved("a&#63;b&#58;c", true);
    assert_preserved("f(/*t*/)/*q*/?b/*c*/:c", false);
    assert_preserved("a?f(/*b*/)/*c*/:g(/*d*/)", false);
}

#[test]
fn original_physical_newlines_remain_separate_from_generated_ternary_layout() {
    for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
        let generated = if line_ending == LineEnding::Lf {
            "\n"
        } else {
            "\r\n"
        };
        for (source, decode, expected) in [
            (
                "a //t\r\n ?b:c",
                false,
                std::format!("a //t\r\n ?{generated}  b :{generated}  c"),
            ),
            (
                "a?b //c\n :c",
                false,
                std::format!("a ?{generated}  b //c\n :{generated}  c"),
            ),
            (
                "a?&#39;//x&#39;\n:c",
                true,
                std::format!("a ?{generated}  &#39;//x&#39;\n:{generated}  c"),
            ),
            (
                "a?\r\n&#39;x&#39;:c",
                true,
                std::format!("a ?\r\n&#39;x&#39; :{generated}  c"),
            ),
        ] {
            for lang in [Lang::Js, Lang::Ts] {
                for width in [0, 1, 7, 80, 200] {
                    assert_eq!(
                        format(source, lang, decode, options(width, line_ending)),
                        expected,
                        "{source}"
                    );
                }
            }
            assert_preserved(source, decode);
        }
    }
}
