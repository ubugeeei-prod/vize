//! Only the named exact original class header becomes normally complete.
use super::{Allocator, check, equal, owner};
use vize_l0::Span;
use vize_l2::{file::NativeFileAttributeValueState, op::Op};

pub(super) fn assert_current(arena: &Allocator, source: &str) -> Result<(), &'static str> {
    equal(
        source,
        "<template>prefix<div data-first='kept' class='a  b'>unvisited</div>tail</template>",
    )?;
    let mut original = owner(arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "original begin")?;
        let selected = walk.selected();
        for child in selected.children() {
            walk.child(child)
                .map_err(|_| "complete named original class source")?;
        }
        walk.complete().map_err(|_| "normal original root end")?;
    }
    let output = original.finish();
    let view = output.view().map_err(|_| "complete original view")?;
    check(core::ptr::eq(view.owner(), &output))?;
    let file = view.file().ok_or("same completed File")?;
    check(core::ptr::eq(file, output.file().ok_or("owner File")?))?;
    check(file.is_complete())?;
    check(core::ptr::eq(file.artifact().source(), source))?;
    equal(file.artifact().node_count(), 4)?;
    equal(output.selected().children().len(), 3)?;
    let [Op::Text(prefix), Op::Element(element), Op::Text(tail)] =
        file.artifact().root().ops.as_slice()
    else {
        return Err("whole original ordered siblings");
    };
    equal((prefix.content, tail.content), ("prefix", "tail"))?;
    let [Op::Text(body)] = element.children.ops.as_slice() else {
        return Err("whole original body");
    };
    equal(body.content, "unvisited")?;
    equal(element.attributes.len(), 2)?;
    equal(file.native_attribute_values().len(), 2)?;
    for (index, expected) in [(0, ("data-first", "kept")), (1, ("class", "a  b"))] {
        let record = &file.native_attribute_values()[index];
        let NativeFileAttributeValueState::Attached { node, slot } = record.state() else {
            return Err("original attached value");
        };
        equal(node.index(), 1)?;
        equal(slot, index)?;
        let joined = file
            .native_attribute_value_for(index, element, slot)
            .ok_or("actual File/Element/slot")?;
        check(core::ptr::eq(joined.file(), file))?;
        check(core::ptr::eq(joined.element(), &**element))?;
        let value = joined
            .observation()
            .ok_or("complete original observation")?;
        check(core::ptr::eq(
            value,
            record.observation().ok_or("same observation")?,
        ))?;
        check(core::ptr::eq(value.source().authored_root(), source))?;
        equal(value.raw_value(), expected.1)?;
        equal(value.source().text(), expected.1)?;
        equal(value.value_span().slice(source), expected.1)?;
        check(value.source().decode_map().is_none())?;
        let attribute = joined.attribute().ok_or("actual canonical slot")?;
        equal(
            (attribute.name, attribute.value),
            (expected.0, Some(expected.1)),
        )?;
        check(core::ptr::eq(
            attribute.value.ok_or("decoded value")?,
            value.source().text(),
        ))?;
    }
    equal(element.attributes[1].span, Span::new(39, 51))?;
    check(file.template_interruption().is_none())?;
    Ok(())
}
