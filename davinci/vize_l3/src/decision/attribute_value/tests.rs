use super::*;
use crate::decision::{native, policy::TargetPolicy, ssr, vapor};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    markup::NativeTemplateComponent,
};
use vize_l2::{
    lang::js::{NativeTemplateFile, NativeTemplateOwner},
    op::Op,
};

mod refusal;
mod scope;
mod scoped;
mod setup;

type Test = Result<(), &'static str>;
fn check(value: bool) -> Test {
    if value {
        Ok(())
    } else {
        Err("original attribute condition")
    }
}
fn owner<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateOwner<'a>, &'static str> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let selected =
        NativeTemplateComponent::parse_in(arena, descriptor.admitted().map_err(|_| "descriptor")?)
            .map_err(|_| "original template")?
            .ok_or("missing template")?;
    NativeTemplateOwner::new(selected).map_err(|_| "original owner")
}
fn completed<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateFile<'a>, &'static str> {
    let mut original = owner(arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        for child in walk.selected().children() {
            walk.child(child).map_err(|_| "child")?;
        }
        walk.complete().map_err(|_| "complete")?;
    }
    Ok(original.finish())
}
fn element<'f, 'a>(
    file: &'f FileArtifact<'a>,
    index: usize,
) -> Result<&'f ElementOp<'a>, &'static str> {
    match file.artifact().root().ops.get(index) {
        Some(Op::Element(element)) => Ok(element),
        _ => Err("actual Element"),
    }
}
fn node(file: &FileArtifact<'_>, index: usize) -> Result<NodeId, &'static str> {
    match file
        .native_attribute_values()
        .get(index)
        .ok_or("original row")?
        .state()
    {
        NativeFileAttributeValueState::Attached { node, .. } => Ok(node),
        _ => Err("normal attachment"),
    }
}

#[test]
fn original_title_family_keeps_same_file_full_preparation_in_each_sole_target_walk() -> Test {
    let cases = [
        (r#"<div title="a &amp;lt; b">x</div>"#, "a &lt; b"),
        (r#"<div title="&amp;lt;">x</div>"#, "&lt;"),
        (r#"<div title="&#38;copy;">x</div>"#, "&copy;"),
        (r#"<div title="&#x26;#60;">x</div>"#, "&#60;"),
        (r#"<div title="&amp;amp;lt;">x</div>"#, "&amp;lt;"),
        (r#"<div title="a&b">x</div>"#, "a&b"),
        (r#"<div title='a&amp;lt;"b'>x</div>"#, "a&lt;\"b"),
        (r#"<div title='a"b'>x</div>"#, "a\"b"),
        ("<div title=a&b>x</div>", "a&b"),
        (r#"<div title="&lt;b&gt;">x</div>"#, "<b>"),
        ("<div title=&amp;lt;>x</div>", "&lt;"),
        (r#"<div title="plain">x</div>"#, "plain"),
    ];
    for (template, expected) in cases {
        let arena = Allocator::default();
        let source = alloc::format!("<template>{template}</template>");
        let original = completed(&arena, &source)?;
        let file = original.file().ok_or("normal File")?;
        let actual = element(file, 0)?;
        let dom =
            native::build_native_dom_file_decisions(original.view().map_err(|_| "native view")?)
                .map_err(|_| "DOM cursor")?;
        let ssr = ssr::build_native_ssr_file_decisions(original.view().map_err(|_| "SSR view")?)
            .map_err(|_| "SSR cursor")?;
        let vapor =
            vapor::build_native_vapor_file_decisions(original.view().map_err(|_| "Vapor view")?)
                .map_err(|_| "Vapor cursor")?;
        check(dom.dom().ok_or("DOM facts")?.unsupported().is_empty())?;
        check(ssr.ssr().ok_or("SSR facts")?.unsupported().is_empty())?;
        check(vapor.vapor().ok_or("Vapor facts")?.unsupported().is_empty())?;
        for facts in [
            dom.original_attributes(),
            ssr.original_attributes(),
            vapor.original_attributes(),
        ] {
            let facts = facts.ok_or("native whole-row receipt")?;
            check(core::ptr::eq(facts.file(), file) && facts.len() == 1)?;
            let value = facts
                .value(0, actual, 0)
                .ok_or("actual slot")?
                .observation()
                .ok_or("whole source")?;
            check(
                value.source().text() == expected
                    && core::ptr::eq(value.source().authored_root(), source.as_str()),
            )?;
            check(value.value_span().slice(&source) == value.raw_value())?;
            check(core::ptr::eq(
                value,
                file.native_attribute_values()[0]
                    .observation()
                    .ok_or("parked original")?,
            ))?;
        }
    }
    Ok(())
}

#[test]
fn bare_none_empty_unknown_and_nested_unicode_slots_consume_only_their_actual_rows() -> Test {
    let arena = Allocator::default();
    let source = "<template><div hidden title='' data-note='&unknown;'><span title='😀&amp;lt;'/></div><div title='plain'/></template>";
    let original = completed(&arena, source)?;
    let file = original.file().ok_or("File")?;
    let analysis = native::build_native_dom_file_decisions(original.view().map_err(|_| "view")?)
        .map_err(|_| "cursor")?;
    let facts = analysis.original_attributes().ok_or("normal finish")?;
    check(facts.len() == 4 && core::ptr::eq(facts.file(), file))?;
    let parent = element(file, 0)?;
    let [Op::Element(child)] = parent.children.ops.as_slice() else {
        return Err("nested actual Element");
    };
    for (index, actual, slot, expected) in [
        (0, parent, 1, ""),
        (1, parent, 2, "&unknown;"),
        (2, child.as_ref(), 0, "😀&lt;"),
        (3, element(file, 1)?, 0, "plain"),
    ] {
        let value = facts
            .value(index, actual, slot)
            .ok_or("exact nested slot")?
            .observation()
            .ok_or("normal source")?;
        check(
            value.source().text() == expected
                && core::ptr::eq(value.source().authored_root(), source),
        )?;
        check(core::ptr::eq(
            actual.attributes[slot].value.ok_or("Some even empty")?,
            value.source().text(),
        ))?;
    }
    check(parent.attributes[0].value.is_none())?;
    check(facts.value(0, parent, 0).is_none())?;
    Ok(())
}

#[test]
fn generic_file_and_bare_artifact_routes_preserve_none_original_value_authority() -> Test {
    let arena = Allocator::default();
    let original = completed(&arena, "<template><div title='plain'>x</div></template>")?;
    let file = original.file().ok_or("File")?;
    let dom = crate::decision::build_dom_file_decisions(file).map_err(|_| "generic DOM")?;
    let ssr = ssr::build_ssr_file_decisions(file).map_err(|_| "generic SSR")?;
    let vapor = vapor::build_vapor_file_decisions(file).map_err(|_| "generic Vapor")?;
    check(
        dom.original_attributes().is_none()
            && ssr.original_attributes().is_none()
            && vapor.original_attributes().is_none(),
    )?;
    for policy in [TargetPolicy::Dom, TargetPolicy::Ssr, TargetPolicy::Vapor] {
        let analysis = crate::decision::build_decisions(file.artifact(), policy)
            .map_err(|_| "bare analysis")?;
        check(analysis.original_attributes().is_none())?;
    }
    Ok(())
}
