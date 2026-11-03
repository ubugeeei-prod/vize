use super::*;
use vize_l2::file::FileIssueKind;

#[test]
fn duplicate_original_for_headers_never_mint_and_keep_every_already_observed_owner()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<script setup>const items=2;</script><template><div v-for='item in items' v-for='other in items'/></template>";
    let descriptor = descriptor(&arena, source);
    let program = syntax(&arena, &descriptor, true)?;
    let mut original = owner(&arena, source)?;
    original
        .setup_program(program.admitted_program().ok_or("setup Program")?)
        .map_err(|_| "setup")?;
    let carrier = original.selected().component().carrier();
    let carrier_refused = !carrier.errors.is_empty() || !carrier.unsupported.is_empty();
    if carrier_refused {
        // An original carrier refusal precedes observation. Do not invent a
        // retained normal head or claim its header loop was ever entered.
        equal(
            original.begin().err().ok_or("carrier refusal")?.kind,
            Kind::UnsupportedChild,
        )?;
    } else {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let issue = walk
            .child(walk.selected().children().next().ok_or("original root")?)
            .err()
            .ok_or("duplicate For refusal")?;
        equal(issue.kind, Kind::UnsupportedChild)?;
        equal(walk.complete().err(), Some(issue))?;
    }
    let output = original.finish();
    check(output.view().is_err())?;
    let file = output.file().ok_or("retained partial File")?;
    equal(file.artifact().node_count(), 0)?;
    equal(file.template_declarations().count(), 0)?;
    equal(file.scopes().len(), 2)?;
    check(file.rejected_for_heads().is_empty())?;
    let element = output
        .selected()
        .children()
        .next()
        .ok_or("original root")?
        .into_element()
        .ok_or("original Element")?;
    equal(element.attributes().len(), 2)?;
    let mut pending = file.unattached_for_heads();
    if carrier_refused {
        check(pending.next().is_none())?;
    } else {
        for (attribute, value) in element
            .attributes()
            .zip(["item in items", "other in items"])
        {
            let input = pending
                .next()
                .ok_or("whole already parked original input")?;
            equal(input.operand().raw_value(), value)?;
            check(input.admitted_for(output.selected(), attribute).is_some())?;
        }
        check(pending.next().is_none())?;
    }
    check(
        !file
            .template_issues()
            .iter()
            .any(|issue| issue.kind == FileIssueKind::DuplicateDeclaration),
    )?;
    Ok(())
}

#[test]
fn equal_original_files_and_node_binding_ids_never_replace_actual_for_allocation()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source =
        "<script setup>const items=2;</script><template><div v-for='item in items'/></template>";
    let first = complete_setup(&arena, source)?;
    let copied_source = source.to_owned();
    for other_source in [source, copied_source.as_str()] {
        let second = complete_setup(&arena, other_source)?;
        let first_file = first.file().ok_or("first File")?;
        let second_file = second.file().ok_or("second File")?;
        let [Op::OriginalFor(first_for)] = first_file.artifact().root().ops.as_slice() else {
            return Err("first original arena Box");
        };
        let [Op::OriginalFor(second_for)] = second_file.artifact().root().ops.as_slice() else {
            return Err("second original arena Box");
        };
        equal(first_for.id(), second_for.id())?;
        equal(first_for.span, second_for.span)?;
        check(!core::ptr::eq(&**first_for, &**second_for))?;
        let first_head = first_file
            .for_head_for(first_for)
            .ok_or("first authentic join")?;
        let second_head = second_file
            .for_head_for(second_for)
            .ok_or("second authentic join")?;
        equal(
            first_head.value().ok_or("value")?.id(),
            second_head.value().ok_or("value")?.id(),
        )?;
        check(first_file.for_head(second_for.id()).is_some())?;
        check(first_file.for_head_for(second_for).is_none())?;
        check(second_file.for_head_for(first_for).is_none())?;
        check(!first_head.accepts(second_for))?;
        check(!second_head.accepts(first_for))?;
        let first_value = first_head
            .value()
            .ok_or("first value")?
            .template_declaration()
            .ok_or("first row")?;
        let second_value = second_head
            .value()
            .ok_or("second value")?
            .template_declaration()
            .ok_or("second row")?;
        check(!first_value.same_owner(&second_value))?;
        let foreign_element = second
            .selected()
            .children()
            .next()
            .ok_or("second root")?
            .into_element()
            .ok_or("second original Element")?;
        check(
            first_head
                .resolution()
                .ok_or("first facts")?
                .input()
                .admitted_for(
                    second.selected(),
                    foreign_element.attributes().next().ok_or("foreign For")?,
                )
                .is_none(),
        )?;
        let first_element = first
            .selected()
            .children()
            .next()
            .ok_or("first root")?
            .into_element()
            .ok_or("first original Element")?;
        check(
            first_head
                .resolution()
                .ok_or("first facts")?
                .input()
                .admitted_for(
                    first.selected(),
                    first_element.attributes().next().ok_or("first For")?,
                )
                .is_some(),
        )?;
    }
    Ok(())
}

