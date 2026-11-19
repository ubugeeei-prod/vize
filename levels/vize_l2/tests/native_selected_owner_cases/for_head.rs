use super::{Allocator, Kind, check, descriptor, equal, owner, syntax};
use vize_l2::{
    file::Namespace,
    lang::js::NativeTemplateFile,
    op::{BindingOp, Op},
    resolution::{ForAliasRole, HandlerBindingRef, Usage},
};

fn complete_setup<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateFile<'a>, &'static str> {
    let descriptor = descriptor(arena, source);
    let program = syntax(arena, &descriptor, true)?;
    let mut original = owner(arena, source)?;
    original
        .setup_program(program.admitted_program().ok_or("original setup Program")?)
        .map_err(|_| "setup entry")?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        for child in walk.selected().children() {
            walk.child(child).map_err(|_| "actual root child")?;
        }
        walk.complete().map_err(|_| "normal root completion")?;
    }
    Ok(core::hint::black_box(original.finish()))
}

#[test]
fn original_for_introduces_real_alias_rows_before_resolving_earlier_and_nested_events()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<script setup>const items=2;</script><template><div @click='item' v-for='(item,index) in items' id='x'>text<span @blur='index'/></div></template>";
    let output = complete_setup(&arena, source)?;
    let file = output
        .view()
        .map_err(|_| "native view")?
        .file()
        .ok_or("File")?;
    check(file.is_complete())?;
    equal(file.units().len(), 1)?;
    equal(file.artifact().node_count(), 6)?;
    equal(file.bindings().count(), 3)?;
    equal(file.template_declarations().count(), 2)?;
    check(file.unattached_for_heads().next().is_none())?;
    check(file.unattached_handlers().next().is_none())?;
    check(file.rejected_for_heads().is_empty())?;
    let [Op::OriginalFor(original_for)] = file.artifact().root().ops.as_slice() else {
        return Err("one introducing original For");
    };
    let joined = file
        .for_head_for(original_for)
        .ok_or("exact For allocation")?;
    check(core::ptr::eq(joined.file(), file))?;
    check(joined.accepts(original_for))?;
    let facts = joined.resolution().ok_or("whole original head facts")?;
    let scope = joined.scope().ok_or("real alias scope")?;
    let enclosing = joined.enclosing_scope().ok_or("enclosing setup scope")?;
    check(scope != enclosing)?;
    equal(
        file.scopes()
            .get(scope.index() as usize)
            .ok_or("child scope row")?
            .parent,
        Some(enclosing),
    )?;
    equal(file.units()[0].scope, enclosing)?;
    let script = file
        .lookup(enclosing, "items", Namespace::Value)
        .ok_or("actual setup binding")?;
    check(script.declaration().is_some())?;
    check(script.template_declaration().is_none())?;
    equal(facts.collection().binding, script.id())?;
    equal(facts.collection().usage, Usage::Read)?;
    equal(facts.collection_authored_span().slice(source), "items")?;
    let value = joined.value().ok_or("real value binding")?;
    let key = joined.key().ok_or("real key binding")?;
    equal(value.id().index(), u32::MAX)?;
    equal(key.id().index(), u32::MAX - 1)?;
    check(value.id() != script.id() && key.id() != script.id())?;
    check(value.declaration().is_none() && key.declaration().is_none())?;
    equal(
        file.lookup(scope, "item", Namespace::Value)
            .ok_or("value lookup")?
            .id(),
        value.id(),
    )?;
    equal(
        file.lookup(scope, "index", Namespace::Value)
            .ok_or("key lookup")?
            .id(),
        key.id(),
    )?;
    check(file.lookup(enclosing, "item", Namespace::Value).is_none())?;
    check(file.lookup(enclosing, "index", Namespace::Value).is_none())?;
    let value_row = value.template_declaration().ok_or("template value row")?;
    let key_row = key.template_declaration().ok_or("template key row")?;
    check(value_row.same_owner(&key_row))?;
    check(core::ptr::eq(value_row.file(), file))?;
    for (row, parameter, role) in [
        (&value_row, &facts.input().aliases()[0], ForAliasRole::Value),
        (&key_row, &facts.input().aliases()[1], ForAliasRole::Key),
    ] {
        equal(row.declaration().origin(), original_for.id())?;
        equal(row.declaration().scope(), scope)?;
        let declaration = row.declaration().original().ok_or("original declaration")?;
        check(core::ptr::eq(declaration.resolution(), facts))?;
        check(core::ptr::eq(declaration.parameter(), parameter))?;
        check(core::ptr::eq(declaration.pattern(), &parameter.pattern))?;
        equal(declaration.fact().role(), role)?;
    }
    let [Op::Element(div)] = original_for.region.ops.as_slice() else {
        return Err("owned original Element");
    };
    equal(div.attributes.len(), 1)?;
    equal(div.attributes[0].name, "id")?;
    equal(div.attributes[0].value, Some("x"))?;
    let [BindingOp::On(click)] = div.bindings.as_slice() else {
        return Err("one original earlier click");
    };
    let [Op::Text(text), Op::Element(span)] = div.children.ops.as_slice() else {
        return Err("one original body walk");
    };
    equal(text.content, "text")?;
    let [BindingOp::On(blur)] = span.bindings.as_slice() else {
        return Err("one nested blur");
    };
    for (on, binding, body) in [(click, value, "item"), (blur, key, "index")] {
        let handler = file.handler_for(on).ok_or("same File and actual On")?;
        equal(handler.scope(), Some(scope))?;
        let resolution = handler.resolution().ok_or("whole original handler")?;
        equal(resolution.input().operand().raw_value(), body)?;
        equal(resolution.references().len(), 1)?;
        equal(
            resolution.references()[0].binding,
            HandlerBindingRef::Outer(binding.id()),
        )?;
        check(core::ptr::eq(
            resolution.input().body(),
            resolution
                .input()
                .operand()
                .syntax()
                .body()
                .ok_or("stock body")?,
        ))?;
    }
    let element = output
        .selected()
        .children()
        .next()
        .ok_or("original root")?
        .into_element()
        .ok_or("original Element")?;
    equal(element.attributes().len(), 3)?;
    let token = element.attributes().nth(1).ok_or("original For token")?;
    let admitted = facts
        .input()
        .admitted_for(output.selected(), token)
        .ok_or("original token join")?;
    let head = admitted
        .for_head()
        .ok_or("original two stock observations")?;
    check(core::ptr::eq(
        head.aliases().parameters().items.as_slice(),
        facts.input().aliases(),
    ))?;
    check(core::ptr::eq(
        head.collection().expression(),
        facts.input().collection(),
    ))?;
    Ok(())
}

