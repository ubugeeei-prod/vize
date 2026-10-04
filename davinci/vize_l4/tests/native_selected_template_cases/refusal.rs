use crate::{check, completed, equal, owner};
use vize_l0::{Allocator, Span};
use vize_l2::{
    file::{FileIssueKind, RejectedFileHandler},
    lang::js::NativeTemplateIssueKind as Kind,
    op::Op,
    resolution::ResolutionErrorKind,
};
use vize_l3::decision::native::build_native_dom_file_decisions;

#[test]
fn late_original_header_refusals_never_reach_template_emission() -> Result<(), &'static str> {
    let arena = Allocator::default();
    for rejected in [
        "id='a' id='b'",
        "class='a  b'",
        "style='color:red'",
        "key='key'",
        "ref='node'",
        "is='vue:Widget'",
        ":title='value'",
        "@click='handler'",
    ] {
        let source = format!(
            "<template>prefix<div data-first='kept' {rejected}>unvisited</div>tail</template>"
        );
        let mut original = owner(&arena, &source)?;
        let refusal;
        {
            let mut walk = original.begin().map_err(|_| "original begin")?;
            let mut children = walk.selected().children();
            walk.child(children.next().ok_or("actual prefix")?)
                .map_err(|_| "first original sibling")?;
            refusal = walk
                .child(children.next().ok_or("actual late header")?)
                .err()
                .ok_or("header refusal")?;
            let expected = if rejected == "@click='handler'" {
                let start = u32::try_from(source.find("handler").ok_or("authored handler")?)
                    .map_err(|_| "authored offset")?;
                Kind::Handler {
                    span: Span::new(start, start + 7),
                    kind: FileIssueKind::UnresolvedReference,
                }
            } else {
                Kind::UnsupportedChild
            };
            equal(refusal.kind, expected)?;
            equal(
                walk.child(children.next().ok_or("tail")?).err(),
                Some(refusal),
            )?;
            equal(walk.complete().err(), Some(refusal))?;
        }
        let output = original.finish();
        equal(
            output.view().map(build_native_dom_file_decisions).err(),
            Some(refusal),
        )?;
        let original = output
            .selected()
            .children()
            .nth(1)
            .ok_or("retained original")?
            .into_element()
            .ok_or("retained original Element")?;
        check(original.attributes().len() >= 2)?;
        equal(original.children().len(), 1)?;
        let file = output.file().ok_or("same partial File")?;
        check(!file.is_complete())?;
        equal(file.artifact().node_count(), 1)?;
        let [Op::Text(prefix)] = file.artifact().root().ops.as_slice() else {
            return Err("only actual earlier sibling");
        };
        equal(prefix.content, "prefix")?;
        check(file.template_interruption().is_some())?;
        if rejected == "@click='handler'" {
            let [RejectedFileHandler::Resolution { input, error }] = file.rejected_handlers()
            else {
                return Err("whole original unresolved handler");
            };
            equal(error.kind, ResolutionErrorKind::MissingBinding)?;
            equal(error.span, Span::new(0, 7))?;
            equal(input.operand().raw_value(), "handler")?;
            equal(input.operand().syntax().source().text(), "handler")?;
            equal(input.operand().syntax().diagnostics().count(), 0)?;
            check(core::ptr::eq(
                input.operand().syntax().source().authored_root(),
                source.as_str(),
            ))?;
        }
    }
    Ok(())
}

#[test]
fn formerly_refused_entity_header_retains_original_input_for_target_qualification()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source =
        "<template>prefix<div data-first='kept' title='&amp;'>unvisited</div>tail</template>";
    let output = completed(&arena, source)?;
    let file = output
        .view()
        .map_err(|_| "normal original completion")?
        .file()
        .ok_or("same File")?;
    check(file.is_complete())?;
    check(core::ptr::eq(file.artifact().source(), source))?;
    let [Op::Text(prefix), Op::Element(element), Op::Text(tail)] =
        file.artifact().root().ops.as_slice()
    else {
        return Err("original complete siblings");
    };
    equal((prefix.content, tail.content), ("prefix", "tail"))?;
    let [Op::Text(body)] = element.children.ops.as_slice() else {
        return Err("actual body");
    };
    equal(body.content, "unvisited")?;
    equal(file.native_attribute_values().len(), 2)?;
    let joined = file
        .native_attribute_value_for(1, element, 1)
        .ok_or("actual File/Element/slot")?;
    check(core::ptr::eq(joined.file(), file))?;
    check(core::ptr::eq(joined.element(), &**element))?;
    let value = joined.observation().ok_or("whole original value")?;
    equal(value.raw_value(), "&amp;")?;
    equal(value.value_span().slice(source), "&amp;")?;
    equal(value.source().text(), "&")?;
    check(value.source().decode_map().is_some())?;
    check(core::ptr::eq(value.source().authored_root(), source))?;
    let slot = joined.attribute().ok_or("real canonical slot")?;
    equal((slot.name, slot.value), ("title", Some("&")))?;
    check(core::ptr::eq(
        slot.value.ok_or("decoded value")?,
        value.source().text(),
    ))?;
    // This proves original lower custody. Whole native modules/maps/runtime stay separately gated.
    Ok(())
}