#[test]
fn many_original_for_siblings_keep_all_roots_and_allocation_joins_after_growth_and_move()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = format!(
        "<script setup>const items=2;</script><template>{}</template>",
        "<div v-for='item in items' @click='item'/>".repeat(64),
    );
    let descriptor = descriptor(&arena, &source);
    let program = syntax(&arena, &descriptor, true)?;
    let mut original = owner(&arena, &source)?;
    original
        .setup_program(program.admitted_program().ok_or("setup Program")?)
        .map_err(|_| "setup")?;
    let mut roots = Vec::new();
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let mut visits = 0;
        for child in walk.selected().children() {
            walk.child(child).map_err(|_| "original For child")?;
            visits += 1;
        }
        equal(visits, 64)?;
        walk.complete().map_err(|_| "complete")?;
    }
    let output = original.finish();
    let file = output
        .view()
        .map_err(|_| "initial view")?
        .file()
        .ok_or("File")?;
    for operation in &file.artifact().root().ops {
        let Op::OriginalFor(original_for) = operation else {
            return Err("one original For per sibling");
        };
        let resolution = file
            .for_head_for(original_for)
            .ok_or("original join")?
            .resolution()
            .ok_or("original facts")?;
        roots.push((
            &**original_for as *const _,
            resolution.input().aliases().as_ptr(),
            resolution.input().collection() as *const _,
        ));
    }
    let output = core::hint::black_box(output);
    let file = output
        .view()
        .map_err(|_| "moved native view")?
        .file()
        .ok_or("moved File")?;
    equal(file.artifact().root().ops.len(), 64)?;
    equal(file.artifact().node_count(), 192)?;
    equal(file.bindings().count(), 65)?;
    equal(file.template_declarations().count(), 64)?;
    check(file.unattached_for_heads().next().is_none())?;
    check(file.unattached_handlers().next().is_none())?;
    for (ordinal, (operation, roots)) in file.artifact().root().ops.iter().zip(roots).enumerate() {
        let Op::OriginalFor(original_for) = operation else {
            return Err("moved original For");
        };
        check(core::ptr::eq(&**original_for, roots.0))?;
        let joined = file
            .for_head_for(original_for)
            .ok_or("moved allocation join")?;
        let resolution = joined.resolution().ok_or("moved facts")?;
        check(core::ptr::eq(
            resolution.input().aliases().as_ptr(),
            roots.1,
        ))?;
        check(core::ptr::eq(resolution.input().collection(), roots.2))?;
        let element = output
            .selected()
            .children()
            .nth(ordinal)
            .ok_or("original sibling")?
            .into_element()
            .ok_or("actual Element")?;
        check(
            resolution
                .input()
                .admitted_for(
                    output.selected(),
                    element.attributes().next().ok_or("For token")?,
                )
                .is_some(),
        )?;
        let [Op::Element(div)] = original_for.region.ops.as_slice() else {
            return Err("owned sibling Element");
        };
        let [BindingOp::On(on)] = div.bindings.as_slice() else {
            return Err("exactly one actual handler");
        };
        let handler = file.handler_for(on).ok_or("moved original handler")?;
        equal(handler.scope(), joined.scope())?;
        let references = handler.resolution().ok_or("handler facts")?.references();
        equal(references.len(), 1)?;
        equal(
            references[0].binding,
            HandlerBindingRef::Outer(joined.value().ok_or("value")?.id()),
        )?;
    }
    Ok(())
}
