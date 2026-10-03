//! The original selected owner feeds L3 and the typed L4 template entry.

use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    markup::NativeTemplateComponent,
};
use vize_l2::lang::js::{NativeTemplateFile, NativeTemplateOwner};
use vize_l3::decision::native::build_native_dom_file_decisions;
use vize_l4::{
    module::assemble_template,
    runtime::{Runtime, vocabulary},
    targets::dom::emit_template,
    write::{NoLinks, Recorded},
};

mod native_selected_template_cases;

fn check(condition: bool) -> Result<(), &'static str> {
    if condition {
        Ok(())
    } else {
        Err("required condition")
    }
}

fn equal<T: PartialEq>(actual: T, expected: T) -> Result<(), &'static str> {
    check(actual == expected)
}

fn field<'f>(fixture: &'f serde_json::Value, name: &str) -> Result<&'f str, &'static str> {
    fixture
        .get(name)
        .and_then(serde_json::Value::as_str)
        .ok_or("fixture string")
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
    let selected = NativeTemplateComponent::parse_in(
        arena,
        descriptor.admitted().map_err(|_| "descriptor admission")?,
    )
    .map_err(|_| "original Component")?
    .ok_or("selected template")?;
    NativeTemplateOwner::new(selected).map_err(|_| "original File start")
}

fn completed<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateFile<'a>, &'static str> {
    let mut original = owner(arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "original root begin")?;
        let selected = walk.selected();
        for child in selected.children() {
            walk.child(child).map_err(|_| "ordered original child")?;
        }
        walk.complete().map_err(|_| "normal original root end")?;
    }
    Ok(core::hint::black_box(original.finish()))
}

#[test]
fn complete_original_selected_templates_keep_custody_and_exact_output() -> Result<(), &'static str>
{
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/selected-template-dom-vue-3.5.35.json"
    ))
    .map_err(|_| "selected reference JSON")?;
    let fixtures = pack
        .get("fixtures")
        .and_then(serde_json::Value::as_array)
        .ok_or("fixtures")?;
    let expected = [
        ("ordered-headers", 4, 7),
        ("unicode-escape", 2, 3),
        ("bare-empty", 1, 3),
        ("text-comment", 3, 0),
        ("empty", 0, 0),
        ("nested", 5, 3),
    ];
    equal(fixtures.len(), expected.len())?;
    let mut captured = Vec::new();
    for (fixture, (id, nodes, attributes)) in fixtures.iter().zip(expected) {
        equal(field(fixture, "id")?, id)?;
        equal(
            fixture.get("nodes").and_then(serde_json::Value::as_u64),
            Some(u64::from(nodes)),
        )?;
        let source = field(fixture, "source")?;
        let arena = Allocator::default();
        let original = completed(&arena, source)?;
        equal(
            original.selected().component().block().source(),
            field(fixture, "template")?,
        )?;
        let analysis =
            build_native_dom_file_decisions(original.view().map_err(|_| "completion view")?)
                .map_err(|_| "same-owner L3")?;
        check(core::ptr::eq(analysis.artifact().source(), source))?;
        let recorded = assemble_template(
            emit_template::<Recorded>(&analysis).map_err(|_| "recorded template")?,
            vocabulary(Runtime::VueDom),
        )
        .map_err(|_| "recorded module")?;
        let plain = assemble_template(
            emit_template::<NoLinks>(&analysis).map_err(|_| "plain template")?,
            vocabulary(Runtime::VueDom),
        )
        .map_err(|_| "plain module")?;
        equal(recorded.text.as_str(), field(fixture, "code")?)?;
        equal(&plain.text, &recorded.text)?;
        equal(&plain.helpers, &recorded.helpers)?;
        let plain_document = plain.into_document();
        check(plain_document.links().is_empty())?;
        let document = recorded.into_document();
        let counts = native_selected_template_cases::projection::inspect(
            id, &original, &analysis, &document,
        )?;
        equal(counts, (nodes, attributes))?;
        equal(analysis.artifact().node_count(), nodes)?;
        let map: serde_json::Value =
            serde_json::from_str(&document.source_map("SelectedTemplate.vue", source))
                .map_err(|_| "whole-source map")?;
        equal(&map["version"], &serde_json::json!(3))?;
        equal(&map["file"], &serde_json::json!("SelectedTemplate.vue"))?;
        equal(
            &map["sources"],
            &serde_json::json!(["SelectedTemplate.vue"]),
        )?;
        equal(&map["sourcesContent"], &serde_json::json!([source]))?;
        equal(&map["names"], &serde_json::json!([]))?;
        check(fixture.get("referenceMap").is_some() && fixture.get("runtime").is_some())?;
        captured.push(serde_json::json!({
            "id": id, "source": source, "code": document.as_str(), "map": map, "nodes": nodes,
        }));
    }
    if let Ok(path) = std::env::var("VIZE_L4_SELECTED_TEMPLATE_CAPTURE") {
        let bytes = serde_json::to_vec_pretty(&captured).map_err(|_| "native capture encoding")?;
        std::fs::write(path, bytes).map_err(|_| "native capture write")?;
    }
    Ok(())
}
