//! Whole selected arrays retain genuine elements, holes and authored framing.

use super::{format, preservation::assert_preserved};
use oxc_ast::ast::Expression;
use vize_glyph::native_doc::{LineEnding, PrintOptions};
use vize_l0::Allocator;

#[path = "array/custody.rs"]
mod custody;
#[path = "array/refusals.rs"]
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
fn ordinary_arrays_pin_whole_selected_js_ts_outputs_and_exact_hole_commas() {
    for (content, body) in [
        ("[]", "[ ]"),
        ("[a]", "[ a ]"),
        ("[a,b]", "[ a, b ]"),
        ("[ a , b , ]", "[ a, b, ]"),
        ("[a,]", "[ a, ]"),
        ("[,]", "[ , ]"),
        ("[,,]", "[ , , ]"),
        ("[,a]", "[ , a ]"),
        ("[a,,]", "[ a, , ]"),
        ("[a,,b]", "[ a, , b ]"),
        ("[,,a,,]", "[ , , a, , ]"),
        ("[a,,,b,]", "[ a, , , b, ]"),
        ("[[,],[],[a,,]]", "[ [ , ], [ ], [ a, , ] ]"),
        ("[f(a),obj[key],-1]", "[ f ( a ), obj [ key ], - 1 ]"),
        ("[日本,'x\\x20y',null]", "[ 日本, 'x\\x20y', null ]"),
        ("[true,false,0xA_F]", "[ true, false, 0xA_F ]"),
        ("[a].length", "[ a ] . length"),
        ("[a][0]", "[ a ] [ 0 ]"),
        ("f([a,,])", "f ( [ a, , ] )"),
        ("[/*empty*/]", "[/*empty*/]"),
        ("[a/*,*/,/*,*/b]", "[ a/*,*/,/*,*/b ]"),
        ("[,/*keep*/,a/*tail*/,]", "[ ,/*keep*/, a/*tail*/, ]"),
        ("[,/*,*/]", "[ ,/*,*/]"),
        ("[,,/*,*/a]", "[ , ,/*,*/a ]"),
        ("[a,/*,*/]", "[ a,/*,*/]"),
        ("[a/*,*/,/*,*/]", "[ a/*,*/,/*,*/]"),
        (
            "&#91;a&#44;&#44;b&#44;&#93;",
            "&#91; a&#44; &#44; b&#44; &#93;",
        ),
        ("&#91;&#44;&#44;&#93;", "&#91; &#44; &#44; &#93;"),
        ("[&#9;a&#32;,&#10;b]", "[&#9;a&#32;,&#10;b ]"),
        ("[/*#__PURE__*/f(a)]", "[/*#__PURE__*/f ( a ) ]"),
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
fn arrays_add_no_breaks_while_supported_children_keep_their_original_group_layout() {
    for (content, flat, narrow) in [
        ("[a+b,c]", "[ a + b, c ]", "[ a +\n      b, c ]"),
        ("[a?b:c,]", "[ a ? b : c, ]", "[ a ?\n      b :\n      c, ]"),
        (
            "[f(a+b),,c]",
            "[ f ( a + b ), , c ]",
            "[ f ( a +\n      b ), , c ]",
        ),
        (
            "a?[b,,]:[c]",
            "a ? [ b, , ] : [ c ]",
            "a ?\n      [ b, , ] :\n      [ c ]",
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
fn array_holes_and_comments_keep_authored_entities_and_physical_lf_columns() {
    for (content, body) in [
        ("[&#39;//x&#39;\n]", "[ &#39;//x&#39;\n]"),
        ("[&quot;//x&quot;\r\n]", "[ &quot;//x&quot;\r\n]"),
        ("[&#39;//x&#39;\n,,a]", "[ &#39;//x&#39;\n, , a ]"),
        ("[,&quot;//x&quot;\r\n,]", "[ , &quot;//x&quot;\r\n, ]"),
        ("[a//,\n,b]", "[ a//,\n, b ]"),
        ("[,//,\n,a]", "[ ,//,\n, a ]"),
        ("[a,//,\n]", "[ a,//,\n]"),
        ("[a//,\n,]", "[ a//,\n, ]"),
        ("[a,,//,\n]", "[ a, ,//,\n]"),
        ("[a /*原\r\n*/,b]", "[ a /*原\r\n*/, b ]"),
        ("[a,&#47;*//x*&#47;\n,b]", "[ a,&#47;*//x*&#47;\n, b ]"),
        ("[&#39;//x&#39;\n/*,*/,a]", "[ &#39;//x&#39;\n/*,*/, a ]"),
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
fn encoded_quote_array_keeps_the_complete_authored_trimmed_tail_for_descriptor_replay() {
    for (content, body, tail) in [
        ("[&#39;//x&#39;]\n", "[ &#39;//x&#39; ]", "\n"),
        ("[&quot;//x&quot;,]\r\n", "[ &quot;//x&quot;, ]", "\r\n"),
        ("[&#39;//x&#39;,,]\n \t", "[ &#39;//x&#39;, , ]", "\n \t"),
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
fn actual_descriptor_requires_literal_lf_for_encoded_quote_array_not_a_decoded_terminator() {
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source = vize_l0::cstr!("<template>{{{{[&#39;//x&#39;\n]}}}}</template>{script}");
        let owner = crate::selected(&arena, &source);
        let original = crate::operands(&owner);
        let syntax = original.first().unwrap().syntax();
        assert_eq!(syntax.hole(), None);
        assert_eq!(syntax.comments().count(), 0);
        assert!(syntax.source().decode_map().is_some());
        assert!(
            matches!(syntax.expression().unwrap(), Expression::ArrayExpression(array)
            if matches!(array.elements.first().unwrap().as_expression().unwrap(),
                Expression::StringLiteral(string) if string.value.as_str() == "//x"))
        );
        for terminator in ["", "\r", "&#10;"] {
            let source =
                vize_l0::cstr!("<template>{{{{[&#39;//x&#39;{terminator}]}}}}</template>{script}");
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
