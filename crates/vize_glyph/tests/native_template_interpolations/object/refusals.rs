//! Whole explicit Object refusals preserve original observations and bounds.

use super::{SCRIPTS, assert_preserved, comment_snapshot, format, options};
use crate::{operands, selected};
use oxc_ast::ast::{Expression, ObjectPropertyKind, PropertyKey};
use vize_glyph::native_doc::{
    ExpressionRefusal, LineEnding, NativeTemplateRefusal, expression_document,
    native_template_document,
};
use vize_l0::{Allocator, SourceRoot};

fn assert_unsupported(content: &str, script: &str, prototype_key: bool) {
    let arena = Allocator::default();
    let source = vize_l0::cstr!(
        "<template>{{{{{} }}}}<p>{{{{/*keep*/ {content} }}}}</p></template>{script}",
        "{safe:a}"
    );
    let owner = selected(&arena, &source);
    let original = operands(&owner);
    let operand = original.get(1).unwrap();
    let syntax = operand.syntax();
    assert_eq!(syntax.hole(), None, "{source}");
    assert!(syntax.admitted_expression().is_some(), "{source}");
    if prototype_key {
        let Expression::ObjectExpression(object) = syntax.expression().unwrap() else {
            panic!("actual Object")
        };
        let ObjectPropertyKind::ObjectProperty(property) = object.properties.first().unwrap()
        else {
            panic!("actual explicit key")
        };
        assert!(!property.computed && !property.method && !property.shorthand);
        assert!(
            matches!(&property.key,
            PropertyKey::StaticIdentifier(key) if key.name.as_str() == "__proto__")
                || matches!(&property.key, PropertyKey::StringLiteral(key) if key.value.as_str() == "__proto__")
        );
    }
    let roots = original
        .iter()
        .map(|operand| core::ptr::from_ref(operand.syntax().expression().unwrap()))
        .collect::<std::vec::Vec<_>>();
    let comments = comment_snapshot(syntax, &source);
    let view = syntax.source();
    let map = view
        .decode_map()
        .map(|map| (map.segments().as_ptr(), map.segments().to_vec()));
    let diagnostics = syntax.diagnostics().count();
    let raw = operand.raw_content().as_ptr();
    let refs = original.iter().collect::<std::vec::Vec<_>>();
    assert!(
        matches!(
            native_template_document(&owner, &refs, &arena),
            Err(NativeTemplateRefusal::Expression {
                refusal: ExpressionRefusal::UnsupportedNode { .. },
                ..
            })
        ),
        "{source}"
    );
    assert_eq!(
        original
            .iter()
            .map(|operand| core::ptr::from_ref(operand.syntax().expression().unwrap()))
            .collect::<std::vec::Vec<_>>(),
        roots
    );
    assert_eq!(comment_snapshot(syntax, &source), comments);
    assert_eq!(
        syntax
            .source()
            .decode_map()
            .map(|map| (map.segments().as_ptr(), map.segments().to_vec())),
        map
    );
    assert_eq!(syntax.source().span(), view.span());
    assert!(core::ptr::eq(syntax.source().text(), view.text()));
    assert!(core::ptr::eq(
        syntax.source().authored_root(),
        source.as_str()
    ));
    assert_eq!(syntax.diagnostics().count(), diagnostics);
    assert_eq!(operand.raw_content().as_ptr(), raw);
    for operand in &original {
        assert_eq!(operand.content_span().slice(&source), operand.raw_content());
    }
}

