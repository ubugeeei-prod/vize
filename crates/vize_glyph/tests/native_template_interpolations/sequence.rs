//! Whole selected sequences keep original commas, parentheses and child order.

use super::{format, preservation::assert_preserved};
use oxc_ast::ast::Expression;
use vize_glyph::native_doc::{LineEnding, PrintOptions};
use vize_l0::Allocator;

#[path = "sequence/custody.rs"]
mod custody;
#[path = "sequence/refusals.rs"]
mod refusals;

const WIDTHS: [usize; 5] = [0, 1, 7, 80, 200];
const SCRIPTS: [&str; 2] = ["", "<script setup lang=ts>let a=1</script>"];

fn newline(ending: LineEnding) -> &'static str {
    if ending == LineEnding::Lf {
        "\n"
    } else {
        "\r\n"
    }
}

fn options(width: usize, line_ending: LineEnding) -> PrintOptions {
    PrintOptions {
        width,
        line_ending,
        ..PrintOptions::default()
    }
}

#[test]
fn selected_sequences_pin_complete_js_ts_outputs_without_new_sequence_breaks() {
    for (content, body) in [
        ("a,b", "a, b"),
        ("a , b , c", "a, b, c"),
        ("(a,b)", "(a, b)"),
        ("((a,b))", "((a, b))"),
        ("a,(b,c)", "a, (b, c)"),
        ("(a,b),c", "(a, b), c"),
        ("f(a),g(b),h(c)", "f ( a ), g ( b ), h ( c )"),
        ("(0,obj.method)()", "(0, obj . method) ( )"),
        ("((0,obj.method))()", "((0, obj . method)) ( )"),
        ("(obj.method)()", "(obj . method) ( )"),
        ("(0,eval)('x')", "(0, eval) ( 'x' )"),
        ("(eval)('x')", "(eval) ( 'x' )"),
        ("eval('x')", "eval ( 'x' )"),
        ("delete (0,obj.x)", "delete (0, obj . x)"),
        ("delete (obj.x)", "delete (obj . x)"),
        ("typeof (a,b)", "typeof (a, b)"),
        ("f((a,b),c)", "f ( (a, b), c )"),
        ("[(a,b)]", "[ (a, b) ]"),
        ("obj[a,b]", "obj [ a, b ]"),
        ("[a,,],b", "[ a, , ], b"),
        ("'a,b',null", "'a,b', null"),
        ("日本,\\u0061", "日本, \\u0061"),
        ("a&#44;b&#44;c", "a&#44; b&#44; c"),
        ("&#40;a&#44;b&#41;", "&#40;a&#44; b&#41;"),
        ("a&#32;&#44;&#9;b", "a&#32;&#44;&#9;b"),
        ("(a/*,*/,/*,*/b)", "(a/*,*/,/*,*/b)"),
        ("a/*,*/,/*,*/b,/*tail*/c", "a/*,*/,/*,*/b,/*tail*/c"),
        ("/*#__PURE__*/f(a),g(b)", "/*#__PURE__*/f ( a ), g ( b )"),
        ("f(),[a,,],obj.key", "f ( ), [ a, , ], obj . key"),
    ] {
        for script in SCRIPTS {
            let source = vize_l0::cstr!(
                "<!--前--><template><!--keep--><p>{{{{{content}}}}}</p></template>{script}"
            );
            for width in WIDTHS {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    let generated = newline(ending);
                    let expected = if width < 80 {
                        vize_l0::cstr!(
                            "<!--keep--><p>{{{{{generated}    {body}{generated}  }}}}</p>"
                        )
                    } else {
                        vize_l0::cstr!("<!--keep--><p>{{{{ {body} }}}}</p>")
                    };
                    let output = format(&source, options(width, ending));
                    assert_eq!(output, expected, "{source} / {width} / {ending:?}");
                    let replay = vize_l0::cstr!("<template>{output}</template>{script}");
                    assert_eq!(format(&replay, options(width, ending)), output, "{replay}");
                }
            }
        }
        assert_preserved(content);
    }
}

