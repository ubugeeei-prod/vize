use super::super::{Allocator, NodeId, Op, check, owner};
use vize_l2::{
    expr::ExprRef, file::NativeFileInterpolationState, lang::js::NativeInterpolationInput,
};
use vize_l3::decision::{
    dom::{DomChild, DomDependency, DomRootKind},
    native::build_native_dom_file_decisions,
};

#[test]
fn original_root_literal_retains_file_input_and_resolution_in_the_existing_sole_l3_walk()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template>{{ /*kept*/ '雪&amp;🌸' }}</template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        let child = selected.children().next().ok_or("original child")?;
        let input = NativeInterpolationInput::from_operand(
            selected
                .observe_interpolation_expression(child.reborrow())
                .map_err(|_| "original observation")?,
        );
        check(
            walk.root_interpolation(child, input)
                .map_err(|_| "actual File event")?
                == NodeId::FIRST,
        )?;
        walk.complete().map_err(|_| "normal original root end")?;
    }
    let output = core::hint::black_box(original.finish());
    let analysis =
        build_native_dom_file_decisions(output.view().map_err(|_| "genuine native view")?)
            .map_err(|_| "existing sole L3 walk")?;
    check(core::ptr::eq(analysis.owner(), &output))?;
    let file = analysis.file();
    check(core::ptr::eq(file, output.file().ok_or("same File")?))?;
    check(core::ptr::eq(analysis.artifact().source(), source))?;
    check(analysis.artifact().node_count() == 1)?;
    let [Op::Interpolation(op)] = file.artifact().root().ops.as_slice() else {
        return Err("actual interpolation Op");
    };
    let input = file
        .native_interpolation(NodeId::FIRST)
        .ok_or("actual File-owned input")?;
    check(input.state() == NativeFileInterpolationState::Admitted(NodeId::FIRST))?;
    check(input.input().operand().full_span() == op.span)?;
    check(input.input().operand().syntax().comments().count() == 1)?;
    check(input.input().operand().syntax().diagnostics().count() == 0)?;
    let ExprRef::Js(expression) = op.expression else {
        return Err("actual retained expression");
    };
    check(core::ptr::eq(
        expression.ast,
        input
            .input()
            .operand()
            .syntax()
            .expression()
            .ok_or("original stock AST")?,
    ))?;
    let row = analysis
        .expression(NodeId::FIRST)
        .ok_or("same-walk original File expression")?;
    let resolution = row.resolution();
    check(core::ptr::eq(resolution.file(), file))?;
    check(resolution.node() == NodeId::FIRST)?;
    let table = resolution.table().ok_or("original resolver table")?;
    // Normal resolution copies only the metadata wrapper, retaining all
    // authentic arena/source identities and the original authored extent.
    let resolved = table.expression();
    check(core::ptr::eq(resolved.ast, expression.ast))?;
    check(core::ptr::eq(resolved.source, expression.source))?;
    check(resolved.span == expression.span)?;
    let (Some(resolved_coordinates), Some(original_coordinates)) =
        (resolved.coordinates, expression.coordinates)
    else {
        return Err("original coordinate metadata missing");
    };
    check(core::ptr::eq(resolved_coordinates, original_coordinates))?;
    check(table.occurrences().is_empty())?;
    let dom = analysis.dom().ok_or("same-walk DOM facts")?;
    check(dom.unsupported().is_empty())?;
    check(dom.root().kind == DomRootKind::Direct)?;
    let [DomChild::Text(text)] = dom.root().children.as_slice() else {
        return Err("original text group");
    };
    check(text.nodes.as_slice() == [NodeId::FIRST])?;
    check(!text.dynamic)?;
    check(dom.dependencies() == [DomDependency::DisplayValue])?;
    check(core::ptr::eq(
        dom.node(NodeId::FIRST).ok_or("same Op fact")?.op(),
        &file.artifact().root().ops[0],
    ))
}