#[test]
fn unsupported_object_keys_properties_values_and_decoded_prototype_keys_refuse_whole_documents() {
    for (content, ts_only) in [
        ("{a}", false),
        ("{a:b,c}", false),
        ("{...a}", false),
        ("{a:b,...c}", false),
        ("{a(){} }", false),
        ("{get a(){return b} }", false),
        ("{set a(b){} }", false),
        ("{async a(){} }", false),
        ("{*a(){} }", false),
        ("{[a]:b}", false),
        ("{['a']:b}", false),
        ("{[(a,b)]:c}", false),
        ("{1n:a}", false),
        ("{a:this}", false),
        ("{a:b,c:this}", false),
        ("{a:fn(...b)}", false),
        ("{a:/x/}", false),
        ("{a:`x`}", false),
        ("{a:()=>b}", false),
        ("{a:function(){} }", false),
        ("{a:await b}", false),
        ("{a:(b=c)}", false),
        ("{a:b++}", false),
        ("{a:new B}", false),
        ("{a:import('b')}", false),
        ("{a:b?.c}", false),
        ("{a:f(...b)}", false),
        ("{a:[...b]}", false),
        ("f({a:++b})", false),
        ("[{a:b?.c}]", false),
        ("a?{b:c}:{d:this}", false),
        ("a,&#123;b:this&#125;", false),
        ("{a:(b as T)}", true),
        ("{a:b!}", true),
        ("{a:f<T>(b)}", true),
    ] {
        for script in SCRIPTS {
            if !ts_only || !script.is_empty() {
                assert_unsupported(content, script, false);
            }
        }
    }
    // These are conservative source-consumer refusals, not JS early-error proofs.
    for content in [
        "{__proto__:a}",
        "{'__proto__':a}",
        "{\"__proto__\":a}",
        "{__pr\\u006fto__:a}",
        "{'__pro\\u0074o__':a}",
        "{__pr&#111;to__:a}",
        "{&#39;__proto__&#39;:a}",
        "{__proto__:a,__proto__:b}",
    ] {
        for script in SCRIPTS {
            assert_unsupported(content, script, true);
        }
    }
}

#[test]
fn balanced_object_syntax_holes_keep_original_observations_after_a_supported_property() {
    for content in [
        "{a:}",
        "{a:,b:c}",
        "{a b}",
        "{,a:b}",
        "{a:b,,c:d}",
        "{a=b}",
        "{#a:b}",
        "{[a,b]:c}",
    ] {
        for script in SCRIPTS {
            let arena = Allocator::default();
            let source = vize_l0::cstr!(
                "<template>{{{{{} }}}}<p>{{{{/*keep*/ {content} }}}}</p></template>{script}",
                "{safe:a}"
            );
            let owner = selected(&arena, &source);
            let original = operands(&owner);
            let operand = original.get(1).unwrap();
            assert!(operand.syntax().hole().is_some(), "{source}");
            let diagnostics = operand.syntax().diagnostics().count();
            let comments = operand.syntax().comments().count();
            let refs = original.iter().collect::<std::vec::Vec<_>>();
            assert!(
                matches!(
                    native_template_document(&owner, &refs, &arena),
                    Err(NativeTemplateRefusal::OperandRejected {
                        index: 1,
                        hole: Some(_),
                        ..
                    })
                ),
                "{source}"
            );
            assert_eq!(operand.syntax().diagnostics().count(), diagnostics);
            assert_eq!(operand.syntax().comments().count(), comments);
            assert_eq!(operand.content_span().slice(&source), operand.raw_content());
        }
    }
}

#[test]
fn non_ascii_object_key_colon_comma_and_brace_gaps_have_no_plain_whitespace_authority() {
    for content in [
        "{\u{a0}a:b}",
        "{a\u{a0}:b}",
        "{a:\u{a0}b}",
        "{a:b\u{a0},c:d}",
        "{a:b,\u{a0}c:d}",
        "{a:b,\u{a0}}",
        "{\u{a0}}",
        "{a&#160;:b}",
        "{a:&#160;b}",
    ] {
        for script in SCRIPTS {
            let arena = Allocator::default();
            let source = vize_l0::cstr!("<template>{{{{{content} }}}}</template>{script}");
            let owner = selected(&arena, &source);
            let original = operands(&owner);
            let operand = original.first().unwrap();
            let syntax = operand.syntax();
            assert_eq!(syntax.hole(), None);
            let root = core::ptr::from_ref(syntax.expression().unwrap());
            let refs = original.iter().collect::<std::vec::Vec<_>>();
            assert!(
                matches!(
                    native_template_document(&owner, &refs, &arena),
                    Err(NativeTemplateRefusal::Expression {
                        refusal: ExpressionRefusal::InvalidGap { .. },
                        ..
                    })
                ),
                "{source}"
            );
            assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), root);
            assert_eq!(operand.content_span().slice(&source), operand.raw_content());
        }
    }
}

