use super::*;

#[test]
fn fourteen_original_history_sources_keep_thirteen_actual_slots_and_entity_text_boundary() -> Test {
    let cases = [
        (r#"<div title="a &amp;lt; b">x</div>"#, "a &lt; b", true),
        (r#"<div title="&amp;lt;">x</div>"#, "&lt;", true),
        (r#"<div title="&#38;copy;">x</div>"#, "&copy;", true),
        (r#"<div title="&#x26;#60;">x</div>"#, "&#60;", true),
        (r#"<div title="&amp;amp;lt;">x</div>"#, "&amp;lt;", true),
        (r#"<div title="a&b">x</div>"#, "a&b", true),
        (r#"<div title='a&amp;lt;"b'>x</div>"#, "a&lt;\"b", true),
        (r#"<div title='a"b'>x</div>"#, "a\"b", true),
        ("<div title=a&b>x</div>", "a&b", true),
        (r#"<div title="&lt;b&gt;">x</div>"#, "<b>", true),
        ("<div title=&amp;lt;>x</div>", "&lt;", true),
        (r#"<div class="a&amp;amp;b">x</div>"#, "a&amp;b", true),
        (
            r#"<div title="a &amp;lt; b">&amp;lt;</div>"#,
            "a &lt; b",
            false,
        ),
        (r#"<div title="plain">x</div>"#, "plain", true),
    ];
    for (template, decoded, admitted) in cases {
        let arena = Allocator::default();
        let source = alloc::format!("<template>{template}</template>");
        let mut original = owner(&arena, &source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let selected = walk.selected();
            same(selected.component().block().source(), template)?;
            let child = selected.children().next().ok_or("actual child")?;
            let result = walk.child(child);
            same(result.is_ok(), admitted)?;
            if admitted {
                walk.complete().map_err(|_| "normal end")?;
            }
        }
        let output = core::hint::black_box(original.finish());
        same(output.view().is_ok(), admitted)?;
        let file = output.file().ok_or("actual complete/prefix File")?;
        let [record] = file.native_attribute_values() else {
            return Err("one normal owner");
        };
        let value = record.observation().ok_or("complete value observation")?;
        same(value.source().text(), decoded)?;
        check(core::ptr::eq(
            value.source().authored_root(),
            source.as_str(),
        ))?;
        same(value.value_span().slice(&source), value.raw_value())?;
        if admitted {
            let actual = element(file, 0)?;
            let view = file
                .native_attribute_value_for(0, actual, 0)
                .ok_or("actual allocation/slot")?;
            check(core::ptr::eq(view.file(), file))?;
            check(core::ptr::eq(view.element(), actual))?;
            check(core::ptr::eq(
                view.observation().ok_or("joined value")?,
                value,
            ))?;
            same(actual.attributes[0].value, Some(decoded))?;
            same(
                actual.attributes[0].span.slice(&source),
                &template[5..template.find('>').ok_or("original opening end")?],
            )?;
        } else {
            check(!matches!(record.state(), State::Attached { .. }))?;
            check(!file.is_complete())?;
        }
    }
    Ok(())
}

#[test]
fn one_header_storage_survives_empty_boolean_slots_nested_growth_and_owner_movement() -> Test {
    let arena = Allocator::default();
    let mut source = vize_l0::String::from("<template><div hidden title='' role='&unknown;' ");
    for index in 0..40 {
        source.push_str(&alloc::format!("data-{index}='&amp;lt;' "));
    }
    source.push_str("><span title='雪🌸&acE;'/>x</div><p title='tail'/></template>");
    let output = lower(&arena, &source)?;
    let file = output
        .view()
        .map_err(|_| "completed")?
        .file()
        .ok_or("File")?;
    let parent = element(file, 0)?;
    same(parent.attributes.len(), 43)?;
    same(parent.attributes[0].value, None)?;
    same(file.native_attribute_values().len(), 44)?;
    for (index, slot) in (1..43).enumerate() {
        let joined = file
            .native_attribute_value_for(index, parent, slot)
            .ok_or("grown canonical storage")?;
        let value = joined.observation().ok_or("parked whole value")?;
        check(core::ptr::eq(
            joined
                .attribute()
                .ok_or("actual slot")?
                .value
                .ok_or("value")?,
            value.source().text(),
        ))?;
        check(value.source().decode_map().is_some() == (slot >= 3))?;
        check(file.native_attribute_value_for(index, parent, 0).is_none())?;
    }
    let Some(Op::Element(nested)) = parent.children.ops.first() else {
        return Err("original nested Element");
    };
    let nested_view = file
        .native_attribute_value_for(42, nested, 0)
        .ok_or("nested actual row")?;
    same(
        nested_view
            .observation()
            .ok_or("nested observation")?
            .source()
            .text(),
        "雪🌸∾̳",
    )?;
    check(file.native_attribute_value_for(42, parent, 1).is_none())?;
    let sibling = element(file, 1)?;
    check(file.native_attribute_value_for(43, sibling, 0).is_some())?;
    check(file.native_attribute_value_for(43, nested, 0).is_none())?;
    Ok(())
}

#[test]
fn equal_source_files_sibling_slots_and_neutral_elements_cannot_borrow_value_authority() -> Test {
    let arena = Allocator::default();
    let source =
        "<template><p title='&amp;lt;' role='same'/><p title='&amp;lt;' role='same'/></template>";
    let first = lower(&arena, source)?;
    let second = lower(&arena, source)?;
    let file = first
        .view()
        .map_err(|_| "first view")?
        .file()
        .ok_or("first File")?;
    let foreign = second
        .view()
        .map_err(|_| "second view")?
        .file()
        .ok_or("second File")?;
    let actual = element(file, 0)?;
    for (index, slot) in [(0, 0), (1, 1)] {
        check(
            file.native_attribute_value_for(index, actual, slot)
                .is_some(),
        )?;
        check(
            file.native_attribute_value_for(index, element(file, 1)?, slot)
                .is_none(),
        )?;
        check(
            file.native_attribute_value_for(index, element(foreign, 0)?, slot)
                .is_none(),
        )?;
        check(
            foreign
                .native_attribute_value_for(index, actual, slot)
                .is_none(),
        )?;
        check(
            file.native_attribute_value_for(index, actual, 1 - slot)
                .is_none(),
        )?;
    }
    let mut attributes = vize_l0::Vec::new_in(&&arena);
    for attribute in &actual.attributes {
        attributes.push(crate::op::Attribute {
            name: attribute.name,
            value: attribute.value,
            span: attribute.span,
        });
    }
    let neutral = ElementOp {
        tag: actual.tag,
        namespace: actual.namespace,
        span: actual.span,
        attributes,
        bindings: vize_l0::Vec::new_in(&&arena),
        children: crate::op::Region {
            ops: vize_l0::Vec::new_in(&&arena),
        },
    };
    same(neutral.attributes[0].span, actual.attributes[0].span)?;
    check(core::ptr::eq(
        neutral.attributes[0].name,
        actual.attributes[0].name,
    ))?;
    check(core::ptr::eq(
        neutral.attributes[0].value.ok_or("neutral copied value")?,
        actual.attributes[0].value.ok_or("actual value")?,
    ))?;
    check(file.native_attribute_value_for(0, &neutral, 0).is_none())?;
    Ok(())
}

#[test]
fn actual_for_allocation_alias_scope_and_same_original_handler_join_survive_value_slots() -> Test {
    for lang in ["", " lang='ts'"] {
        let arena = Allocator::default();
        let source = alloc::format!(
            "<script setup{lang}>let items=2</script><template><div title='&amp;lt;' v-for='item in items' @click='$event.count++'>x</div></template>"
        );
        let output = lower_setup(&arena, &source)?;
        let setup = output.setup().map_err(|_| "authentic owned setup")?;
        check(core::ptr::eq(
            setup.syntax(),
            output.retained_setup().ok_or("whole original Program")?,
        ))?;
        let file = output
            .view()
            .map_err(|_| "normal original view")?
            .file()
            .ok_or("File")?;
        let Some(Op::OriginalFor(actual_for)) = file.artifact().root().ops.first() else {
            return Err("original For");
        };
        let head = file
            .for_head_for(actual_for)
            .ok_or("exact original For allocation")?;
        let Some(Op::Element(actual)) = actual_for.region.ops.first() else {
            return Err("original For Element");
        };
        check(file.native_attribute_value_for(0, actual, 0).is_some())?;
        let Some(crate::op::BindingOp::On(on)) = actual.bindings.first() else {
            return Err("actual same handler");
        };
        let handler = file
            .handler_for(on)
            .ok_or("unchanged exact handler allocation")?;
        same(handler.scope(), head.scope())?;
        same(
            handler
                .resolution()
                .ok_or("whole resolution")?
                .input()
                .operand()
                .raw_value(),
            "$event.count++",
        )?;
        same(
            file.native_attribute_values()[0]
                .observation()
                .ok_or("normal value")?
                .source()
                .text(),
            "&lt;",
        )?;
    }
    Ok(())
}

#[test]
fn moved_file_keeps_nonzero_unicode_full_value_and_complete_original_decode_atoms() -> Test {
    let arena = Allocator::default();
    let source =
        "<!--頭🌸-->\r\n<template><p title='雪🌸&acE; &#x1F338; &amp;lt; &unknown;'/></template>";
    let output = lower(&arena, source)?;
    let file = output
        .view()
        .map_err(|_| "normal view")?
        .file()
        .ok_or("File")?;
    let actual = element(file, 0)?;
    let joined = file
        .native_attribute_value_for(0, actual, 0)
        .ok_or("original actual slot")?;
    let value = joined.observation().ok_or("whole observation")?;
    same(value.source().text(), "雪🌸∾̳ 🌸 &lt; &unknown;")?;
    same(
        value.full_value_span().slice(source),
        "'雪🌸&acE; &#x1F338; &amp;lt; &unknown;'",
    )?;
    let original_element = output
        .selected()
        .children()
        .next()
        .ok_or("same original child")?
        .into_element()
        .ok_or("same original Element")?;
    let original_attribute = original_element
        .attributes()
        .next()
        .ok_or("same actual token")?;
    check(
        value
            .admitted_for(output.selected(), original_attribute)
            .is_some(),
    )?;
    let map = value
        .source()
        .decode_map()
        .ok_or("unchanged complete map")?;
    let mut decoded = 0;
    let mut authored = value.value_span().start;
    for segment in map.segments() {
        same(segment.decoded().start, decoded)?;
        same(segment.authored().start, authored)?;
        same(
            value
                .source()
                .authored_span(segment.decoded())
                .map_err(|_| "complete atom")?,
            segment.authored(),
        )?;
        if segment.authored().slice(source) == "&acE;" {
            same(
                value.source().authored_span(vize_l0::Span::new(
                    segment.decoded().start,
                    segment.decoded().start + 3,
                )),
                Err(vize_l1::embed::SourceError::PartialEntityBoundary),
            )?;
        }
        decoded = segment.decoded().end;
        authored = segment.authored().end;
    }
    same(decoded as usize, value.source().text().len())?;
    same(authored, value.value_span().end)
}
