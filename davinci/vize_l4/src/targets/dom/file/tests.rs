//! Genuine factories exercise the private splice preflight and equal-id owners.

#[path = "../../../../tests/native_dom_file/support.rs"]
mod support;

use super::{ExpressionWriter, FileExpressions};
use crate::expr::EmitErrorKind;
use crate::runtime::{Runtime, vocabulary};
use crate::targets::dom::DomErrorKind;
use crate::write::{Recorded, Writer};
use vize_l0::{Allocator, Span};
use vize_l2::{
    expr::ExprRef,
    file::TemplateScope,
    lang::js::{FileProducer, ProgramInput, ProgramScope},
};
use vize_l3::decision::build_dom_file_decisions;

#[test]
fn equal_numeric_nodes_do_not_authorize_a_foreign_retained_literal() {
    let arena = Allocator::default();
    let source = "<p>{{'same'}}</p>";
    let mut first = FileProducer::new(&arena, source).unwrap();
    let first_native = support::construct(&arena, source, source, &mut first, TemplateScope::Root);
    let first = first.finish().unwrap();
    let mut second = FileProducer::new(&arena, source).unwrap();
    let second_native =
        support::construct(&arena, source, source, &mut second, TemplateScope::Root);
    let second = second.finish().unwrap();
    let first_analysis = build_dom_file_decisions(&first).unwrap();
    let second_analysis = build_dom_file_decisions(&second).unwrap();
    let node = first_native.embeds.first().unwrap().node.unwrap();
    assert_eq!(second_native.embeds.first().unwrap().node, Some(node));
    let original = first_analysis
        .dom()
        .unwrap()
        .file_expression(node)
        .unwrap()
        .resolution();
    let foreign = second_analysis
        .dom()
        .unwrap()
        .file_expression(node)
        .unwrap()
        .resolution();
    assert!(!core::ptr::eq(original.file(), foreign.file()));
    let original_expression = original.table().unwrap().expression();
    let foreign_expression = foreign.table().unwrap().expression();
    assert_eq!(original_expression.span, foreign_expression.span);
    assert_eq!(original_expression.source, foreign_expression.source);
    assert!(!core::ptr::eq(
        original_expression.ast,
        foreign_expression.ast
    ));
    let helper = vocabulary(Runtime::VueDom).helper("openBlock").unwrap();
    let mut writer = Writer::<Recorded>::default();
    writer.push_linked("prefix", Span::new(0, 3));
    writer.indent();
    writer.use_helper(helper);
    let error = FileExpressions {
        file: first_analysis.file(),
        facts: first_analysis.dom(),
    }
    .write(&mut writer, node, ExprRef::Js(foreign_expression))
    .unwrap_err();
    let DomErrorKind::Expression(error) = error.kind else {
        panic!("foreign AST identity must be checked before spelling");
    };
    assert_eq!(error.kind, EmitErrorKind::SourceMismatch);
    writer.newline();
    writer.push("suffix");
    let emitted = writer.finish();
    assert_eq!(emitted.text.as_str(), "prefix\n  suffix");
    assert_eq!(emitted.helpers.in_use_order(), &[helper]);
    let document = emitted.into_document();
    assert_eq!(document.links().len(), 1);
    assert_eq!(document.links().first().unwrap().authored, Span::new(0, 3));
}

#[test]
fn equal_binding_ids_keep_real_owners_and_reference_refusal_is_atomic() {
    let arena = Allocator::default();
    let source = "<script setup>const msg = 'literal';</script><template>{{msg}}</template>";
    let (syntax, block) = support::script(&arena, source, "const msg = 'literal';");
    let mut first = FileProducer::new(&arena, source).unwrap();
    first
        .program(
            ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap(),
            ProgramScope::Nested,
        )
        .unwrap();
    let first_native = support::construct(
        &arena,
        source,
        "{{msg}}",
        &mut first,
        TemplateScope::LastUnit,
    );
    let first = first.finish().unwrap();
    let mut second = FileProducer::new(&arena, source).unwrap();
    second
        .program(
            ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap(),
            ProgramScope::Nested,
        )
        .unwrap();
    let second_native = support::construct(
        &arena,
        source,
        "{{msg}}",
        &mut second,
        TemplateScope::LastUnit,
    );
    let second = second.finish().unwrap();
    let first_analysis = build_dom_file_decisions(&first).unwrap();
    let second_analysis = build_dom_file_decisions(&second).unwrap();
    let node = first_native.embeds.first().unwrap().node.unwrap();
    assert_eq!(second_native.embeds.first().unwrap().node, Some(node));
    let original = first_analysis
        .dom()
        .unwrap()
        .file_expression(node)
        .unwrap()
        .resolution();
    let foreign = second_analysis
        .dom()
        .unwrap()
        .file_expression(node)
        .unwrap()
        .resolution();
    let first_use = original.table().unwrap().occurrences().first().unwrap();
    let second_use = foreign.table().unwrap().occurrences().first().unwrap();
    assert_eq!(first_use.binding, second_use.binding);
    let first_binding = original.binding(first_use.binding).unwrap();
    let second_binding = foreign.binding(second_use.binding).unwrap();
    assert!(!first_binding.same_owner(second_binding));
    assert!(!original.accepts(second_binding));
    let expression = original.table().unwrap().expression();
    let mut writer = Writer::<Recorded>::default();
    writer.push("prefix");
    let error = FileExpressions {
        file: first_analysis.file(),
        facts: first_analysis.dom(),
    }
    .write(&mut writer, node, ExprRef::Js(expression))
    .unwrap_err();
    assert_eq!(error.kind, DomErrorKind::RuntimeAccessUnavailable);
    assert_eq!(error.span, expression.span);
    assert_eq!(writer.as_str(), "prefix");
    let emitted = writer.finish();
    assert!(emitted.helpers.in_use_order().is_empty());
    assert!(emitted.into_document().links().is_empty());
}
