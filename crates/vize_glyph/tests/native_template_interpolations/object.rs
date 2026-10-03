//! Whole selected explicit Object properties retain their original wire source.

use super::{format, preservation::assert_preserved};
use oxc_ast::ast::{Expression, ObjectPropertyKind, PropertyKey, PropertyKind};
use vize_glyph::native_doc::{LineEnding, PrintOptions};
use vize_l0::Allocator;

#[path = "object/custody.rs"]
mod custody;
#[path = "object/refusals.rs"]
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

type CommentSnapshot = std::vec::Vec<(
    oxc_ast::ast::CommentKind,
    vize_l0::Span,
    vize_l0::Span,
    *const u8,
    std::string::String,
)>;

fn comment_snapshot(
    syntax: &vize_l1::embed::syntax::RetainedExpression<'_>,
    source: &str,
) -> CommentSnapshot {
    syntax
        .comments()
        .map(|comment| {
            (
                comment.kind(),
                comment.decoded_span().unwrap(),
                comment.authored_span().unwrap(),
                comment.text().unwrap().as_ptr(),
                comment.authored_span().unwrap().slice(source).to_owned(),
            )
        })
        .collect()
}

#[test]
fn selected_explicit_objects_pin_complete_js_ts_outputs_without_object_breaks() {
    for (content, body) in [
        ("{}", "{ }"),
        ("{a:b}", "{ a: b }"),
        ("{a:b,c:d,}", "{ a: b, c: d, }"),
        ("{'a':b,\"c\":d}", "{ 'a': b, \"c\": d }"),
        ("{0:b,1.0:c,0xCA_FE:d}", "{ 0: b, 1.0: c, 0xCA_FE: d }"),
        ("{get:a,set:b,default:c}", "{ get: a, set: b, default: c }"),
        ("{a:obj.x,a:f()}", "{ a: obj . x, a: f ( ) }"),
        ("{a:true,b:null,c:0.5}", "{ a: true, b: null, c: 0.5 }"),
        ("{a:'x',b:false}", "{ a: 'x', b: false }"),
        ("{a:{b:c},d:[]}", "{ a: { b: c }, d: [ ] }"),
        ("{a:[b,,],c:f(d)}", "{ a: [ b, , ], c: f ( d ) }"),
        ("{a:(b,c)}", "{ a: (b, c) }"),
        ("f({a:b},c)", "f ( { a: b }, c )"),
        ("[{a:b},{c:d}]", "[ { a: b }, { c: d } ]"),
        ("({a:b}).a", "({ a: b }) . a"),
        ("{a:b},c", "{ a: b }, c"),
        ("{日本:日本,\\u0061:b}", "{ 日本: 日本, \\u0061: b }"),
        ("{'\\u0061':b}", "{ '\\u0061': b }"),
        (
            "&#123;a&#58;b&#44;&#39;c&#39;:d&#44;&#125;",
            "&#123; a&#58; b&#44; &#39;c&#39;: d&#44; &#125;",
        ),
        ("{a&#32;&#58;&#9;b}", "{ a&#32;&#58;&#9;b }"),
        ("{&#39;a&#39;:b}", "{ &#39;a&#39;: b }"),
        ("{/*only*/}", "{/*only*/}"),
        ("{/*open*/a:b}", "{/*open*/a: b }"),
        ("{a/*:*/:/*:*/b}", "{ a/*:*/:/*:*/b }"),
        ("{a:/*:*/b/*,*/,/*:*/c:d}", "{ a:/*:*/b/*,*/,/*:*/c: d }"),
        ("{a:b/*,*/,/*tail*/}", "{ a: b/*,*/,/*tail*/}"),
        ("{a:/*#__PURE__*/f()}", "{ a:/*#__PURE__*/f ( ) }"),
    ] {
        for script in SCRIPTS {
            let source = vize_l0::cstr!(
                "<!--前--><template><!--keep--><p>{{{{{content} }}}}</p></template>{script}"
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
                    assert_eq!(format(&replay, options(width, ending)), output);
                }
            }
        }
        assert_preserved(&vize_l0::cstr!("{content} "));
    }
}

