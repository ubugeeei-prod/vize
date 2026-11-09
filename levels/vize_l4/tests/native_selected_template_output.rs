//! Sealed original output custody, whole modules and absolute source maps.

use vize_l0::Allocator;
use vize_l2::file::NativeFileInterpolationState;
use vize_l2::op::Op;
use vize_l3::decision::native::build_native_dom_file_decisions;
use vize_l4::{
    module::assemble_template,
    runtime::{Runtime, vocabulary},
    targets::dom::{NativeTemplateDomOutputError, emit_template, emit_template_output},
    write::Recorded,
};

mod native_selected_template_output_cases;
use native_selected_template_output_cases::{check, completed, equal, map_positions};

#[test]
fn original_output_keeps_whole_source_frame_and_five_line_terminators() -> Result<(), &'static str>
{
    for ending in ["\n", "\r\n", "\r", "\u{2028}", "\u{2029}"] {
        let source = format!(
            "<!--雪🌸{ending}--><template><i title='雪🌸'>é🌸</i>{{{{'雪🦀'}}}}</template>"
        );
        let foreign = source.clone();
        check(!core::ptr::eq(source.as_str(), foreign.as_str()))?;
        let arena = Allocator::default();
        let original = completed(&arena, &source)?;
        let analysis = build_native_dom_file_decisions(original.view().map_err(|_| "view")?)
            .map_err(|_| "sole L3 visit")?;
        let baseline = assemble_template(
            emit_template::<Recorded>(&analysis).map_err(|_| "existing encoder")?,
            vocabulary(Runtime::VueDom),
        )
        .map_err(|_| "existing assembler")?;
        let helpers = baseline.helpers.clone();
        let document = baseline.into_document();
        let output = core::hint::black_box(
            emit_template_output(analysis).map_err(|_| "sealed original output")?,
        );
        check(core::ptr::eq(output.analysis().owner(), &original))?;
        check(core::ptr::eq(
            output.analysis().file(),
            original.file().ok_or("File")?,
        ))?;
        check(core::ptr::eq(
            output.source_block().root_source(),
            source.as_str(),
        ))?;
        check(!core::ptr::eq(
            output.source_block().root_source(),
            foreign.as_str(),
        ))?;
        equal(
            output.source_block(),
            original.selected().component().block(),
        )?;
        equal(output.code(), document.as_str())?;
        equal(output.links(), document.links())?;
        equal(output.helpers(), &helpers)?;
        equal(
            output.source_map("雪🌸\".vue"),
            document.source_map("雪🌸\".vue", &source),
        )?;
        check(!output.links().is_empty())?;
        let [Op::Element(element), Op::Interpolation(_)] =
            output.analysis().artifact().root().ops.as_slice()
        else {
            return Err("actual ordered Element and original interpolation");
        };
        let [attribute] = element.attributes.as_slice() else {
            return Err("actual original static Attribute");
        };
        equal(attribute.name, "title")?;
        let key = output
            .links()
            .iter()
            .find(|link| {
                output
                    .code()
                    .get(link.generated.start as usize..link.generated.end as usize)
                    == Some("title")
            })
            .ok_or("actual emitted static key range")?;
        equal(key.authored, attribute.span)?;
        check(key.name.is_none() && output.links().iter().all(|link| link.name.is_none()))?;
        for link in output.links() {
            check(output.source_block().contains_block_span(link.authored))?;
            check(
                output
                    .code()
                    .get(link.generated.start as usize..link.generated.end as usize)
                    .is_some(),
            )?;
            check(link.authored.start > 10)?;
        }
        map_positions(&output, "雪🌸\".vue")?;
        let renamed: serde_json::Value = serde_json::from_str(&output.source_map("Foreign.vue"))
            .map_err(|_| "renamed metadata")?;
        equal(&renamed["sourcesContent"], &serde_json::json!([source]))?;
        equal(&renamed["names"], &serde_json::json!([]))?;
        check(
            renamed["mappings"]
                .as_str()
                .is_some_and(|text| !text.is_empty()),
        )?;
    }
    Ok(())
}

