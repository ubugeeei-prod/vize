//! Complete selected conditionals keep three-child order and authored framing.

use super::{format, operands, preservation::assert_preserved, selected};
use oxc_ast::ast::Expression;
use vize_glyph::native_doc::{LineEnding, PrintOptions};
use vize_l0::Allocator;

#[path = "conditional/custody.rs"]
mod custody;
#[path = "conditional/refusals.rs"]
mod refusals;

const WIDTHS: [usize; 5] = [0, 1, 7, 80, 200];
const SCRIPTS: [&str; 2] = ["", "<script setup lang=ts>let a=1</script>"];

fn options(width: usize, line_ending: LineEnding) -> PrintOptions {
    PrintOptions {
        width,
        line_ending,
        ..PrintOptions::default()
    }
}

fn newline(line_ending: LineEnding) -> &'static str {
    if line_ending == LineEnding::Lf {
        "\n"
    } else {
        "\r\n"
    }
}

#[test]
fn selected_js_ts_conditionals_pin_whole_flat_and_narrow_outputs() {
    for (content, flat, narrow) in [
        ("a?b:c", "a ? b : c", "a ?\n      b :\n      c"),
        ("a?.2:0", "a ? .2 : 0", "a ?\n      .2 :\n      0"),
        ("a ? b : c", "a ? b : c", "a ?\n      b :\n      c"),
        (
            "日本?true:null",
            "日本 ? true : null",
            "日本 ?\n      true :\n      null",
        ),
        (
            "0xCA_FE?'x\\x20y':false",
            "0xCA_FE ? 'x\\x20y' : false",
            "0xCA_FE ?\n      'x\\x20y' :\n      false",
        ),
        (
            "a?b:c?d:e",
            "a ? b : c ? d : e",
            "a ?\n      b :\n      c ?\n        d :\n        e",
        ),
        (
            "a?b?c:d:e",
            "a ? b ? c : d : e",
            "a ?\n      b ?\n        c :\n        d :\n      e",
        ),
        (
            "(a?b:c)?d:e",
            "(a ? b : c) ? d : e",
            "(a ?\n      b :\n      c) ?\n      d :\n      e",
        ),
        (
            "a+b?c:d",
            "a + b ? c : d",
            "a +\n      b ?\n      c :\n      d",
        ),
        (
            "a?b+c:d",
            "a ? b + c : d",
            "a ?\n      b +\n        c :\n      d",
        ),
        (
            "f(a)?g(b):h(c)",
            "f ( a ) ? g ( b ) : h ( c )",
            "f ( a ) ?\n      g ( b ) :\n      h ( c )",
        ),
        (
            "obj.key?obj[a]:-1",
            "obj . key ? obj [ a ] : - 1",
            "obj . key ?\n      obj [ a ] :\n      - 1",
        ),
        (
            "f(a?b:c)",
            "f ( a ? b : c )",
            "f ( a ?\n      b :\n      c )",
        ),
        (
            "obj[a?b:c]",
            "obj [ a ? b : c ]",
            "obj [ a ?\n      b :\n      c ]",
        ),
        (
            "a&#63;b&#58;c",
            "a &#63; b &#58; c",
            "a &#63;\n      b &#58;\n      c",
        ),
        (
            "a&#32;&#63;&#9;b&#32;&#58;&#32;c",
            "a&#32;&#63;&#9;b&#32;&#58;&#32;c",
            "a&#32;&#63;&#9;b&#32;&#58;&#32;c",
        ),
        (
            "a/*?*/?b/*:*/:c",
            "a/*?*/? b/*:*/: c",
            "a/*?*/?\n      b/*:*/:\n      c",
        ),
        (
            "a?/*?:*/b:/*:?*/c",
            "a ?/*?:*/b :/*:?*/c",
            "a ?/*?:*/b :/*:?*/c",
        ),
        (
            "f(/*?:*/a)?b:c",
            "f (/*?:*/a ) ? b : c",
            "f (/*?:*/a ) ?\n      b :\n      c",
        ),
        (
            "/*#__PURE__*/f(a)?b:c",
            "/*#__PURE__*/f ( a ) ? b : c",
            "/*#__PURE__*/f ( a ) ?\n      b :\n      c",
        ),
    ] {
        for script in SCRIPTS {
            let source = vize_l0::cstr!(
                "<!--前--><template><!--keep--><p>{{{{{content}}}}}</p></template>{script}"
            );
            for width in WIDTHS {
                for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                    let generated = newline(line_ending);
                    let expected = if width < 80 {
                        // Every LF here is a frozen generated break, not source text.
                        let body = narrow.replace('\n', generated);
                        vize_l0::cstr!(
                            "<!--keep--><p>{{{{{generated}    {body}{generated}  }}}}</p>"
                        )
                    } else {
                        vize_l0::cstr!("<!--keep--><p>{{{{ {flat} }}}}</p>")
                    };
                    let output = format(&source, options(width, line_ending));
                    assert_eq!(output, expected, "{source} / {width} / {line_ending:?}");
                    let replay = vize_l0::cstr!("<template>{output}</template>{script}");
                    assert_eq!(
                        format(&replay, options(width, line_ending)),
                        output,
                        "{replay}"
                    );
                }
            }
        }
        assert_preserved(content);
    }
}

