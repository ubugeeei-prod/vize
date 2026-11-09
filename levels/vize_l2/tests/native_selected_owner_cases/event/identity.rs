use super::*;
use vize_l0::dump::{Dump, Mode};
use vize_l2::dump::Page;

#[test]
fn actual_on_membership_rejects_copied_and_foreign_equal_id_bindings() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template><button @click='return $event'/></template>";
    let mut first = owner(&arena, source)?;
    let mut second = owner(&arena, source)?;
    for original in [&mut first, &mut second] {
        let mut walk = original.begin().map_err(|_| "begin")?;
        walk.child(walk.selected().children().next().ok_or("actual child")?)
            .map_err(|_| "child")?;
        walk.complete().map_err(|_| "complete")?;
    }
    let first = first.finish();
    let second = second.finish();
    let first_file = first.file().ok_or("first File")?;
    let second_file = second.file().ok_or("second File")?;
    let [Op::Element(first_button)] = first_file.artifact().root().ops.as_slice() else {
        return Err("button");
    };
    let [BindingOp::On(first_on)] = first_button.bindings.as_slice() else {
        return Err("first On");
    };
    let [Op::Element(second_button)] = second_file.artifact().root().ops.as_slice() else {
        return Err("button");
    };
    let [BindingOp::On(second_on)] = second_button.bindings.as_slice() else {
        return Err("second On");
    };
    let id = first_on
        .handler
        .and_then(OnHandlerRef::body)
        .ok_or("body")?;
    equal(second_on.handler.and_then(OnHandlerRef::body), Some(id))?;
    let first_handler = first_file
        .handler_for(first_on)
        .ok_or("original membership")?;
    check(
        !first_handler.same_owner(
            second_file
                .handler_for(second_on)
                .ok_or("second original")?,
        ),
    )?;
    // Numeric lookup is explicitly local and cannot replace original-On join.
    check(first_file.handler(id).is_some())?;
    check(first_file.handler_for(second_on).is_none())?;
    check(!first_handler.accepts_on(second_on))?;
    let copied = vize_l2::op::OnOp {
        name: first_on.name,
        modifiers: vize_l0::Vec::new_in(&&arena),
        handler: first_on.handler,
        span: first_on.span,
    };
    check(first_file.handler_for(&copied).is_none())?;
    check(!first_handler.accepts_on(&copied))?;
    Ok(())
}

#[test]
fn diagnostic_body_reference_round_trips_without_granting_whole_handler_authority()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template><button @click='return $event'/></template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        walk.child(walk.selected().children().next().ok_or("root")?)
            .map_err(|_| "child")?;
        walk.complete().map_err(|_| "complete")?;
    }
    let output = original.finish();
    let file = output.file().ok_or("File")?;
    let page = Page::of(&file.artifact().root().ops);
    let text = page.print_to_string(Mode::Full);
    check(text.contains("handler-ref=1"))?;
    check(!text.contains("handler=js:"))?;
    let parsed = Page::parse(&text).map_err(|_| "diagnostic parse")?;
    equal(parsed, page)?;
    check(Page::parse(&text.replace("handler-ref=1", "handler-ref=-1")).is_err())?;
    check(
        Page::parse(&text.replace("handler-ref=1", "handler=js(\"$event\") handler-ref=1"))
            .is_err(),
    )?;
    equal(core::mem::size_of::<vize_l2::expr::ExprRef<'_>>(), 16)?;
    equal(core::mem::size_of::<OnHandlerRef<'_>>(), 16)?;
    equal(core::mem::size_of::<vize_l2::op::OnOp<'_>>(), 72)?;
    Ok(())
}

#[test]
fn duplicate_normalized_event_refusal_retains_first_whole_handler_before_mint()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source =
        "<template><button @click='/*first*/ return $event' v-on:click='return 2'/></template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        equal(
            walk.child(walk.selected().children().next().ok_or("root")?)
                .err()
                .ok_or("duplicate refusal")?
                .kind,
            Kind::UnsupportedChild,
        )?;
        check(walk.complete().is_err())?;
    }
    let output = original.finish();
    check(output.view().is_err())?;
    let file = output.file().ok_or("File")?;
    equal(file.artifact().node_count(), 0)?;
    let mut pending = file.unattached_handlers();
    let first = pending.next().ok_or("first original")?;
    equal(first.operand().raw_value(), "/*first*/ return $event")?;
    equal(first.operand().syntax().comments().count(), 1)?;
    equal(
        pending
            .next()
            .ok_or("second original")?
            .operand()
            .raw_value(),
        "return 2",
    )?;
    check(pending.next().is_none())?;
    Ok(())
}
