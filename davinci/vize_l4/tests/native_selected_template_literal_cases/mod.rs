//! Read-only laws join actual root events, normally owned inputs and emitted ranges.

use super::{check, equal};
use vize_l0::{Span, id::NodeId};
use vize_l1::SurfaceChild;
use vize_l2::{
    expr::ExprRef, file::NativeFileInterpolationState, lang::js::NativeTemplateFile, op::Op,
};
use vize_l3::decision::native::NativeTemplateDomAnalysis;
use vize_l4::write::EmitDocument;

pub(super) fn inspect<'owner, 'a>(
    id: &str,
    original: &'owner NativeTemplateFile<'a>,
    analysis: &NativeTemplateDomAnalysis<'owner, 'a>,
    document: &EmitDocument,
    nodes: u32,
) -> Result<(), &'static str> {
    let file = original.file().ok_or("original File")?;
    check(file.is_complete())?;
    check(core::ptr::eq(analysis.owner(), original))?;
    check(core::ptr::eq(analysis.file(), file))?;
    check(core::ptr::eq(analysis.artifact(), file.artifact()))?;
    check(core::ptr::eq(
        file.artifact().source(),
        original.selected().component().block().root_source(),
    ))?;
    equal(analysis.artifact().node_count(), nodes)?;
    equal(analysis.tables().nodes.len(), nodes as usize)?;
    check(analysis.tables().controls.is_empty())?;
    let dom = analysis.dom().ok_or("same-walk DOM facts")?;
    check(dom.unsupported().is_empty())?;
    let selected = original.selected();
    equal(selected.children().len(), file.artifact().root().ops.len())?;
    let mut interpolations = 0;
    for (ordinal, (child, op)) in selected
        .children()
        .zip(&file.artifact().root().ops)
        .enumerate()
    {
        equal(child.ordinal(), ordinal)?;
        check(child.parent_element().is_none())?;
        check(core::ptr::eq(child.component(), selected.component()))?;
        let node = NodeId::from_index(ordinal as u32).ok_or("real root node")?;
        check(core::ptr::eq(
            dom.node(node).ok_or("actual node fact")?.op(),
            op,
        ))?;
        let row = analysis
            .tables()
            .nodes
            .get(node)
            .ok_or("original node row")?;
        check(row.control.is_none() && row.dynamic_bindings.is_empty())?;
        match (child.surface(), op) {
            (SurfaceChild::Interpolation(_), Op::Interpolation(op)) => {
                interpolations += 1;
                let record = file
                    .native_interpolation(node)
                    .ok_or("normal original input association")?;
                equal(record.state(), NativeFileInterpolationState::Admitted(node))?;
                let operand = record.input().operand();
                check(operand.admitted_for(selected, child.reborrow()).is_some())?;
                equal(operand.full_span(), op.span)?;
                equal(
                    operand.syntax().comments().count(),
                    usize::from(id == "comment-literal"),
                )?;
                equal(operand.syntax().diagnostics().count(), 0)?;
                let receipt = operand
                    .syntax()
                    .admitted_expression()
                    .ok_or("original parser receipt")?;
                check(!receipt.has_legacy_literals())?;
                let ExprRef::Js(expression) = op.expression else {
                    return Err("retained actual JsExpr");
                };
                check(core::ptr::eq(receipt.expression(), expression.ast))?;
                let source = operand.syntax().source();
                check(core::ptr::eq(source.text(), expression.source))?;
                check(core::ptr::eq(
                    source.authored_root(),
                    file.artifact().source(),
                ))?;
                equal(source.span(), expression.span)?;
                let resolution = analysis
                    .expression(node)
                    .ok_or("sole-visit expression fact")?
                    .resolution();
                check(core::ptr::eq(resolution.file(), file))?;
                equal(resolution.node(), node)?;
                let table = resolution.table().ok_or("original sole resolver table")?;
                check(table.occurrences().is_empty())?;
                let resolved = table.expression();
                check(core::ptr::eq(resolved.ast, expression.ast))?;
                check(core::ptr::eq(resolved.source, expression.source))?;
                equal(resolved.span, expression.span)?;
                let (Some(left), Some(right)) = (resolved.coordinates, expression.coordinates)
                else {
                    return Err("actual coordinate association");
                };
                check(core::ptr::eq(left, right))?;
                let end = u32::try_from(expression.source.len()).map_err(|_| "decoded extent")?;
                let authored = expression
                    .authored_span(Span::new(0, end))
                    .ok_or("whole decoded projection")?;
                equal(authored, expression.span)?;
                let mut actual = document
                    .links()
                    .iter()
                    .filter(|link| link.authored == authored);
                let link = actual.next().ok_or("whole emitted expression range")?;
                check(actual.next().is_none())?;
                check(link.segment && link.name.is_none())?;
                equal(
                    document
                        .as_str()
                        .get(link.generated.start as usize..link.generated.end as usize),
                    Some(expression.source),
                )?;
            }
            (SurfaceChild::Text(token), Op::Text(text)) => {
                check(core::ptr::eq(text.content, token.text))?;
                equal(
                    text.span,
                    selected
                        .component()
                        .block()
                        .span_of(token.text)
                        .ok_or("original text extent")?,
                )?;
                check(
                    file.native_interpolation(node).is_none()
                        && analysis.expression(node).is_none(),
                )?;
            }
            (SurfaceChild::Comment(token), Op::Comment(comment)) => {
                let body = token
                    .text
                    .strip_prefix("<!--")
                    .and_then(|text| text.strip_suffix("-->"))
                    .ok_or("original comment framing")?;
                check(core::ptr::eq(comment.content, body))?;
                equal(
                    comment.span,
                    selected
                        .component()
                        .block()
                        .span_of(token.text)
                        .ok_or("original comment extent")?,
                )?;
                check(
                    file.native_interpolation(node).is_none()
                        && analysis.expression(node).is_none(),
                )?;
            }
            _ => return Err("same original ordered root operation"),
        }
    }
    equal(file.native_interpolations().len(), interpolations)?;
    equal(interpolations, if id == "mixed" { 2 } else { 1 })?;
    let next = NodeId::from_index(nodes).ok_or("first absent node")?;
    check(file.native_interpolation(next).is_none() && analysis.expression(next).is_none())?;
    check(analysis.tables().nodes.get(next).is_none() && dom.node(next).is_none())?;
    check(!document.links().is_empty())?;
    for link in document.links() {
        check(
            file.artifact()
                .source()
                .get(link.authored.start as usize..link.authored.end as usize)
                .is_some(),
        )?;
        check(
            document
                .as_str()
                .get(link.generated.start as usize..link.generated.end as usize)
                .is_some(),
        )?;
        check(link.name.is_none())?;
    }
    Ok(())
}
