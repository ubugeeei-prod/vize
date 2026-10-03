//! Original BigInt spelling and metadata through whole selected templates.

use super::{format, preservation::assert_preserved};
use oxc_ast::ast::{BigIntLiteral, BigintBase, Expression};
use vize_glyph::native_doc::{LineEnding, PrintOptions};
use vize_l0::Allocator;

#[path = "bigint/custody.rs"]
mod custody;
#[path = "bigint/refusals.rs"]
mod refusals;

const WIDTHS: [usize; 5] = [0, 1, 7, 80, 200];
const SCRIPTS: [&str; 2] = ["", "<script setup lang=ts>let a=1</script>"];

#[derive(Debug, PartialEq, Eq)]
pub(super) struct BigIntFingerprint {
    base: BigintBase,
    value: std::string::String,
    raw: Option<std::string::String>,
    authored: std::string::String,
}

impl BigIntFingerprint {
    pub(super) fn from_literal(literal: &BigIntLiteral<'_>, authored: std::string::String) -> Self {
        Self {
            base: literal.base,
            value: literal.value.as_str().to_owned(),
            raw: literal.raw.map(|raw| raw.as_str().to_owned()),
            authored,
        }
    }
}

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
fn selected_bigints_pin_all_bases_separators_and_complete_js_ts_outputs() {
    // These preserve original syntax; arithmetic and callable values are not executed.
    for (content, body) in [
        ("0n", "0n"),
        ("42n", "42n"),
        ("1_000n", "1_000n"),
        ("9007199254740993n", "9007199254740993n"),
        ("0b10_1010n", "0b10_1010n"),
        ("0B101010n", "0B101010n"),
        ("0o5_2n", "0o5_2n"),
        ("0O52n", "0O52n"),
        ("0x2_An", "0x2_An"),
        ("0X2an", "0X2an"),
        ("&#49;&#110;", "&#49;&#110;"),
        ("&#48;b10_1010n", "&#48;b10_1010n"),
        ("0x2_A&#110;", "0x2_A&#110;"),
        ("(1n)", "(1n)"),
        ("((0x2An))", "((0x2An))"),
        ("-1n", "- 1n"),
        ("+1n", "+ 1n"),
        ("~1n", "~ 1n"),
        ("typeof 1n", "typeof 1n"),
        ("1n.value", "1n . value"),
        ("obj[1n]", "obj [ 1n ]"),
        ("f(1n,0x2An,)", "f ( 1n, 0x2An, )"),
        ("1n()", "1n ( )"),
        ("[1n,,0o52n]", "[ 1n, , 0o52n ]"),
        ("{a:1n,b:0b10n}", "{ a: 1n, b: 0b10n }"),
        ("1n,2n", "1n, 2n"),
        ("/*#__PURE__*/f(1n)", "/*#__PURE__*/f ( 1n )"),
        ("1n&#32;,&#9;2n", "1n&#32;,&#9;2n"),
        ("/*before*/1n/*after*/", "/*before*/1n/*after*/"),
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
fn bigints_keep_original_child_groups_and_operator_token_boundaries() {
    for (content, flat, narrow) in [
        ("1n+2n", "1n + 2n", "1n +\n      2n"),
        ("1n+1", "1n + 1", "1n +\n      1"),
        ("1n- -2n", "1n - - 2n", "1n -\n      - 2n"),
        ("(-1n)**2n", "(- 1n) ** 2n", "(- 1n) **\n      2n"),
        (
            "1n&amp;&amp;2n",
            "1n &amp;&amp; 2n",
            "1n &amp;&amp;\n      2n",
        ),
        ("1n?2n:3n", "1n ? 2n : 3n", "1n ?\n      2n :\n      3n"),
        ("f(1n+2n)", "f ( 1n + 2n )", "f ( 1n +\n      2n )"),
        ("{a:1n+2n}", "{ a: 1n + 2n }", "{ a: 1n +\n      2n }"),
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
fn bigint_tokens_comments_and_encoded_quote_gaps_keep_physical_lf_without_generated_indent() {
    for (content, body) in [
        ("1n//,\n,2n", "1n//,\n, 2n"),
        ("1n,/*原\r\n*/2n", "1n,/*原\r\n*/2n"),
        ("1n,&#39;//x&#39;\n,2n", "1n, &#39;//x&#39;\n, 2n"),
        ("f(1n,&quot;//x&quot;\r\n)", "f ( 1n, &quot;//x&quot;\r\n)"),
        ("{a:1n,b:&#39;//x&#39;\n}", "{ a: 1n, b: &#39;//x&#39;\n}"),
        ("1n &#47;*//x*&#47;\n,2n", "1n &#47;*//x*&#47;\n, 2n"),
        ("1n&#44;\n2n", "1n&#44;\n2n"),
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
fn bigint_original_comment_and_encoded_quote_tails_keep_exact_lf_crlf_and_tabs() {
    for (content, body, tail) in [
        ("1n//x\n", "1n//x", "\n"),
        ("1n &#47;*//x*&#47;\r\n", "1n &#47;*//x*&#47;", "\r\n"),
        ("1n,&#39;//x&#39;\n \t", "1n, &#39;//x&#39;", "\n \t"),
        ("1n&#32;\n", "1n&#32;", "\n"),
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
fn original_bigint_sequence_descriptor_and_language_profile_require_actual_physical_lf() {
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source = vize_l0::cstr!("<template>{{{{1n,&#39;//x&#39;\n,2n}}}}</template>{script}");
        let owner = crate::selected(&arena, &source);
        let original = crate::operands(&owner);
        let syntax = original.first().unwrap().syntax();
        assert_eq!(syntax.hole(), None);
        assert_eq!(syntax.comments().count(), 0);
        assert!(syntax.source().decode_map().is_some());
        assert!(syntax.source_type().is_module());
        assert_eq!(syntax.source_type().is_typescript(), !script.is_empty());
        let Expression::SequenceExpression(sequence) = syntax.expression().unwrap() else {
            panic!("actual Sequence")
        };
        assert!(
            matches!(sequence.expressions.first().unwrap(), Expression::BigIntLiteral(literal) if literal.value.as_str() == "1" && literal.base == BigintBase::Decimal)
        );
        for terminator in ["", "\r", "&#10;"] {
            let source = vize_l0::cstr!(
                "<template>{{{{1n,&#39;//x&#39;{terminator},2n}}}}</template>{script}"
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

#[test]
fn original_bigint_method_double_closer_keeps_earliest_selected_safety_refusal() {
    use vize_glyph::native_doc::{NativeTemplateRefusal, native_template_document};
    use vize_l1::embed::syntax::EmbedHole;
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source = vize_l0::cstr!(
            "<template>{{{{1n}}}}<p>{{{{/*原*/ {{a:1n,b(){{}}}} }}}}</p></template>{script}"
        );
        assert_eq!(
            source.strip_suffix(script).unwrap(),
            "<template>{{1n}}<p>{{/*原*/ {a:1n,b(){}} }}</p></template>"
        );
        let owner = crate::selected(&arena, &source);
        let original = crate::operands(&owner);
        let operand = original.get(1).unwrap();
        let syntax = operand.syntax();
        assert_eq!(syntax.hole(), Some(EmbedHole::SafetyAdmission));
        assert!(syntax.expression().is_none());
        assert_eq!(syntax.diagnostics().count(), 0);
        assert_eq!(syntax.comments().count(), 0);
        assert_eq!(operand.raw_content(), "/*原*/ {a:1n,b(){");
        assert_eq!(operand.full_span().slice(&source), "{{/*原*/ {a:1n,b(){}}");
        let view = syntax.source();
        let refs = original.iter().collect::<std::vec::Vec<_>>();
        assert!(matches!(
            native_template_document(&owner, &refs, &arena),
            Err(NativeTemplateRefusal::OperandRejected {
                index: 1,
                hole: Some(EmbedHole::SafetyAdmission),
                ..
            })
        ));
        assert!(core::ptr::eq(syntax.source().text(), view.text()));
        assert!(core::ptr::eq(
            syntax.source().authored_root(),
            source.as_str()
        ));
        assert_eq!(syntax.source().span(), view.span());
        assert_eq!(operand.content_span().slice(&source), operand.raw_content());
        assert_eq!(
            vize_l1::check_fidelity(&owner.component().carrier().tree),
            Ok(())
        );
    }
}
