use super::*;
use vize_l0::Span;
use vize_l2::file::{FileIssueKind, RejectedFileFor};

#[test]
fn unicode_alias_and_collection_ranges_keep_both_original_intrinsic_profiles()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for (language, typescript) in [("", false), (" lang='ts'", true)] {
        let source = format!(
            "<script setup{language}>const 項目群=2;</script><template><div v-for='(項目,位置) in 項目群' @click='項目;位置'/></template>",
        );
        let output = complete_setup(&arena, &source)?;
        let file = output
            .view()
            .map_err(|_| "native view")?
            .file()
            .ok_or("File")?;
        let [Op::OriginalFor(original_for)] = file.artifact().root().ops.as_slice() else {
            return Err("original For");
        };
        let facts = file
            .for_head_for(original_for)
            .ok_or("authentic join")?
            .resolution()
            .ok_or("original facts")?;
        let input = facts.input();
        let syntax = input.operand().syntax();
        let aliases = syntax
            .aliases()
            .ok_or("retained aliases")?
            .map_err(|_| "alias owner")?;
        let collection = syntax
            .collection()
            .ok_or("retained collection")?
            .map_err(|_| "collection owner")?;
        equal(aliases.source_type().is_typescript(), typescript)?;
        equal(collection.source_type().is_typescript(), typescript)?;
        check(aliases.source_type().is_module() && collection.source_type().is_module())?;
        check(core::ptr::eq(
            syntax.source().authored_root(),
            source.as_str(),
        ))?;
        equal(input.operand().raw_value(), "(項目,位置) in 項目群")?;
        equal(aliases.source().text(), "項目,位置")?;
        equal(collection.source().text(), "項目群")?;
        let value = facts.value_declaration();
        let key = facts.key_declaration().ok_or("real key declaration")?;
        equal(value.decoded_span(), Span::new(0, 6))?;
        equal(key.decoded_span(), Span::new(7, 13))?;
        equal(value.authored_span().slice(&source), "項目")?;
        equal(key.authored_span().slice(&source), "位置")?;
        equal(facts.collection().span, Span::new(0, 9))?;
        equal(facts.collection_authored_span().slice(&source), "項目群")?;
        check(
            input
                .alias_decoded_span(oxc_span::Span::new(0, aliases.parser_prefix()))
                .is_err(),
        )?;
        check(
            input
                .collection_decoded_span(oxc_span::Span::new(0, collection.parser_prefix()))
                .is_err(),
        )?;
        check(core::ptr::eq(value.parameter(), &input.aliases()[0]))?;
        check(core::ptr::eq(key.parameter(), &input.aliases()[1]))?;
    }
    Ok(())
}

#[test]
fn collection_uses_the_actual_selected_setup_binding_before_same_named_alias_shadowing()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<script setup>const items=2;</script><template><div v-for='(items,index) in items' @click='items'/></template>";
    let output = complete_setup(&arena, source)?;
    let file = output
        .view()
        .map_err(|_| "native view")?
        .file()
        .ok_or("File")?;
    let [Op::OriginalFor(original_for)] = file.artifact().root().ops.as_slice() else {
        return Err("original For");
    };
    let joined = file.for_head_for(original_for).ok_or("same File join")?;
    let script = file
        .lookup(
            joined.enclosing_scope().ok_or("outer scope")?,
            "items",
            Namespace::Value,
        )
        .ok_or("setup items")?;
    let value = joined.value().ok_or("same-named alias")?;
    equal(
        joined.resolution().ok_or("For facts")?.collection().binding,
        script.id(),
    )?;
    check(script.id() != value.id())?;
    let [Op::Element(div)] = original_for.region.ops.as_slice() else {
        return Err("owned Element");
    };
    let [BindingOp::On(on)] = div.bindings.as_slice() else {
        return Err("original event");
    };
    let references = file
        .handler_for(on)
        .ok_or("actual On join")?
        .resolution()
        .ok_or("handler facts")?
        .references();
    equal(references.len(), 1)?;
    equal(references[0].binding, HandlerBindingRef::Outer(value.id()))?;
    Ok(())
}

#[test]
fn ordinary_selected_program_values_remain_invisible_to_the_original_for_collection()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<script>const items=2;</script><template><div v-for='item in items'/></template>";
    let descriptor = descriptor(&arena, source);
    let program = syntax(&arena, &descriptor, false)?;
    let mut original = owner(&arena, source)?;
    original
        .ordinary_program(program.admitted_program().ok_or("ordinary Program")?)
        .map_err(|_| "ordinary entry")?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let issue = walk
            .child(walk.selected().children().next().ok_or("original root")?)
            .err()
            .ok_or("ordinary visibility refusal")?;
        let Kind::For { span, kind } = issue.kind else {
            return Err("For collection refusal");
        };
        equal(kind, FileIssueKind::UnresolvedReference)?;
        equal(span.slice(source), "items")?;
        equal(issue.span, span)?;
        check(walk.complete().is_err())?;
    }
    let output = original.finish();
    check(output.view().is_err())?;
    let file = output.file().ok_or("partial File")?;
    equal(file.bindings().count(), 1)?;
    equal(file.units().len(), 1)?;
    equal(file.template_declarations().count(), 0)?;
    equal(file.artifact().node_count(), 0)?;
    let [RejectedFileFor::Resolution { input, .. }] = file.rejected_for_heads() else {
        return Err("retained original collection owner");
    };
    equal(input.operand().raw_value(), "item in items")?;
    Ok(())
}

#[test]
fn original_for_adds_no_size_growth_to_existing_for_binding_or_operation_variants()
-> Result<(), &'static str> {
    if usize::BITS == 64 {
        equal(core::mem::size_of::<vize_l2::op::ForBinding<'_>>(), 64)?;
        equal(core::mem::size_of::<vize_l2::op::ForOp<'_>>(), 96)?;
        equal(core::mem::size_of::<Op<'_>>(), 16)?;
        equal(core::mem::size_of::<vize_l2::op::OriginalForOp<'_>>(), 40)?;
    }
    Ok(())
}
