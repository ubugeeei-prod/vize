//! Whole named original class source reaches the native template entry.
use crate::{check, completed, equal};
use vize_l0::Allocator;
use vize_l2::op::Op;
use vize_l3::decision::native::build_native_dom_file_decisions;
use vize_l4::{
    module::assemble_template,
    runtime::{Runtime, vocabulary},
    targets::dom::emit_template,
    write::{NoLinks, Recorded},
};

pub(super) fn assert_current(arena: &Allocator, source: &str) -> Result<(), &'static str> {
    equal(
        source,
        "<template>prefix<div data-first='kept' class='a  b'>unvisited</div>tail</template>",
    )?;
    let owner = completed(arena, source)?;
    let analysis =
        build_native_dom_file_decisions(owner.view().map_err(|_| "complete original view")?)
            .map_err(|_| "complete original L3")?;
    check(core::ptr::eq(analysis.owner(), &owner))?;
    let file = analysis.file();
    check(file.is_complete())?;
    check(core::ptr::eq(file.artifact().source(), source))?;
    let facts = analysis
        .original_attributes()
        .ok_or("whole original values")?;
    check(core::ptr::eq(facts.file(), file))?;
    equal(facts.len(), 2)?;
    let [Op::Text(prefix), Op::Element(element), Op::Text(tail)] =
        file.artifact().root().ops.as_slice()
    else {
        return Err("whole original siblings");
    };
    equal((prefix.content, tail.content), ("prefix", "tail"))?;
    let [Op::Text(body)] = element.children.ops.as_slice() else {
        return Err("whole original body");
    };
    equal(body.content, "unvisited")?;
    for (index, expected) in [(0, ("data-first", "kept")), (1, ("class", "a  b"))] {
        let joined = facts
            .value(index, element, index)
            .ok_or("actual original L3 join")?;
        let value = joined.observation().ok_or("whole original observation")?;
        check(core::ptr::eq(joined.file(), file))?;
        check(core::ptr::eq(
            value,
            file.native_attribute_values()[index]
                .observation()
                .ok_or("same value")?,
        ))?;
        check(core::ptr::eq(value.source().authored_root(), source))?;
        equal(
            (value.raw_value(), value.source().text()),
            (expected.1, expected.1),
        )?;
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
    check(
        analysis
            .dom()
            .ok_or("genuine DOM facts")?
            .unsupported()
            .is_empty(),
    )?;
    let recorded = assemble_template(
        emit_template::<Recorded>(&analysis).map_err(|_| "native render declaration")?,
        vocabulary(Runtime::VueDom),
    )
    .map_err(|_| "whole native module")?;
    let plain = assemble_template(
        emit_template::<NoLinks>(&analysis).map_err(|_| "native unlinked declaration")?,
        vocabulary(Runtime::VueDom),
    )
    .map_err(|_| "whole native unlinked module")?;
    equal(recorded.text.as_str(), plain.text.as_str())?;
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/native-static-class-history-successor-vue-3.5.35.json"
    ))
    .map_err(|_| "independent whole primary fixture")?;
    equal(
        recorded.text.as_str(),
        reference["dom"]["development"]["code"]
            .as_str()
            .ok_or("whole primary module")?,
    )?;
    check(plain.links == NoLinks)?;
    check(!recorded.into_document().links().is_empty())?;
    Ok(())
}
