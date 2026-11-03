use super::{Allocator, NativeInterpolationInput, State, SurfaceChild, Test, check, owner, same};
use crate::expr::ExprRef;
use crate::op::Op;
use vize_l0::{Span, id::NodeId};

#[test]
fn original_root_literals_keep_whole_inputs_file_resolution_and_dense_actual_nodes() -> Test {
    let arena = Allocator::default();
    let source = "<template>pré{{ /*kept*/ '雪&amp;🌸' }}<!--tail-->{{ 42 }}</template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        for child in selected.children() {
            if matches!(child.surface(), SurfaceChild::Interpolation(_)) {
                let input = NativeInterpolationInput::from_operand(
                    selected
                        .observe_interpolation_expression(child.reborrow())
                        .map_err(|_| "once original parse")?,
                );
                walk.root_interpolation(child, input)
                    .map_err(|_| "actual interpolation event")?;
            } else {
                walk.child(child).map_err(|_| "original static child")?;
            }
        }
        walk.complete().map_err(|_| "normal root completion")?;
    }
    let output = core::hint::black_box(original.finish());
    let view = output.view().map_err(|_| "genuine completed view")?;
    let file = view.file().ok_or("retained File")?;
    check(core::ptr::eq(view.owner(), &output))?;
    check(core::ptr::eq(file, output.file().ok_or("owning File")?))?;
    check(core::ptr::eq(file.artifact().source(), source))?;
    check(file.is_complete())?;
    same(file.artifact().node_count(), 4)?;
    same(file.native_interpolations().len(), 2)?;
    let [
        Op::Text(text),
        Op::Interpolation(first),
        Op::Comment(comment),
        Op::Interpolation(second),
    ] = file.artifact().root().ops.as_slice()
    else {
        return Err("actual root order");
    };
    same((text.content, comment.content), ("pré", "tail"))?;
    for (index, actual, content, comment_count) in [
        (1, first, " /*kept*/ '雪&amp;🌸' ", 1),
        (3, second, " 42 ", 0),
    ] {
        let node = NodeId::from_index(index).ok_or("dense node")?;
        let record = file
            .native_interpolation(node)
            .ok_or("actual File input association")?;
        same(record.state(), State::Admitted(node))?;
        same(record.input().operand().full_span(), actual.span)?;
        same(record.input().operand().raw_content(), content)?;
        same(
            record.input().operand().syntax().comments().count(),
            comment_count,
        )?;
        same(record.input().operand().syntax().diagnostics().count(), 0)?;
        check(core::ptr::eq(
            record.input().operand().syntax().source().authored_root(),
            source,
        ))?;
        let ExprRef::Js(expression) = actual.expression else {
            return Err("actual JsExpr");
        };
        check(core::ptr::eq(
            expression.ast,
            record
                .input()
                .operand()
                .syntax()
                .expression()
                .ok_or("retained stock AST")?,
        ))?;
        let resolution = file.expression(node).ok_or("attached File resolution")?;
        check(core::ptr::eq(resolution.file(), file))?;
        same(resolution.node(), node)?;
        let table = resolution.table().ok_or("sole resolver table")?;
        // The sole resolver normally copies this small metadata wrapper.
        // Original AST/source/coordinate custody, not wrapper address, joins it.
        let resolved = table.expression();
        check(core::ptr::eq(resolved.ast, expression.ast))?;
        check(core::ptr::eq(resolved.source, expression.source))?;
        same(resolved.span, expression.span)?;
        let (Some(resolved_coordinates), Some(original_coordinates)) =
            (resolved.coordinates, expression.coordinates)
        else {
            return Err("original coordinate metadata missing");
        };
        check(core::ptr::eq(resolved_coordinates, original_coordinates))?;
        same(table.occurrences().len(), 0)?;
        same(resolution.scope().ok_or("actual scope")?.index(), 0)?;
    }
    check(file.native_interpolation(NodeId::FIRST).is_none())?;
    let first = file
        .native_interpolations()
        .first()
        .ok_or("first retained operand")?;
    same(
        first.input().operand().full_span().slice(source),
        "{{ /*kept*/ '雪&amp;🌸' }}",
    )?;
    // Independent whole-source extent remains byte-exact after File movement.
    same(first.input().operand().full_span(), Span::new(14, 43))
}