#[test]
fn object_values_keep_original_infix_conditional_and_nested_child_groups() {
    for (content, flat, narrow) in [
        ("{a:b+c}", "{ a: b + c }", "{ a: b +\n      c }"),
        (
            "{a:b?c:d}",
            "{ a: b ? c : d }",
            "{ a: b ?\n      c :\n      d }",
        ),
        (
            "{a:{b:c+d} }",
            "{ a: { b: c + d } }",
            "{ a: { b: c +\n      d } }",
        ),
        (
            "f({a:b+c})",
            "f ( { a: b + c } )",
            "f ( { a: b +\n      c } )",
        ),
        (
            "[{a:b?c:d}]",
            "[ { a: b ? c : d } ]",
            "[ { a: b ?\n      c :\n      d } ]",
        ),
        (
            "{a:(b+c,d)}",
            "{ a: (b + c, d) }",
            "{ a: (b +\n      c, d) }",
        ),
        (
            "a?{b:c}:{d:e}",
            "a ? { b: c } : { d: e }",
            "a ?\n      { b: c } :\n      { d: e }",
        ),
    ] {
        for script in SCRIPTS {
            let source = vize_l0::cstr!("<template><p>{{{{{content} }}}}</p></template>{script}");
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
        assert_preserved(&vize_l0::cstr!("{content} "));
    }
}

#[test]
fn object_keys_colons_commas_and_suffixes_keep_typed_comments_and_physical_lf() {
    for (content, body) in [
        ("{&#39;//x&#39;\n:a}", "{ &#39;//x&#39;\n: a }"),
        ("{a:&quot;//x&quot;\r\n}", "{ a: &quot;//x&quot;\r\n}"),
        ("{a:&#39;//x&#39;\n,b:c}", "{ a: &#39;//x&#39;\n, b: c }"),
        ("{a:&#47;*//x*&#47;\n b}", "{ a:&#47;*//x*&#47;\n b }"),
        ("{a:b//,\n,c:d}", "{ a: b//,\n, c: d }"),
        ("{a:b,//,\r\n c:d}", "{ a: b,//,\r\n c: d }"),
        ("{a:b//,\n,/*tail*/}", "{ a: b//,\n,/*tail*/}"),
        ("{a:b,/*,\r\n*/}", "{ a: b,/*,\r\n*/}"),
        ("{a:/*原\n*/b}", "{ a:/*原\n*/b }"),
        ("{a&#58;\nb&#44;\nc:d}", "{ a&#58;\nb&#44;\nc: d }"),
    ] {
        for script in SCRIPTS {
            let source = vize_l0::cstr!("<template><p>{{{{{content} }}}}</p></template>{script}");
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
        assert_preserved(&vize_l0::cstr!("{content} "));
    }
}

#[test]
fn mapped_object_braces_keep_the_complete_original_trimmed_lf_tail() {
    for (content, body, tail) in [
        (
            "&#123;a:&#39;//x&#39;&#125;\n",
            "&#123; a: &#39;//x&#39; &#125;",
            "\n",
        ),
        (
            "&#123;a:&quot;//x&quot;&#125;\r\n",
            "&#123; a: &quot;//x&quot; &#125;",
            "\r\n",
        ),
        ("({a:b})\n \t", "({ a: b })", "\n \t"),
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
fn actual_descriptor_requires_physical_lf_at_the_original_encoded_object_key_boundary() {
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source = vize_l0::cstr!(
            "<template>{{{{{} }}}}</template>{script}",
            "{&#39;//x&#39;\n:a}"
        );
        let owner = crate::selected(&arena, &source);
        let original = crate::operands(&owner);
        let syntax = original.first().unwrap().syntax();
        assert_eq!(syntax.hole(), None);
        assert_eq!(syntax.comments().count(), 0);
        assert!(syntax.source().decode_map().is_some());
        assert_eq!(syntax.source_type().is_typescript(), !script.is_empty());
        let Expression::ObjectExpression(object) = syntax.expression().unwrap() else {
            panic!("actual Object")
        };
        let ObjectPropertyKind::ObjectProperty(property) = object.properties.first().unwrap()
        else {
            panic!("actual property")
        };
        assert_eq!(property.kind, PropertyKind::Init);
        assert!(!property.method && !property.shorthand && !property.computed);
        assert!(
            matches!(&property.key, PropertyKey::StringLiteral(key) if key.value.as_str() == "//x")
        );
        for content in ["{a:b}", "a,{b:1}"] {
            let unsafe_source =
                vize_l0::cstr!("<template><p>{{{{/*原*/ {content}}}}}</p></template>{script}");
            let unsafe_owner = crate::selected(&arena, &unsafe_source);
            let unsafe_original = crate::operands(&unsafe_owner);
            let unsafe_operand = unsafe_original.first().unwrap();
            assert!(unsafe_operand.syntax().hole().is_some(), "{unsafe_source}");
            assert!(unsafe_operand.syntax().admitted_expression().is_none());
            let refs = unsafe_original.iter().collect::<std::vec::Vec<_>>();
            assert!(matches!(
                vize_glyph::native_doc::native_template_document(&unsafe_owner, &refs, &arena),
                Err(
                    vize_glyph::native_doc::NativeTemplateRefusal::OperandRejected {
                        index: 0,
                        hole: Some(_),
                        ..
                    }
                )
            ));
            assert_eq!(
                unsafe_operand.content_span().slice(&unsafe_source),
                unsafe_operand.raw_content()
            );
        }
        for terminator in ["", "\r", "&#10;"] {
            let content = vize_l0::cstr!("{{&#39;//x&#39;{terminator}:a}}");
            let source = vize_l0::cstr!("<template>{{{{{content} }}}}</template>{script}");
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
