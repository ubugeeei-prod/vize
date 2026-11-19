use super::{Allocator, Kind, State, Test, check, owner, same};
use crate::expr::ExprRef;
use crate::file::FileArtifact;
use crate::lang::js::NativeTemplateOwner;
use crate::op::{InterpolationOp, Op};

mod interruption;
mod refusal;
mod scope;

fn complete(original: &mut NativeTemplateOwner<'_>) -> Test {
    let mut walk = original.begin().map_err(|_| "begin")?;
    for child in walk.selected().children() {
        walk.child(child)
            .map_err(|_| "actual original root child")?;
    }
    walk.complete().map_err(|_| "complete root")
}

fn joined(file: &FileArtifact<'_>, index: usize, actual: &InterpolationOp<'_>) -> Test {
    let record = file
        .native_interpolations()
        .get(index)
        .ok_or("whole original input")?;
    let State::Admitted(node) = record.state() else {
        return Err("normally admitted actual node");
    };
    check(core::ptr::eq(
        record,
        file.native_interpolation(node).ok_or("actual node join")?,
    ))?;
    same(actual.span, record.input().operand().full_span())?;
    let ExprRef::Js(expression) = actual.expression else {
        return Err("original expression");
    };
    check(core::ptr::eq(
        expression.ast,
        record
            .input()
            .operand()
            .syntax()
            .expression()
            .ok_or("original stock root")?,
    ))?;
    let table = file
        .expression(node)
        .ok_or("sole File resolver")?
        .table()
        .ok_or("complete original table")?;
    check(core::ptr::eq(table.expression().ast, expression.ast))?;
    check(core::ptr::eq(table.expression().source, expression.source))?;
    check(core::ptr::eq(
        record.input().operand().syntax().source().authored_root(),
        file.artifact().source(),
    ))
}

#[test]
fn nested_original_literals_preserve_order_full_source_stock_roots_and_move_custody() -> Test {
    let arena = Allocator::default();
    let source = "<!--頭--><template><div>pré{{ /*kept*/ '雪&amp;🌸' }}<!--tail--><span>{{ 42 }}</span>{{ 7 }}</div></template>";
    let mut original = owner(&arena, source)?;
    complete(&mut original)?;
    let output = core::hint::black_box(original.finish());
    let view = output.view().map_err(|_| "completed original owner")?;
    let file = view.file().ok_or("same original File")?;
    check(file.is_complete())?;
    same(file.artifact().node_count(), 7)?;
    same(file.native_interpolations().len(), 3)?;
    check(file.native_interpolation_failures().is_empty())?;
    let [Op::Element(div)] = file.artifact().root().ops.as_slice() else {
        return Err("one original parent");
    };
    let [
        Op::Text(text),
        Op::Interpolation(first),
        Op::Comment(comment),
        Op::Element(span),
        Op::Interpolation(last),
    ] = div.children.ops.as_slice()
    else {
        return Err("single ordered original child sequence");
    };
    let [Op::Interpolation(middle)] = span.children.ops.as_slice() else {
        return Err("nested original span child");
    };
    same((text.content, comment.content), ("pré", "tail"))?;
    for (index, actual, authored, decoded, comments) in [
        (
            0,
            first,
            "{{ /*kept*/ '雪&amp;🌸' }}",
            "/*kept*/ '雪&🌸'",
            1,
        ),
        (1, middle, "{{ 42 }}", "42", 0),
        (2, last, "{{ 7 }}", "7", 0),
    ] {
        joined(file, index, actual)?;
        same(actual.span.slice(source), authored)?;
        let syntax = file.native_interpolations()[index]
            .input()
            .operand()
            .syntax();
        same(syntax.source().text(), decoded)?;
        same(syntax.comments().count(), comments)?;
        same(syntax.diagnostics().count(), 0)?;
        check(syntax.hole().is_none())?;
    }
    Ok(())
}