#[test]
fn byte_equal_foreign_originals_keep_distinct_output_owners() -> Result<(), &'static str> {
    let left_source = String::from("<template>{{'🌸'}}</template>");
    let right_source = left_source.clone();
    let (left_arena, right_arena) = (Allocator::default(), Allocator::default());
    let left = completed(&left_arena, &left_source)?;
    let right = completed(&right_arena, &right_source)?;
    let left_output = emit_template_output(
        build_native_dom_file_decisions(left.view().map_err(|_| "left view")?)
            .map_err(|_| "left L3")?,
    )
    .map_err(|_| "left output")?;
    let right_output = emit_template_output(
        build_native_dom_file_decisions(right.view().map_err(|_| "right view")?)
            .map_err(|_| "right L3")?,
    )
    .map_err(|_| "right output")?;
    equal(left_output.code(), right_output.code())?;
    equal(left_output.links(), right_output.links())?;
    equal(
        left_output.source_map("Same.vue"),
        right_output.source_map("Same.vue"),
    )?;
    check(core::ptr::eq(left_output.analysis().owner(), &left))?;
    check(core::ptr::eq(right_output.analysis().owner(), &right))?;
    check(!core::ptr::eq(
        left_output.analysis().file(),
        right_output.analysis().file(),
    ))?;
    check(core::ptr::eq(
        left_output.source_block().root_source(),
        left_source.as_str(),
    ))?;
    check(!core::ptr::eq(
        left_output.source_block().root_source(),
        right_source.as_str(),
    ))?;
    check(core::ptr::eq(
        right_output.source_block().root_source(),
        right_source.as_str(),
    ))?;
    map_positions(&left_output, "Same.vue")?;
    map_positions(&right_output, "Same.vue")
}

#[test]
fn empty_original_keeps_a_valid_empty_map_without_fabricated_links() -> Result<(), &'static str> {
    let source = "<!--prefix雪🌸--><template></template>";
    let arena = Allocator::default();
    let original = completed(&arena, source)?;
    let output = emit_template_output(
        build_native_dom_file_decisions(original.view().map_err(|_| "empty completed view")?)
            .map_err(|_| "empty L3")?,
    )
    .map_err(|_| "empty output")?;
    check(output.links().is_empty() && output.helpers().is_empty())?;
    equal(
        output.code(),
        "\nexport function render(_ctx, _cache, $props, $setup, $data, $options) {\n  return null\n}",
    )?;
    let map: serde_json::Value =
        serde_json::from_str(&output.source_map("Empty.vue")).map_err(|_| "empty whole map")?;
    equal(&map["mappings"], &serde_json::json!(""))?;
    equal(&map["sourcesContent"], &serde_json::json!([source]))?;
    map_positions(&output, "Empty.vue")
}

#[test]
fn refused_output_retains_original_analysis_file_and_whole_input() -> Result<(), &'static str> {
    for source in [
        "<template>{{010}}</template>",
        "<template>{{/a/}}</template>",
        "<template><search /></template>",
    ] {
        let arena = Allocator::default();
        let original = completed(&arena, source)?;
        let analysis = build_native_dom_file_decisions(original.view().map_err(|_| "view")?)
            .map_err(|_| "refused L3 facts")?;
        let expected = emit_template::<Recorded>(&analysis)
            .err()
            .ok_or("typed target refusal")?;
        let Err(failure) = emit_template_output(analysis) else {
            return Err("no partial native output");
        };
        equal(failure.error(), NativeTemplateDomOutputError::Dom(expected))?;
        check(core::ptr::eq(failure.analysis().owner(), &original))?;
        let file = original.file().ok_or("retained complete File")?;
        check(file.is_complete() && core::ptr::eq(failure.analysis().file(), file))?;
        check(core::ptr::eq(
            failure.analysis().artifact().source(),
            source,
        ))?;
        if let Some(record) = file.native_interpolations().first() {
            let NativeFileInterpolationState::Admitted(node) = record.state() else {
                return Err("original normal input admission");
            };
            check(core::ptr::eq(
                file.native_interpolation(node).ok_or("same input")?,
                record,
            ))?;
            let child = original
                .selected()
                .children()
                .next()
                .ok_or("original child")?;
            check(
                record
                    .input()
                    .operand()
                    .admitted_for(original.selected(), child)
                    .is_some(),
            )?;
            equal(expected.span, record.input().operand().full_span())?;
        }
    }
    Ok(())
}
