//! Original model syntax ownership only: no assignability, File or DOM credit.

mod support;
mod trim;
use support::{assert_original, observe, selected, source_span, value_span};

use crate::embed::{DecodeSegmentKind, Grammar, Lang, Shape, syntax::EmbedHole};
use crate::markup::{
    NativeAttributeModelExpression, NativeAttributeOperandError, NativeTemplateGrammar,
};
use alloc::vec::Vec;
use oxc_ast::ast::{CommentKind, Expression, TSType};
use oxc_span::GetSpan;
use vize_l0::{Allocator, Span};

#[test]
fn actual_js_and_ts_module_heads_keep_original_unicode_profiles_and_ast_coordinates() {
    let arena = Allocator::default();
    for (source, raw, lang, grammar) in [
        (
            "<!--雪🌸-->\r\n<template><input title=x v-model='draft'/></template>",
            "draft",
            Lang::Js,
            NativeTemplateGrammar::JavaScriptModule,
        ),
        (
            "<script setup lang='ts'>let draft:string='x'</script>\r\n<template><input title=x v-model='draft as string'/></template>",
            "draft as string",
            Lang::Ts,
            NativeTemplateGrammar::TypeScriptModule,
        ),
    ] {
        let owner = selected(&arena, source);
        assert_eq!(owner.grammar(), grammar);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let attribute = element.attributes().nth(1).unwrap();
        let before = arena.allocated_bytes();
        let head = owner.observe_attribute_head(attribute.reborrow()).unwrap();
        let model = head.model().unwrap().unwrap();
        assert!(core::ptr::eq(model.head(), &head));
        assert_eq!(head.attribute().ordinal(), 1);
        assert_eq!(head.name_block().span(), source_span(source, "v-model"));
        assert_eq!(head.condition_kind(), None);
        assert_eq!(arena.allocated_bytes(), before);
        let operand = model.observe_expression().unwrap();
        assert_eq!(operand.name_span(), source_span(source, "v-model"));
        let value = value_span(source, raw);
        assert_eq!(operand.value_span(), value);
        assert_eq!(operand.raw_value(), raw);
        assert_eq!(value.slice(source), raw);
        let syntax = operand.syntax();
        assert_eq!(
            syntax.grammar(),
            Grammar {
                shape: Shape::Expr,
                lang
            }
        );
        assert!(syntax.source_type().is_module());
        assert_eq!(syntax.source_type().is_typescript(), lang == Lang::Ts);
        assert_eq!(syntax.parser_prefix(), 2);
        assert_eq!(syntax.hole(), None);
        assert_eq!(syntax.diagnostics().count(), 0);
        assert!(syntax.source().decode_map().is_none());
        let root = syntax.expression().unwrap();
        assert_eq!(syntax.authored_span(root.span()), Ok(value));
        if lang == Lang::Js {
            let Expression::Identifier(identifier) = root else {
                panic!("actual JS model identifier");
            };
            assert_eq!(identifier.name.as_str(), "draft");
            assert_eq!(identifier.span, oxc_span::Span::new(2, 7));
        } else {
            let Expression::TSAsExpression(cast) = root else {
                panic!("actual TS model expression");
            };
            let Expression::Identifier(identifier) = &cast.expression else {
                panic!("actual TS identifier");
            };
            assert_eq!(identifier.name.as_str(), "draft");
            assert_eq!(
                syntax.authored_span(identifier.span),
                Ok(Span::new(value.start, value.start + 5))
            );
            let TSType::TSStringKeyword(keyword) = &cast.type_annotation else {
                panic!("actual TS type");
            };
            assert_eq!(
                syntax.authored_span(keyword.span),
                Ok(Span::new(value.start + 9, value.end))
            );
        }
        assert_original(&operand, &owner, 1);
    }
}