#[test]
fn physical_lf_and_typed_comments_keep_original_columns_and_generated_line_endings() {
    for (content, lf, crlf) in [
        (
            "&#39;//x&#39;\n?b:c",
            "&#39;//x&#39;\n?\n      b :\n      c",
            "&#39;//x&#39;\n?\r\n      b :\r\n      c",
        ),
        (
            "a?&#39;//x&#39;\n:c",
            "a ?\n      &#39;//x&#39;\n:\n      c",
            "a ?\r\n      &#39;//x&#39;\n:\r\n      c",
        ),
        (
            "(&quot;//x&quot;\r\n)?b:c",
            "(&quot;//x&quot;\r\n) ?\n      b :\n      c",
            "(&quot;//x&quot;\r\n) ?\r\n      b :\r\n      c",
        ),
        (
            "a?(&#39;//x&#39;\n):c",
            "a ?\n      (&#39;//x&#39;\n) :\n      c",
            "a ?\r\n      (&#39;//x&#39;\n) :\r\n      c",
        ),
        (
            "a?b:(&#39;//x&#39;\n)",
            "a ?\n      b :\n      (&#39;//x&#39;\n)",
            "a ?\r\n      b :\r\n      (&#39;//x&#39;\n)",
        ),
        (
            "a ? /*q\n*/ b :c",
            "a ? /*q\n*/ b :\n      c",
            "a ? /*q\n*/ b :\r\n      c",
        ),
        (
            "a&#63;&#47;*?:\r\n*&#47;b&#58;c",
            "a &#63;&#47;*?:\r\n*&#47;b &#58;\n      c",
            "a &#63;&#47;*?:\r\n*&#47;b &#58;\r\n      c",
        ),
        (
            "a //q\n?b //c\n:c",
            "a //q\n?\n      b //c\n:\n      c",
            "a //q\n?\r\n      b //c\n:\r\n      c",
        ),
        (
            "a? //q\n b: //c\n c",
            "a ? //q\n b : //c\n c",
            "a ? //q\n b : //c\n c",
        ),
    ] {
        for script in SCRIPTS {
            let source = vize_l0::cstr!("<template><p>{{{{{content}}}}}</p></template>{script}");
            for width in WIDTHS {
                for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                    let generated = newline(line_ending);
                    let body = if line_ending == LineEnding::Lf {
                        lf
                    } else {
                        crlf
                    };
                    let output = format(&source, options(width, line_ending));
                    assert_eq!(
                        output,
                        vize_l0::cstr!("<p>{{{{{generated}    {body}{generated}  }}}}</p>"),
                        "{source} / {width} / {line_ending:?}"
                    );
                    let replay = vize_l0::cstr!("<template>{output}</template>{script}");
                    assert_eq!(
                        format(&replay, options(width, line_ending)),
                        output,
                        "{replay}"
                    );
                }
            }
        }
        assert_preserved(content);
    }
}

#[test]
fn encoded_quote_alternate_keeps_authored_trimmed_lf_tail_at_all_widths() {
    for (content, tail) in [
        ("a?b:&#39;//x&#39;\n", "\n"),
        ("a?b:&quot;//x&quot;\r\n", "\r\n"),
    ] {
        let quoted = if content.contains("&#39;") {
            "&#39;//x&#39;"
        } else {
            "&quot;//x&quot;"
        };
        for script in SCRIPTS {
            let source = vize_l0::cstr!("<template><p>{{{{{content}}}}}</p></template>{script}");
            for width in WIDTHS {
                for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                    let generated = newline(line_ending);
                    let body = if width < 80 {
                        vize_l0::cstr!("a ?{generated}      b :{generated}      {quoted}")
                    } else {
                        vize_l0::cstr!("a ? b : {quoted}")
                    };
                    let output = format(&source, options(width, line_ending));
                    assert_eq!(
                        output,
                        vize_l0::cstr!("<p>{{{{{generated}    {body}{tail}}}}}</p>")
                    );
                    let replay = vize_l0::cstr!("<template>{output}</template>{script}");
                    assert_eq!(
                        format(&replay, options(width, line_ending)),
                        output,
                        "{replay}"
                    );
                }
            }
        }
        assert_preserved(content);
    }
}

#[test]
fn actual_descriptor_requires_physical_lf_after_encoded_quote_conditional_test() {
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source = vize_l0::cstr!("<template>{{{{&#39;//x&#39;\n?b:c}}}}</template>{script}");
        let owner = selected(&arena, &source);
        let original = operands(&owner);
        let syntax = original.first().unwrap().syntax();
        assert_eq!(syntax.hole(), None);
        assert_eq!(syntax.comments().count(), 0);
        assert!(syntax.source().decode_map().is_some());
        assert!(
            matches!(syntax.expression().unwrap(), Expression::ConditionalExpression(value)
            if matches!(&value.test, Expression::StringLiteral(string) if string.value.as_str() == "//x"))
        );
        for terminator in ["", "\r"] {
            let source = vize_l0::cstr!(
                "<template>{{{{&#39;//x&#39;{terminator}?b:c}}}}</template>{script}"
            );
            let descriptor = super::Vue.observe_descriptor(
                &arena,
                &source,
                super::DescriptorOptions {
                    version: super::VueVersion::V3,
                    dialect: super::VueDialect::Vue,
                    template: super::SurfaceParseOptions::default(),
                },
            );
            assert!(descriptor.admitted().is_err(), "{source}");
            assert!(descriptor.issues().iter().any(|issue| issue.code
                == vize_l1::container::vue::DescriptorIssueCode::UnsupportedBoundary));
            assert!(core::ptr::eq(descriptor.source(), source.as_str()));
        }
    }
}
