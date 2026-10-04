use super::{Allocator, Kind, check, equal, kind, owner, selected};
use vize_l0::Span;
use vize_l2::op::Op;

const SOURCE: &str = "<template><section id = 'hé there' hidden data-empty=\"\" data-count=二 aria-label='original'><input disabled/><span title=\"kept\">ok</span></section></template>";

#[test]
fn original_static_header_retains_full_ordered_tokens_spans_and_nested_body()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let mut original = owner(&arena, SOURCE)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        for child in selected.children() {
            walk.child(child).map_err(|_| "original header and body")?;
        }
        walk.complete().map_err(|_| "normal original root end")?;
    }
    let output = core::hint::black_box(original.finish());
    let view = output.view().map_err(|_| "completed original view")?;
    check(core::ptr::eq(view.owner(), &output))?;
    let file = view.file().ok_or("same File")?;
    check(core::ptr::eq(file, output.file().ok_or("owner File")?))?;
    check(file.is_complete())?;
    check(core::ptr::eq(file.artifact().source(), SOURCE))?;
    equal(file.artifact().node_count(), 4)?;
    let [Op::Element(section)] = file.artifact().root().ops.as_slice() else {
        return Err("actual section");
    };
    let [Op::Element(input), Op::Element(span)] = section.children.ops.as_slice() else {
        return Err("actual nested Elements");
    };
    let [Op::Text(text)] = span.children.ops.as_slice() else {
        return Err("actual body text");
    };
    equal(text.content, "ok")?;
    let original_section = output
        .selected()
        .children()
        .next()
        .ok_or("original root")?
        .into_element()
        .ok_or("original section")?;
    let mut children = original_section.children();
    let original_input = children
        .next()
        .ok_or("original input")?
        .into_element()
        .ok_or("input Element")?;
    let original_span = children
        .next()
        .ok_or("original span")?
        .into_element()
        .ok_or("span Element")?;
    let expected = [
        vec![
            ("id", Some("hé there"), Span::new(19, 35)),
            ("hidden", None, Span::new(36, 42)),
            ("data-empty", Some(""), Span::new(43, 56)),
            ("data-count", Some("二"), Span::new(57, 71)),
            ("aria-label", Some("original"), Span::new(72, 93)),
        ],
        vec![("disabled", None, Span::new(101, 109))],
        vec![("title", Some("kept"), Span::new(117, 129))],
    ];
    for ((original, canonical), expected) in [
        (&original_section, section),
        (&original_input, input),
        (&original_span, span),
    ]
    .into_iter()
    .zip(expected)
    {
        equal(canonical.attributes.len(), expected.len())?;
        check(canonical.bindings.is_empty())?;
        for ((attribute, canonical), expected) in original
            .attributes()
            .zip(&canonical.attributes)
            .zip(expected)
        {
            check(core::ptr::eq(
                attribute.component(),
                output.selected().component(),
            ))?;
            check(core::ptr::eq(attribute.element(), original.surface()))?;
            check(core::ptr::eq(
                attribute.surface(),
                original
                    .surface()
                    .open
                    .attrs
                    .get(attribute.ordinal())
                    .ok_or("complete original token")?,
            ))?;
            check(core::ptr::eq(canonical.name, attribute.surface().name.text))?;
            equal((canonical.name, canonical.value, canonical.span), expected)?;
            if let (Some(actual), Some(value)) = (canonical.value, &attribute.surface().value) {
                check(core::ptr::eq(actual, value.content.text))?;
            }
        }
    }
    Ok(())
}

#[test]
fn refusal_at_the_last_header_never_mints_that_element_or_body() -> Result<(), &'static str> {
    let arena = Allocator::default();
    for rejected in [
        "id='a' id='b'",
        "class='a  b'",
        "style='color:red'",
        "key='key'",
        "ref='node'",
        "is='vue:Widget'",
        ":title='value'",
        ".title='value'",
        "@click='handler'",
        "#default",
        "v-custom='value'",
    ] {
        let source = format!(
            "<template>prefix<div data-first='kept' {rejected}>unvisited</div>tail</template>"
        );
        let mut original = owner(&arena, &source)?;
        let refusal;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let mut children = walk.selected().children();
            walk.child(children.next().ok_or("prefix")?)
                .map_err(|_| "actual first sibling")?;
            refusal = walk
                .child(children.next().ok_or("actual header")?)
                .err()
                .ok_or("refused complete original header")?;
            if rejected == "@click='handler'" {
                let Kind::Handler { span, kind } = refusal.kind else {
                    return Err("precise event refusal");
                };
                equal(kind, vize_l2::file::FileIssueKind::UnresolvedReference)?;
                equal(span.slice(&source), "handler")?;
                equal(refusal.span, span)?;
            } else {
                equal(refusal.kind, Kind::UnsupportedChild)?;
            }
            equal(
                walk.child(children.next().ok_or("tail")?).err(),
                Some(refusal),
            )?;
            equal(walk.complete().err(), Some(refusal))?;
        }
        let output = original.finish();
        equal(output.view().err(), Some(refusal))?;
        equal(output.selected().children().len(), 3)?;
        let original_element = output
            .selected()
            .children()
            .nth(1)
            .ok_or("retained refused original")?
            .into_element()
            .ok_or("retained Element")?;
        check(original_element.attributes().len() >= 2)?;
        equal(original_element.children().len(), 1)?;
        let file = output.file().ok_or("same partial File")?;
        check(!file.is_complete())?;
        equal(file.artifact().node_count(), 1)?;
        let [Op::Text(prefix)] = file.artifact().root().ops.as_slice() else {
            return Err("only actual first sibling");
        };
        equal(prefix.content, "prefix")?;
        check(file.template_interruption().is_some())?;
    }
    Ok(())
}