#[test]
fn original_object_operands_refuse_foreign_missing_extra_reversed_duplicated_or_wrong_source_joins()
{
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source = vize_l0::cstr!(
            "<template>{{{{{} }}}}<p>{{{{{} }}}}</p></template>{script}",
            "{a:b}",
            "{c:d}"
        );
        let owner = selected(&arena, &source);
        let original = operands(&owner);
        let a = original.first().unwrap();
        let b = original.get(1).unwrap();
        let roots = [
            core::ptr::from_ref(a.syntax().expression().unwrap()),
            core::ptr::from_ref(b.syntax().expression().unwrap()),
        ];
        assert!(native_template_document(&owner, &[a, b], &arena).is_ok());
        assert!(matches!(
            native_template_document(&owner, &[], &arena),
            Err(NativeTemplateRefusal::MissingOperand { index: 0, .. })
        ));
        assert!(matches!(
            native_template_document(&owner, &[a], &arena),
            Err(NativeTemplateRefusal::MissingOperand { index: 1, .. })
        ));
        assert!(matches!(
            native_template_document(&owner, &[a, b, a], &arena),
            Err(NativeTemplateRefusal::ExtraOperand { index: 2 })
        ));
        for (refs, index) in [([b, a], 0), ([a, a], 1)] {
            assert!(
                matches!(native_template_document(&owner, &refs, &arena), Err(NativeTemplateRefusal::OperandRejected { index: rejected, .. }) if rejected == index)
            );
        }
        let copied = source.to_owned();
        for foreign in [selected(&arena, &source), selected(&arena, &copied)] {
            assert!(matches!(
                native_template_document(&foreign, &[a, b], &arena),
                Err(NativeTemplateRefusal::OperandRejected { index: 0, .. })
            ));
        }
        let whole = SourceRoot::new(&source).unwrap();
        for block in [
            SourceRoot::new(&copied).unwrap().whole_block(),
            whole.block(&source[..1], 0).unwrap(),
        ] {
            assert!(matches!(
                expression_document(a.syntax(), block, &arena),
                Err(ExpressionRefusal::SourceMismatch { .. })
            ));
        }
        assert_eq!(
            [
                core::ptr::from_ref(a.syntax().expression().unwrap()),
                core::ptr::from_ref(b.syntax().expression().unwrap())
            ],
            roots
        );
        assert_eq!(a.raw_content(), "{a:b} ");
        assert_eq!(b.raw_content(), "{c:d} ");
    }
}

#[test]
fn original_object_keys_values_and_empty_objects_keep_the_independent_depth_sixteen_boundary() {
    for (count, content, body) in [(15, "{a:b}", "{ a: b }"), (16, "{}", "{ }")] {
        let supported = vize_l0::cstr!("{}{content}", "!".repeat(count));
        for script in SCRIPTS {
            let source = vize_l0::cstr!("<template>{{{{{supported} }}}}</template>{script}");
            assert_eq!(
                format(&source, options(200, LineEnding::Lf)),
                vize_l0::cstr!("{{{{ {}{body} }}}}", "! ".repeat(count))
            );
        }
        assert_preserved(&vize_l0::cstr!("{supported} "));
    }
    for (count, content, leaf) in [(16, "{a:b}", "a"), (17, "{}", "{}")] {
        for script in SCRIPTS {
            let refused = vize_l0::cstr!("{}{content}", "!".repeat(count));
            let source = vize_l0::cstr!("<template>{{{{{refused} }}}}</template>{script}");
            let arena = Allocator::default();
            let owner = selected(&arena, &source);
            let original = operands(&owner);
            let operand = original.first().unwrap();
            let syntax = operand.syntax();
            assert_eq!(
                syntax.hole(),
                None,
                "whole input stays below the unchanged 31-unit limit"
            );
            let root = core::ptr::from_ref(syntax.expression().unwrap());
            let refs = original.iter().collect::<std::vec::Vec<_>>();
            let Err(NativeTemplateRefusal::Expression {
                refusal: ExpressionRefusal::DepthLimit { span },
                ..
            }) = native_template_document(&owner, &refs, &arena)
            else {
                panic!("actual child depth seventeen must refuse")
            };
            assert_eq!(span.slice(syntax.source().text()), leaf);
            assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), root);
            assert_eq!(operand.content_span().slice(&source), operand.raw_content());
        }
    }
}
