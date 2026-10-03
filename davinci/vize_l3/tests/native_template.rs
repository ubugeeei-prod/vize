use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
    id::NodeId,
};
use vize_l1::{
    SurfaceParseOptions,
    container::{
        Vue,
        vue::{DescriptorObservation, DescriptorOptions},
    },
    markup::NativeTemplateComponent,
};
use vize_l2::{
    lang::js::{NativeTemplateFile, NativeTemplateIssueKind, NativeTemplateOwner},
    op::Op,
};
use vize_l3::decision::{
    StaticLevel, dom::DomRootKind, native::build_native_dom_file_decisions, policy::TargetPolicy,
};

fn check(condition: bool) -> Result<(), &'static str> {
    if condition {
        Ok(())
    } else {
        Err("required condition")
    }
}

fn descriptor<'a>(arena: &'a Allocator, source: &'a str) -> DescriptorObservation<'a> {
    Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    )
}

fn owner<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateOwner<'a>, &'static str> {
    let observation = descriptor(arena, source);
    let selected =
        NativeTemplateComponent::parse_in(arena, observation.admitted().map_err(|_| "descriptor")?)
            .map_err(|_| "component")?
            .ok_or("template")?;
    NativeTemplateOwner::new(selected).map_err(|_| "file start")
}

fn completed<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateFile<'a>, &'static str> {
    let mut owner = owner(arena, source)?;
    {
        let mut walk = owner.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        for child in selected.children() {
            walk.child(child).map_err(|_| "child")?;
        }
        walk.complete().map_err(|_| "complete")?;
    }
    Ok(core::hint::black_box(owner.finish()))
}

#[test]
fn original_text_comment_order_and_owning_file_survive_move() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template>hé<!--original-->tail</template>";
    let output = completed(&arena, source)?;
    let analysis = build_native_dom_file_decisions(output.view().map_err(|_| "view")?)
        .map_err(|_| "analysis")?;
    check(core::ptr::eq(analysis.owner(), &output))?;
    check(core::ptr::eq(analysis.file(), output.file().ok_or("file")?))?;
    check(core::ptr::eq(
        analysis.artifact(),
        analysis.file().artifact(),
    ))?;
    check(analysis.artifact().source() == source)?;
    check(analysis.policy() == TargetPolicy::Dom)?;
    check(analysis.tables().nodes.len() == 3)?;
    check(analysis.tables().controls.is_empty())?;
    let [Op::Text(first), Op::Comment(comment), Op::Text(last)] =
        analysis.artifact().root().ops.as_slice()
    else {
        return Err("actual authored order");
    };
    check(first.content == "hé" && comment.content == "original" && last.content == "tail")?;
    let dom = analysis.dom().ok_or("dom")?;
    check(dom.unsupported().is_empty())?;
    check(dom.root().children.len() == 3)?;
    check(
        analysis
            .tables()
            .nodes
            .get(NodeId::FIRST)
            .ok_or("text row")?
            .output_level
            == StaticLevel::Static,
    )?;
    check(
        analysis
            .tables()
            .nodes
            .get(NodeId::from_index(1).ok_or("comment index")?)
            .ok_or("comment row")?
            .output_level
            == StaticLevel::Dynamic,
    )?;
    check(analysis.expression(NodeId::FIRST).is_none())?;
    Ok(())
}

#[test]
fn a_single_original_comment_retains_its_actual_op() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let output = completed(&arena, "<template><!--body--></template>")?;
    let analysis = build_native_dom_file_decisions(output.view().map_err(|_| "view")?)
        .map_err(|_| "analysis")?;
    let [original] = analysis.artifact().root().ops.as_slice() else {
        return Err("single root");
    };
    check(matches!(original, Op::Comment(comment) if comment.content == "body"))?;
    let dom = analysis.dom().ok_or("dom")?;
    check(core::ptr::eq(
        dom.node(NodeId::FIRST).ok_or("op")?.op(),
        original,
    ))?;
    check(dom.root().kind == DomRootKind::Direct)?;
    check(dom.unsupported().is_empty())?;
    Ok(())
}

#[test]
fn selected_empty_template_has_real_empty_decisions() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let output = completed(&arena, "<template></template>")?;
    let analysis = build_native_dom_file_decisions(output.view().map_err(|_| "view")?)
        .map_err(|_| "analysis")?;
    check(analysis.artifact().node_count() == 0)?;
    check(analysis.tables().nodes.is_empty() && analysis.tables().controls.is_empty())?;
    check(analysis.dom().ok_or("dom")?.root().kind == DomRootKind::Empty)?;
    check(core::ptr::eq(analysis.owner(), &output))?;
    Ok(())
}

#[test]
fn complete_neutral_file_without_original_root_completion_has_no_view() -> Result<(), &'static str>
{
    let arena = Allocator::default();
    let output = owner(&arena, "<template></template>")?.finish();
    check(output.file().ok_or("neutral complete file")?.is_complete())?;
    check(output.view().is_err())?;
    Ok(())
}

#[test]
fn equal_source_foreign_child_cannot_reach_native_analysis() -> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template>same</template>";
    let mut original = owner(&arena, source)?;
    let foreign = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let child = foreign
            .selected()
            .children()
            .next()
            .ok_or("foreign child")?;
        let error = walk.child(child).err().ok_or("foreign refusal")?;
        check(error.kind == NativeTemplateIssueKind::InvalidEvent)?;
    }
    let output = original.finish();
    check(output.view().is_err())?;
    check(
        !output
            .file()
            .is_some_and(vize_l2::file::FileArtifact::is_complete),
    )?;
    Ok(())
}

#[test]
fn unsupported_element_and_entity_roots_keep_no_native_completion() -> Result<(), &'static str> {
    for source in ["<template><div /></template>", "<template>&amp;</template>"] {
        let arena = Allocator::default();
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let child = walk.selected().children().next().ok_or("child")?;
            let error = walk.child(child).err().ok_or("bounded refusal")?;
            check(error.kind == NativeTemplateIssueKind::UnsupportedChild)?;
        }
        check(original.finish().view().is_err())?;
    }
    Ok(())
}

#[test]
fn original_html_whitespace_refuses_before_native_decisions() -> Result<(), &'static str> {
    for source in [
        "<template> </template>",
        "<template>\t</template>",
        "<template>\n</template>",
        "<template>a  b</template>",
        "<template>a\r\nb</template>",
    ] {
        let arena = Allocator::default();
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let child = walk.selected().children().next().ok_or("text")?;
            let issue = walk.child(child).err().ok_or("whitespace refusal")?;
            check(issue.kind == NativeTemplateIssueKind::UnsupportedChild)?;
        }
        let output = original.finish();
        check(output.view().is_err())?;
        check(
            !output
                .file()
                .is_some_and(vize_l2::file::FileArtifact::is_complete),
        )?;
    }
    Ok(())
}

#[test]
fn interrupted_original_walk_retains_partial_custody_without_analysis() -> Result<(), &'static str>
{
    let arena = Allocator::default();
    let mut original = owner(&arena, "<template>first<!--later--></template>")?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let child = walk.selected().children().next().ok_or("first")?;
        walk.child(child).map_err(|_| "first mint")?;
    }
    let output = original.finish();
    check(output.view().is_err())?;
    check(output.selected().children().len() == 2)?;
    let partial = output.file().ok_or("retained partial file")?;
    check(!partial.is_complete())?;
    check(partial.artifact().node_count() == 1)?;
    check(partial.template_interruption().is_some())?;
    Ok(())
}