#[test]
fn sequences_add_no_breaks_while_original_infix_and_conditional_child_groups_keep_their_layout() {
    for (content, flat, narrow) in [
        ("a+b,c", "a + b, c", "a +\n      b, c"),
        ("a,b+c", "a, b + c", "a, b +\n      c"),
        ("a?b:c,d", "a ? b : c, d", "a ?\n      b :\n      c, d"),
        (
            "a,(b?c:d)",
            "a, (b ? c : d)",
            "a, (b ?\n      c :\n      d)",
        ),
        ("f((a+b,c))", "f ( (a + b, c) )", "f ( (a +\n      b, c) )"),
        ("obj[a,b+c]", "obj [ a, b + c ]", "obj [ a, b +\n      c ]"),
        (
            "a?(b,c):(d,e)",
            "a ? (b, c) : (d, e)",
            "a ?\n      (b, c) :\n      (d, e)",
        ),
    ] {
        for script in SCRIPTS {
            let source = vize_l0::cstr!("<template><p>{{{{{content}}}}}</p></template>{script}");
            for width in WIDTHS {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    let generated = newline(ending);
                    let expected = if width < 80 {
                        let body = narrow.replace('\n', generated);
                        vize_l0::cstr!("<p>{{{{{generated}    {body}{generated}  }}}}</p>")
                    } else {
                        vize_l0::cstr!("<p>{{{{ {flat} }}}}</p>")
                    };
                    let output = format(&source, options(width, ending));
                    assert_eq!(output, expected, "{source} / {width} / {ending:?}");
                    let replay = vize_l0::cstr!("<template>{output}</template>{script}");
                    assert_eq!(format(&replay, options(width, ending)), output);
                }
            }
        }
        assert_preserved(content);
    }
}

#[test]
fn sequence_commas_skip_typed_comments_and_keep_authored_entities_and_physical_lf_columns() {
    for (content, body) in [
        ("&#39;//x&#39;\n,a", "&#39;//x&#39;\n, a"),
        ("a,&quot;//x&quot;\r\n,b", "a, &quot;//x&quot;\r\n, b"),
        ("(a,&#39;//x&#39;\n)", "(a, &#39;//x&#39;\n)"),
        ("f((&#39;//x&#39;\n,a))", "f ( (&#39;//x&#39;\n, a) )"),
        ("obj[&#39;//x&#39;\n,a]", "obj [ &#39;//x&#39;\n, a ]"),
        ("a//,\n,b", "a//,\n, b"),
        ("a,//,\n b", "a,//,\n b"),
        ("a /*原\r\n*/,b", "a /*原\r\n*/, b"),
        ("a,&#47;*//x*&#47;\n b", "a,&#47;*//x*&#47;\n b"),
        ("(a /*,*/, /*,\r\n*/ b)", "(a /*,*/, /*,\r\n*/ b)"),
        ("&#39;//x&#39;\n&#44;b", "&#39;//x&#39;\n&#44; b"),
    ] {
        for script in SCRIPTS {
            let source = vize_l0::cstr!("<template><p>{{{{{content}}}}}</p></template>{script}");
            for width in WIDTHS {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    let generated = newline(ending);
                    let output = format(&source, options(width, ending));
                    assert_eq!(
                        output,
                        vize_l0::cstr!("<p>{{{{{generated}    {body}{generated}  }}}}</p>"),
                        "{source} / {width} / {ending:?}"
                    );
                    let replay = vize_l0::cstr!("<template>{output}</template>{script}");
                    assert_eq!(format(&replay, options(width, ending)), output);
                }
            }
        }
        assert_preserved(content);
    }
}

#[test]
fn sequence_last_encoded_quote_keeps_the_complete_authored_trimmed_tail_for_descriptor_replay() {
    for (content, body, tail) in [
        ("a,&#39;//x&#39;\n", "a, &#39;//x&#39;", "\n"),
        ("(a,&quot;//x&quot;)\r\n", "(a, &quot;//x&quot;)", "\r\n"),
        ("a,(b,&#39;//x&#39;)\n \t", "a, (b, &#39;//x&#39;)", "\n \t"),
    ] {
        for script in SCRIPTS {
            let source = vize_l0::cstr!("<template><p>{{{{{content}}}}}</p></template>{script}");
            for width in WIDTHS {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    let generated = newline(ending);
                    let output = format(&source, options(width, ending));
                    assert_eq!(
                        output,
                        vize_l0::cstr!("<p>{{{{{generated}    {body}{tail}}}}}</p>"),
                        "{source} / {width} / {ending:?}"
                    );
                    let replay = vize_l0::cstr!("<template>{output}</template>{script}");
                    assert_eq!(format(&replay, options(width, ending)), output);
                }
            }
        }
        assert_preserved(content);
    }
}

#[test]
fn actual_descriptor_requires_literal_lf_after_encoded_quote_sequence_child() {
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source = vize_l0::cstr!("<template>{{{{&#39;//x&#39;\n,a}}}}</template>{script}");
        let owner = crate::selected(&arena, &source);
        let original = crate::operands(&owner);
        let syntax = original.first().unwrap().syntax();
        assert_eq!(syntax.hole(), None);
        assert_eq!(syntax.comments().count(), 0);
        assert!(syntax.source().decode_map().is_some());
        assert!(
            matches!(syntax.expression().unwrap(), Expression::SequenceExpression(sequence)
            if matches!(sequence.expressions.first().unwrap(),
                Expression::StringLiteral(string) if string.value.as_str() == "//x"))
        );
        for terminator in ["", "\r", "&#10;"] {
            let source =
                vize_l0::cstr!("<template>{{{{&#39;//x&#39;{terminator},a}}}}</template>{script}");
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