#[test]
fn complete_entity_maps_lf_crlf_and_comments_retain_all_original_geometry() {
    let arena = Allocator::default();
    for (source, raw, decoded, shift) in [
        (
            "<template><input v-model='/*雪*/\n(α&amp;&amp;β)//尾\n'/></template>",
            "/*雪*/\n(α&amp;&amp;β)//尾\n",
            "/*雪*/\n(α&&β)//尾\n",
            0,
        ),
        (
            "<template><input v-model='/*雪*/\r\n(α&amp;&amp;β)//尾\r\n'/></template>",
            "/*雪*/\r\n(α&amp;&amp;β)//尾\r\n",
            "/*雪*/\r\n(α&&β)//尾\r\n",
            1,
        ),
    ] {
        let owner = selected(&arena, source);
        let operand = observe(&owner, 0);
        let syntax = operand.syntax();
        assert_eq!(operand.name_span(), Span::new(17, 24));
        assert_eq!(operand.value_span(), Span::new(26, 56 + 2 * shift));
        assert_eq!(operand.raw_value(), raw);
        assert_eq!(syntax.source().text(), decoded);
        assert_eq!(syntax.source().span(), operand.value_span());
        assert_eq!(syntax.parser_prefix(), 2);
        assert_eq!(syntax.hole(), None);
        assert_eq!(syntax.diagnostics().count(), 0);
        let map: Vec<_> = syntax
            .source()
            .decode_map()
            .unwrap()
            .segments()
            .iter()
            .map(|segment| (segment.decoded(), segment.authored(), segment.kind()))
            .collect();
        assert_eq!(
            map,
            [
                (
                    Span::new(0, 11 + shift),
                    Span::new(26, 37 + shift),
                    DecodeSegmentKind::Identity
                ),
                (
                    Span::new(11 + shift, 12 + shift),
                    Span::new(37 + shift, 42 + shift),
                    DecodeSegmentKind::Entity
                ),
                (
                    Span::new(12 + shift, 13 + shift),
                    Span::new(42 + shift, 47 + shift),
                    DecodeSegmentKind::Entity
                ),
                (
                    Span::new(13 + shift, 22 + 2 * shift),
                    Span::new(47 + shift, 56 + 2 * shift),
                    DecodeSegmentKind::Identity
                ),
            ]
        );
        let comments: Vec<_> = syntax
            .comments()
            .map(|comment| {
                (
                    comment.kind(),
                    comment.decoded_span().unwrap(),
                    comment.authored_span().unwrap(),
                    comment.text().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            comments,
            [
                (
                    CommentKind::SingleLineBlock,
                    Span::new(0, 7),
                    Span::new(26, 33),
                    "/*雪*/"
                ),
                (
                    CommentKind::Line,
                    Span::new(16 + shift, 21 + shift),
                    Span::new(50 + shift, 55 + shift),
                    "//尾"
                ),
            ]
        );
        let Expression::ParenthesizedExpression(parentheses) = syntax.expression().unwrap() else {
            panic!("original model parentheses");
        };
        assert_eq!(
            syntax.authored_span(parentheses.span),
            Ok(Span::new(34 + shift, 50 + shift))
        );
        let Expression::LogicalExpression(logical) = &parentheses.expression else {
            panic!("original once-decoded expression");
        };
        assert_eq!(logical.operator.as_str(), "&&");
        for (child, name, authored) in [
            (&logical.left, "α", Span::new(35 + shift, 37 + shift)),
            (&logical.right, "β", Span::new(47 + shift, 49 + shift)),
        ] {
            let Expression::Identifier(identifier) = child else {
                panic!("actual Unicode child");
            };
            assert_eq!(identifier.name.as_str(), name);
            assert_eq!(syntax.authored_span(identifier.span), Ok(authored));
        }
        assert_original(&operand, &owner, 0);
    }
}

#[test]
fn sibling_ordinal_same_buffer_and_equal_byte_owners_cannot_join_the_original_model() {
    let arena = Allocator::default();
    let source =
        "<template><input title='draft' v-model='draft'/><input v-model='draft'/></template>";
    let owner = selected(&arena, source);
    let operand = observe(&owner, 1);
    assert_original(&operand, &owner, 1);
    let first = owner.children().next().unwrap().into_element().unwrap();
    assert!(
        operand
            .admitted_for(&owner, first.attributes().next().unwrap())
            .is_none()
    );
    let sibling = owner.children().nth(1).unwrap().into_element().unwrap();
    assert!(
        operand
            .admitted_for(&owner, sibling.attributes().next().unwrap())
            .is_none()
    );
    let independently_parsed = selected(&arena, source);
    let copy = vize_l0::String::from(source);
    let copied = selected(&arena, copy.as_str());
    assert!(!core::ptr::eq(copy.as_str(), source));
    for foreign in [&independently_parsed, &copied] {
        let element = foreign.children().next().unwrap().into_element().unwrap();
        let attribute = element.attributes().nth(1).unwrap();
        assert!(
            operand
                .admitted_for(foreign, attribute.reborrow())
                .is_none()
        );
        assert!(operand.admitted_for(&owner, attribute.reborrow()).is_none());
        let before = arena.allocated_bytes();
        assert_eq!(
            owner.observe_attribute_head(attribute).err(),
            Some(NativeAttributeOperandError::ForeignComponent)
        );
        assert_eq!(arena.allocated_bytes(), before);
    }
    assert_original(&operand, &owner, 1);
}

#[test]
fn parked_movement_and_owner_drop_preserve_original_ast_and_syntax_transfer() {
    let arena = Allocator::default();
    let source = "<template><input title=x v-model='/*原*/ draft'/></template>";
    let (syntax, original) = {
        let owner = selected(&arena, source);
        let operand = observe(&owner, 1);
        let original = operand.syntax().expression().unwrap();
        let mut parked = alloc::vec![operand];
        parked.reserve(32);
        let moved = core::hint::black_box(owner);
        assert_original(parked.first().unwrap(), &moved, 1);
        let operand = parked.pop().unwrap();
        assert!(core::ptr::eq(
            operand.syntax().expression().unwrap(),
            original
        ));
        (operand.into_syntax(), original)
    };
    assert!(core::ptr::eq(syntax.expression().unwrap(), original));
    assert!(core::ptr::eq(syntax.source().authored_root(), source));
    assert_eq!(syntax.source().text(), "/*原*/ draft");
    assert_eq!(syntax.source().span(), value_span(source, "/*原*/ draft"));
    assert_eq!(syntax.comments().count(), 1);
    assert_eq!(syntax.diagnostics().count(), 0);
    assert_eq!(syntax.hole(), None);
    assert!(syntax.admitted_expression().is_some());
    assert!(core::mem::needs_drop::<
        NativeAttributeModelExpression<'static>,
    >());
}

#[test]
fn model_arguments_and_unsupported_modifiers_refuse_before_any_value_preparation() {
    for name in [
        "v-model:value",
        "v-model:[name]",
        "v-model.lazy",
        "v-model.number",
        "v-model.trim.trim",
        "v-model.trim.lazy",
        "v-model.lazy.trim",
        "v-model.Trim",
        "v-model.",
        "v-model:value.trim",
    ] {
        let arena = Allocator::default();
        let source = alloc::format!("<template><input {name}='broken &amp;'/></template>");
        let owner = selected(&arena, &source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let attribute = element.attributes().next().unwrap();
        assert_eq!(attribute.surface().name.text, name);
        let before = arena.allocated_bytes();
        let head = owner.observe_attribute_head(attribute).unwrap();
        assert_eq!(
            head.model().err(),
            Some(NativeAttributeOperandError::UnsupportedDirective)
        );
        assert_eq!(arena.allocated_bytes(), before);
        assert_eq!(
            head.attribute()
                .surface()
                .value
                .as_ref()
                .unwrap()
                .content
                .text,
            "broken &amp;"
        );
    }
}

#[test]
fn other_families_and_old_conditional_entry_keep_their_separate_contracts() {
    for name in [
        "title",
        ":id",
        "v-bind:id",
        "@input",
        "v-on:input",
        "v-if",
        "v-show",
        "v-models",
        "v-custom",
        "#default",
    ] {
        let arena = Allocator::default();
        let source = alloc::format!("<template><input {name}='draft'/></template>");
        let owner = selected(&arena, &source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let before = arena.allocated_bytes();
        let head = owner
            .observe_attribute_head(element.attributes().next().unwrap())
            .unwrap();
        assert!(head.model().unwrap().is_none());
        assert_eq!(arena.allocated_bytes(), before);
    }
    let arena = Allocator::default();
    let source = "<template><input v-model='draft'/></template>";
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let head = owner
        .observe_attribute_head(element.attributes().next().unwrap())
        .unwrap();
    assert_eq!(head.condition_kind(), None);
    assert!(head.static_binding().unwrap().is_none());
    let before = arena.allocated_bytes();
    let failure = head.observe_expression().err().unwrap();
    assert_eq!(
        failure.kind(),
        NativeAttributeOperandError::UnsupportedDirective
    );
    assert!(failure.syntax().is_none());
    assert_eq!(arena.allocated_bytes(), before);
}

#[test]
fn missing_value_and_parse_holes_keep_actual_failure_and_complete_stock_diagnostics() {
    let arena = Allocator::default();
    let owner = selected(&arena, "<template><input v-model/></template>");
    let element = owner.children().next().unwrap().into_element().unwrap();
    let head = owner
        .observe_attribute_head(element.attributes().next().unwrap())
        .unwrap();
    let before = arena.allocated_bytes();
    let failure = head
        .model()
        .unwrap()
        .unwrap()
        .observe_expression()
        .err()
        .unwrap();
    assert_eq!(failure.kind(), NativeAttributeOperandError::IncompleteValue);
    assert!(failure.syntax().is_none());
    assert_eq!(arena.allocated_bytes(), before);
    for (raw, message, end) in [
        ("", "Empty parenthesized expression", 0),
        ("/*kept*/ draft +", "Unexpected token", 16),
    ] {
        let source = alloc::format!("<template><input v-model='{raw}'/></template>");
        let owner = selected(&arena, &source);
        let operand = observe(&owner, 0);
        let syntax = operand.syntax();
        assert_eq!(operand.raw_value(), raw);
        assert_eq!(operand.value_span(), value_span(&source, raw));
        assert_eq!(syntax.source().text(), raw);
        assert_eq!(syntax.hole(), Some(EmbedHole::Syntax));
        assert!(syntax.expression().is_none());
        assert!(syntax.admitted_expression().is_none());
        let rows: Vec<_> = syntax.diagnostics().collect();
        let [row] = rows.as_slice() else {
            panic!("one complete stock diagnostic");
        };
        assert_eq!(row.message(), message);
        assert_eq!(row.severity(), oxc_diagnostics::Severity::Error);
        assert_eq!(row.code(), &oxc_diagnostics::OxcCode::default());
        assert_eq!((row.help(), row.note(), row.url()), (None, None, None));
        let labels: Vec<_> = row
            .labels()
            .map(|label| {
                (
                    label.decoded_span().unwrap(),
                    label.authored_span().unwrap(),
                    label.message(),
                    label.primary(),
                )
            })
            .collect();
        let boundary = operand.value_span().end;
        assert_eq!(
            labels,
            [(
                Span::new(end, end),
                Span::new(boundary, boundary),
                None,
                false
            )]
        );
        assert_eq!(syntax.comments().count(), usize::from(!raw.is_empty()));
        let element = owner.children().next().unwrap().into_element().unwrap();
        assert!(
            operand
                .admitted_for(&owner, element.attributes().next().unwrap())
                .is_none()
        );
    }
}

#[test]
fn unquoted_value_and_original_recovery_verbatim_boundaries_remain_truthful() {
    let arena = Allocator::default();
    let source = "<template><input v-model=draft /></template>";
    let owner = selected(&arena, source);
    let operand = observe(&owner, 0);
    assert_eq!(operand.raw_value(), "draft");
    assert_eq!(operand.value_span(), source_span(source, "draft"));
    assert_original(&operand, &owner, 0);
    for (source, ordinal, expected) in [
        (
            "<template><div v-model='draft'></template>",
            0,
            NativeAttributeOperandError::RecoveredComponent,
        ),
        (
            "<template><input v-pre v-model='draft'/></template>",
            1,
            NativeAttributeOperandError::Verbatim,
        ),
    ] {
        let owner = selected(&arena, source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let before = arena.allocated_bytes();
        assert_eq!(
            owner
                .observe_attribute_head(element.attributes().nth(ordinal).unwrap())
                .err(),
            Some(expected)
        );
        assert_eq!(arena.allocated_bytes(), before);
    }
}
