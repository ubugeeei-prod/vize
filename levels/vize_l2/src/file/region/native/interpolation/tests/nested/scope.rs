use super::{Allocator, State, Test, check, complete, joined, owner, same};
use crate::file::{FileArtifact, ScopeId};
use crate::op::Op;
use crate::resolution::BindingId;

fn reference(file: &FileArtifact<'_>, index: usize) -> Result<(ScopeId, BindingId), &'static str> {
    let record = file
        .native_interpolations()
        .get(index)
        .ok_or("original row")?;
    let State::Admitted(node) = record.state() else {
        return Err("admitted original expression node");
    };
    let resolution = file.expression(node).ok_or("same File expression")?;
    let [occurrence] = resolution
        .table()
        .ok_or("sole resolver table")?
        .occurrences()
    else {
        return Err("one actual original reference");
    };
    Ok((
        resolution.scope().ok_or("actual current scope")?,
        occurrence.binding,
    ))
}

#[test]
fn nested_setup_and_for_shadowing_use_actual_scope_and_original_declaration_rows() -> Test {
    let arena = Allocator::default();
    let source = "<script setup>const value=2;const items=3;</script><template><div>{{ value }}<span v-for='(value,key) in items'>{{ value }}{{ key }}<b>{{ value }}</b></span>{{ value }}</div></template>";
    let mut original = owner(&arena, source)?;
    let unit = original
        .parse_setup_program()
        .map_err(|_| "original owning Program")?;
    complete(&mut original)?;
    let output = core::hint::black_box(original.finish());
    let setup = output.setup().map_err(|_| "same owning setup receipt")?;
    same(setup.unit(), unit)?;
    let file = setup.file();
    same(file.native_interpolations().len(), 5)?;
    let [Op::Element(div)] = file.artifact().root().ops.as_slice() else {
        return Err("original div");
    };
    let [
        Op::Interpolation(first),
        Op::OriginalFor(original_for),
        Op::Interpolation(last),
    ] = div.children.ops.as_slice()
    else {
        return Err("one original ordered For introduction");
    };
    let [Op::Element(span)] = original_for.region.ops.as_slice() else {
        return Err("same repeated original element");
    };
    let [
        Op::Interpolation(value),
        Op::Interpolation(key),
        Op::Element(b),
    ] = span.children.ops.as_slice()
    else {
        return Err("actual alias-scope original children");
    };
    let [Op::Interpolation(inner)] = b.children.ops.as_slice() else {
        return Err("descendant shares genuine current For scope");
    };
    for (index, actual) in [first, value, key, inner, last].into_iter().enumerate() {
        joined(file, index, actual)?;
    }
    let head = file
        .for_head_for(original_for)
        .ok_or("actual For allocation association")?;
    let alias = head.value().ok_or("genuine original value declaration")?;
    let key_alias = head.key().ok_or("genuine original key declaration")?;
    let (outer_scope, setup_binding) = reference(file, 0)?;
    same(Some(outer_scope), head.enclosing_scope())?;
    same(reference(file, 4)?, (outer_scope, setup_binding))?;
    let alias_scope = head.scope().ok_or("genuine introduced child scope")?;
    check(alias_scope != outer_scope)?;
    same(reference(file, 1)?, (alias_scope, alias.id()))?;
    same(reference(file, 2)?, (alias_scope, key_alias.id()))?;
    same(reference(file, 3)?, (alias_scope, alias.id()))?;
    check(alias.id() != setup_binding)?;
    check(alias.declaration().is_none())?;
    let template = alias
        .template_declaration()
        .ok_or("original template declaration")?;
    same(template.declaration().scope(), alias_scope)?;
    same(template.declaration().origin(), original_for.id())?;
    let declaration = file.binding(setup_binding).ok_or("actual setup binding")?;
    same(
        declaration
            .declaration()
            .ok_or("original script declaration")?
            .unit,
        unit,
    )?;
    check(core::ptr::eq(
        setup.syntax(),
        output.retained_setup().ok_or("whole Program")?,
    ))?;
    Ok(())
}
