use super::super::{Allocator, NoLinks, Recorded, check, owner};
use vize_l0::id::NodeId;
use vize_l2::lang::js::NativeInterpolationInput;
use vize_l3::decision::{dom::DomUnsupported, native::build_native_dom_file_decisions};
use vize_l4::targets::dom::{DomErrorKind, emit_template};

#[test]
fn original_selected_legacy_and_regexp_refusals_return_no_partial_writer()
-> Result<(), &'static str> {
    for source in [
        "<template>{{ 010 }}</template>",
        "<template>{{ 08 }}</template>",
        "<template>{{ 09.5 }}</template>",
        "<template>{{ 08e1 }}</template>",
        r"<template>{{ '\1' }}</template>",
        r"<template>{{ '\8' }}</template>",
        r"<template>{{ '\9' }}</template>",
        r"<template>{{ '\00' }}</template>",
        "<template>{{ &#48;10 }}</template>",
        "<template>{{ '&#92;8' }}</template>",
        "<template>{{ /valid/u }}</template>",
        "<template>{{ /(/ }}</template>",
    ] {
        let arena = Allocator::default();
        let mut original = owner(&arena, source)?;
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
                .map_err(|_| "normal original File receiver")?;
            walk.complete().map_err(|_| "normal root end")?;
        }
        let output = core::hint::black_box(original.finish());
        let analysis = build_native_dom_file_decisions(output.view().map_err(|_| "native view")?)
            .map_err(|_| "existing sole L3 visit")?;
        let file = analysis.file();
        check(file.is_complete())?;
        check(core::ptr::eq(file.artifact().source(), source))?;
        let operand = file
            .native_interpolation(NodeId::FIRST)
            .ok_or("normal whole original input")?
            .input()
            .operand();
        let refusal = emit_template::<Recorded>(&analysis)
            .err()
            .ok_or("no Recorded writer")?;
        check(refusal.node == Some(NodeId::FIRST))?;
        check(refusal.span == operand.full_span())?;
        check(refusal.kind == DomErrorKind::Unsupported(DomUnsupported::Expression))?;
        let unrecorded = emit_template::<NoLinks>(&analysis)
            .err()
            .ok_or("no NoLinks writer")?;
        check(unrecorded == refusal)?;
        check(core::ptr::eq(analysis.owner(), &output))?;
        check(operand.syntax().admitted_expression().is_some())?;
    }
    Ok(())
}