#[test]
fn formerly_refused_entity_header_retains_its_exact_original_slot() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let source =
        "<template>prefix<div data-first='kept' title='&amp;'>unvisited</div>tail</template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        for child in selected.children() {
            walk.child(child).map_err(|_| "actual original child")?;
        }
        walk.complete().map_err(|_| "normal original end")?;
    }
    let output = core::hint::black_box(original.finish());
    let file = output
        .view()
        .map_err(|_| "complete original")?
        .file()
        .ok_or("same File")?;
    check(file.is_complete())?;
    check(core::ptr::eq(file.artifact().source(), source))?;
    equal(file.artifact().node_count(), 4)?;
    let [Op::Text(prefix), Op::Element(element), Op::Text(tail)] =
        file.artifact().root().ops.as_slice()
    else {
        return Err("same ordered original siblings");
    };
    equal((prefix.content, tail.content), ("prefix", "tail"))?;
    let [Op::Text(body)] = element.children.ops.as_slice() else {
        return Err("original body");
    };
    equal(body.content, "unvisited")?;
    equal(file.native_attribute_values().len(), 2)?;
    let joined = file
        .native_attribute_value_for(1, element, 1)
        .ok_or("same Element/slot")?;
    check(core::ptr::eq(joined.file(), file))?;
    check(core::ptr::eq(joined.element(), element))?;
    let value = joined.observation().ok_or("whole original preparation")?;
    equal(value.raw_value(), "&amp;")?;
    equal(value.value_span().slice(source), "&amp;")?;
    equal(value.source().text(), "&")?;
    check(value.source().decode_map().is_some())?;
    check(core::ptr::eq(value.source().authored_root(), source))?;
    let slot = joined.attribute().ok_or("actual canonical slot")?;
    equal((slot.name, slot.value), ("title", Some("&")))?;
    check(core::ptr::eq(
        slot.value.ok_or("value")?,
        value.source().text(),
    ))?;
    Ok(())
}

#[test]
fn nested_header_refusal_keeps_only_the_completed_parent_header_and_actual_prefix()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template><section id='kept'>ok<span title='first' :id='bad'>unvisited</span></section></template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let child = walk.selected().children().next().ok_or("section")?;
        equal(kind(walk.child(child))?, Kind::UnsupportedChild)?;
        equal(kind(walk.complete())?, Kind::UnsupportedChild)?;
    }
    let output = original.finish();
    check(output.view().is_err())?;
    let file = output.file().ok_or("actual partial File")?;
    check(!file.is_complete())?;
    equal(file.artifact().node_count(), 2)?;
    let [Op::Element(section)] = file.artifact().root().ops.as_slice() else {
        return Err("only actual parent");
    };
    let [attribute] = section.attributes.as_slice() else {
        return Err("completed actual parent header");
    };
    equal((attribute.name, attribute.value), ("id", Some("kept")))?;
    let [Op::Text(text)] = section.children.ops.as_slice() else {
        return Err("only actual text before refused child header");
    };
    equal(text.content, "ok")?;
    Ok(())
}

#[test]
fn foreign_attribute_bearing_elements_do_not_establish_original_header_custody()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template><div id='first'/><span title='second'/></template>";
    let copy = source.to_owned();
    let foreign = selected(&arena, &copy)?;
    for reordered in [false, true] {
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let child = if reordered {
                walk.selected().children().nth(1).ok_or("reordered")?
            } else {
                foreign.children().next().ok_or("equal bytes foreign")?
            };
            equal(kind(walk.child(child))?, Kind::InvalidEvent)?;
            check(walk.complete().is_err())?;
        }
        let output = original.finish();
        check(output.view().is_err())?;
        equal(
            output
                .file()
                .ok_or("original File")?
                .artifact()
                .node_count(),
            0,
        )?;
    }
    Ok(())
}

#[test]
fn complete_static_headers_and_bodies_do_not_replace_normal_root_completion()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for mode in 0..3 {
        let mut original = owner(&arena, SOURCE)?;
        if mode == 2 {
            let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || -> Result<(), &'static str> {
                    let mut walk = original.begin().map_err(|_| "begin")?;
                    let child = walk.selected().children().next().ok_or("section")?;
                    walk.child(child).map_err(|_| "complete header and body")?;
                    std::panic::resume_unwind(Box::new(()));
                },
            ));
            check(interrupted.is_err())?;
        } else {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let child = walk.selected().children().next().ok_or("section")?;
            walk.child(child)
                .map_err(|_| "complete original header and body")?;
            if mode == 1 {
                core::mem::forget(walk);
            }
        }
        equal(kind(original.begin())?, Kind::Interrupted)?;
        let output = original.finish();
        check(output.view().is_err())?;
        let file = output.file().ok_or("retained original File")?;
        equal(file.artifact().node_count(), 4)?;
        check(!file.is_complete())?;
        let [Op::Element(section)] = file.artifact().root().ops.as_slice() else {
            return Err("actual completed Element prefix");
        };
        equal(section.attributes.len(), 5)?;
        // Forget skips Drop: its pending walk remains incomplete without
        // inventing a recorded interruption that only drop/unwind observes.
        equal(file.template_interruption().is_some(), mode != 1)?;
    }
    Ok(())
}