#[test]
fn nested_original_for_resolves_parent_alias_before_introducing_its_own_child_scope()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<script setup>const items=2;</script><template><div v-for='item in items'><span v-for='child in item' @click='child;item'/></div></template>";
    let output = complete_setup(&arena, source)?;
    let file = output
        .view()
        .map_err(|_| "native view")?
        .file()
        .ok_or("File")?;
    let [Op::OriginalFor(outer)] = file.artifact().root().ops.as_slice() else {
        return Err("outer For");
    };
    let [Op::Element(div)] = outer.region.ops.as_slice() else {
        return Err("outer Element");
    };
    let [Op::OriginalFor(inner_original)] = div.children.ops.as_slice() else {
        return Err("inner original For");
    };
    let outer = file.for_head_for(outer).ok_or("outer exact allocation")?;
    let inner = file
        .for_head_for(inner_original)
        .ok_or("inner exact allocation")?;
    equal(inner.enclosing_scope(), outer.scope())?;
    let child_scope = inner.scope().ok_or("child scope")?;
    equal(
        file.scopes()
            .get(child_scope.index() as usize)
            .ok_or("scope row")?
            .parent,
        outer.scope(),
    )?;
    let parent_alias = outer.value().ok_or("parent original alias")?;
    let child_alias = inner.value().ok_or("child original alias")?;
    equal(
        inner
            .resolution()
            .ok_or("inner head facts")?
            .collection()
            .binding,
        parent_alias.id(),
    )?;
    check(parent_alias.id() != child_alias.id())?;
    let [Op::Element(span)] = inner_original.region.ops.as_slice() else {
        return Err("inner Element");
    };
    let [BindingOp::On(on)] = span.bindings.as_slice() else {
        return Err("inner handler");
    };
    let handler = file.handler_for(on).ok_or("actual inner handler")?;
    equal(handler.scope(), Some(child_scope))?;
    let references = handler.resolution().ok_or("inner references")?.references();
    equal(references.len(), 2)?;
    equal(
        references[0].binding,
        HandlerBindingRef::Outer(child_alias.id()),
    )?;
    equal(
        references[1].binding,
        HandlerBindingRef::Outer(parent_alias.id()),
    )?;
    equal(file.units().len(), 1)?;
    equal(file.template_declarations().count(), 2)?;
    equal(file.bindings().count(), 3)?;
    equal(file.artifact().node_count(), 5)?;
    check(
        file.lookup(
            outer.scope().ok_or("outer scope")?,
            "child",
            Namespace::Value,
        )
        .is_none(),
    )?;
    Ok(())
}

mod identity;
mod refusal;
mod source;