#[test]
fn nested_header_refusal_retains_parent_header_and_actual_prefix_without_an_emitter_input()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template><section id='kept'>ok<span title='first' :id='bad'>unvisited</span></section></template>";
    let mut original = owner(&arena, source)?;
    let refusal;
    {
        let mut walk = original.begin().map_err(|_| "original begin")?;
        let child = walk.selected().children().next().ok_or("section")?;
        refusal = walk.child(child).err().ok_or("nested header refusal")?;
        equal(refusal.kind, Kind::UnsupportedChild)?;
        equal(walk.complete().err(), Some(refusal))?;
    }
    let output = original.finish();
    equal(
        output.view().map(build_native_dom_file_decisions).err(),
        Some(refusal),
    )?;
    let file = output.file().ok_or("same partial File")?;
    check(!file.is_complete())?;
    equal(file.artifact().node_count(), 2)?;
    let [Op::Element(section)] = file.artifact().root().ops.as_slice() else {
        return Err("actual retained parent");
    };
    let [attribute] = section.attributes.as_slice() else {
        return Err("completed parent header");
    };
    equal((attribute.name, attribute.value), ("id", Some("kept")))?;
    let [Op::Text(text)] = section.children.ops.as_slice() else {
        return Err("only real earlier child");
    };
    equal(text.content, "ok")?;
    let original = output
        .selected()
        .children()
        .next()
        .ok_or("original section")?
        .into_element()
        .ok_or("original Element")?;
    let header = original.attributes().next().ok_or("same original header")?;
    check(core::ptr::eq(attribute.name, header.surface().name.text))?;
    check(core::ptr::eq(
        attribute.value.ok_or("canonical value")?,
        header
            .surface()
            .value
            .as_ref()
            .ok_or("original value")?
            .content
            .text,
    ))?;
    let refused = original
        .children()
        .nth(1)
        .ok_or("refused original child")?
        .into_element()
        .ok_or("refused Element")?;
    equal(refused.attributes().len(), 2)?;
    equal(refused.children().len(), 1)?;
    check(file.template_interruption().is_some())?;
    Ok(())
}

#[test]
fn dropped_forgotten_and_unwound_root_completion_cannot_be_replaced_by_complete_headers()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template><section id='kept'><span title='雪'>ok</span><input disabled/></section></template>";
    // Establish that the identical authentic source can complete normally.
    let complete = completed(&arena, source)?;
    check(complete.view().is_ok())?;
    for mode in 0..3 {
        let mut original = owner(&arena, source)?;
        if mode == 2 {
            let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || -> Result<(), &'static str> {
                    let mut walk = original.begin().map_err(|_| "original begin")?;
                    let child = walk.selected().children().next().ok_or("whole section")?;
                    walk.child(child).map_err(|_| "complete header and body")?;
                    std::panic::resume_unwind(Box::new(()));
                },
            ));
            check(caught.is_err())?;
        } else {
            let mut walk = original.begin().map_err(|_| "original begin")?;
            let child = walk.selected().children().next().ok_or("whole section")?;
            walk.child(child).map_err(|_| "complete header and body")?;
            if mode == 1 {
                core::mem::forget(walk);
            }
        }
        let output = original.finish();
        let refusal = output
            .view()
            .map(build_native_dom_file_decisions)
            .err()
            .ok_or("no original completion view")?;
        equal(refusal.kind, Kind::Interrupted)?;
        let file = output.file().ok_or("same incomplete File")?;
        check(!file.is_complete())?;
        equal(file.artifact().node_count(), 4)?;
        let [Op::Element(section)] = file.artifact().root().ops.as_slice() else {
            return Err("actual complete Element prefix");
        };
        let [attribute] = section.attributes.as_slice() else {
            return Err("actual parent header");
        };
        equal((attribute.name, attribute.value), ("id", Some("kept")))?;
        equal(section.children.ops.len(), 2)?;
        // Forget preserves a pending walk; only Drop/unwind records interruption.
        equal(file.template_interruption().is_some(), mode != 1)?;
    }
    Ok(())
}
