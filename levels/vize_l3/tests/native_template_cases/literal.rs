use super::super::{Allocator, NodeId, Op, check, owner};
use vize_l2::{
    expr::ExprRef,
    lang::js::{NativeInterpolationInput, NativeTemplateFile},
};
use vize_l3::decision::{dom::DomUnsupported, native::build_native_dom_file_decisions};

fn completed<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateFile<'a>, &'static str> {
    let mut original = owner(arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        let child = selected.children().next().ok_or("original child")?;
        let input = NativeInterpolationInput::from_operand(
            selected
                .observe_interpolation_expression(child.reborrow())
                .map_err(|_| "original stock observation")?,
        );
        walk.root_interpolation(child, input)
            .map_err(|_| "normal private File receiver")?;
        walk.complete().map_err(|_| "normal original root end")?;
    }
    Ok(core::hint::black_box(original.finish()))
}

#[test]
fn original_primitive_receipts_join_real_file_ast_and_source_in_the_sole_l3_visit()
-> Result<(), &'static str> {
    for source in [
        "<template>{{ 42 }}</template>",
        "<template>{{ 0x2a }}</template>",
        "<template>{{ 0o52 }}</template>",
        "<template>{{ 0b101010 }}</template>",
        "<template>{{ 4.2e1 }}</template>",
        "<template>{{ 42n }}</template>",
        "<template>{{ true }}</template>",
        "<template>{{ null }}</template>",
        "<template>{{ /*kept*/ '雪&amp;🌸' }}</template>",
        r"<template>{{ '\0' }}</template>",
    ] {
        let arena = Allocator::default();
        let output = completed(&arena, source)?;
        let analysis = build_native_dom_file_decisions(output.view().map_err(|_| "view")?)
            .map_err(|_| "sole original L3 visit")?;
        check(core::ptr::eq(analysis.owner(), &output))?;
        let file = analysis.file();
        check(file.is_complete())?;
        check(core::ptr::eq(file.artifact().source(), source))?;
        let input = file
            .native_interpolation(NodeId::FIRST)
            .ok_or("normal original input association")?;
        let original = input
            .input()
            .operand()
            .syntax()
            .admitted_expression()
            .ok_or("original parser receipt")?;
        check(!original.has_legacy_literals())?;
        let [Op::Interpolation(op)] = file.artifact().root().ops.as_slice() else {
            return Err("actual original Op");
        };
        let ExprRef::Js(expression) = op.expression else {
            return Err("actual retained expression");
        };
        check(core::ptr::eq(original.expression(), expression.ast))?;
        let source = input.input().operand().syntax().source();
        check(core::ptr::eq(source.text(), expression.source))?;
        check(source.span() == expression.span)?;
        check(core::ptr::eq(
            source.authored_root(),
            file.artifact().source(),
        ))?;
        let row = analysis
            .expression(NodeId::FIRST)
            .ok_or("same-walk expression fact")?;
        check(core::ptr::eq(row.resolution().file(), file))?;
        let resolved = row
            .resolution()
            .table()
            .ok_or("actual metadata table")?
            .expression();
        let (Some(left), Some(right)) = (resolved.coordinates, expression.coordinates) else {
            return Err("actual original coordinates");
        };
        check(core::ptr::eq(left, right))?;
        check(
            row.resolution()
                .table()
                .ok_or("sole resolver table")?
                .occurrences()
                .is_empty(),
        )?;
        check(analysis.dom().ok_or("DOM")?.unsupported().is_empty())?;
    }
    Ok(())
}

#[test]
fn legacy_literals_and_regexp_retain_original_complete_file_and_full_span_refusals()
-> Result<(), &'static str> {
    for (source, legacy) in [
        ("<template>{{ 010 }}</template>", true),
        ("<template>{{ 08 }}</template>", true),
        ("<template>{{ 09.5 }}</template>", true),
        ("<template>{{ 08e1 }}</template>", true),
        (r"<template>{{ '\1' }}</template>", true),
        (r"<template>{{ '\8' }}</template>", true),
        (r"<template>{{ '\9' }}</template>", true),
        (r"<template>{{ '\00' }}</template>", true),
        ("<template>{{ &#48;10 }}</template>", true),
        ("<template>{{ '&#92;8' }}</template>", true),
        ("<template>{{ /valid/u }}</template>", false),
        ("<template>{{ /(/ }}</template>", false),
    ] {
        let arena = Allocator::default();
        let output = completed(&arena, source)?;
        let analysis = build_native_dom_file_decisions(output.view().map_err(|_| "view")?)
            .map_err(|_| "sole L3 visit")?;
        let file = analysis.file();
        check(file.is_complete())?;
        let input = file
            .native_interpolation(NodeId::FIRST)
            .ok_or("whole original input")?;
        let operand = input.input().operand();
        let syntax = operand.syntax();
        check(syntax.diagnostics().count() == 0)?;
        check(
            syntax
                .admitted_expression()
                .ok_or("stock receipt")?
                .has_legacy_literals()
                == legacy,
        )?;
        let [Op::Interpolation(op)] = file.artifact().root().ops.as_slice() else {
            return Err("actual original Op");
        };
        check(op.span == operand.full_span())?;
        let ExprRef::Js(expression) = op.expression else {
            return Err("actual retained expression");
        };
        check(core::ptr::eq(
            syntax.expression().ok_or("original AST")?,
            expression.ast,
        ))?;
        let [rejected] = analysis.dom().ok_or("DOM")?.unsupported() else {
            return Err("one typed original target refusal");
        };
        check(rejected.node == NodeId::FIRST)?;
        check(rejected.span == operand.full_span())?;
        check(rejected.reason == DomUnsupported::Expression)?;
        check(analysis.expression(NodeId::FIRST).is_none())?;
    }
    Ok(())
}
