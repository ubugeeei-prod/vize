//! Complete original literal modules, genuine recorded ranges and hosted capture.

use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceChild, SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    markup::NativeTemplateComponent,
};
use vize_l2::lang::js::{NativeInterpolationInput, NativeTemplateFile, NativeTemplateOwner};
use vize_l3::decision::native::build_native_dom_file_decisions;
use vize_l4::{
    module::assemble_template,
    runtime::{Runtime, vocabulary},
    targets::dom::{emit_template, emit_template_output},
    write::{NoLinks, Recorded},
};

mod native_selected_template_literal_cases;

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

fn completed<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateFile<'a>, &'static str> {
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
            .map_err(|_| "original selected Component")?
            .ok_or("selected template")?;
    let mut owner = NativeTemplateOwner::new(selected).map_err(|_| "original File begin")?;
    {
        let mut walk = owner.begin().map_err(|_| "root begin")?;
        let selected = walk.selected();
        for child in selected.children() {
            if matches!(child.surface(), SurfaceChild::Interpolation(_)) {
                let input = NativeInterpolationInput::from_operand(
                    selected
                        .observe_interpolation_expression(child.reborrow())
                        .map_err(|_| "original stock observation")?,
                );
                walk.root_interpolation(child, input)
                    .map_err(|_| "original input receiver")?;
            } else {
                walk.child(child).map_err(|_| "original ordinary child")?;
            }
        }
        walk.complete().map_err(|_| "normal root end")?;
    }
    Ok(core::hint::black_box(owner.finish()))
}

#[test]
fn original_modern_literals_keep_complete_module_map_and_projection_custody()
-> Result<(), &'static str> {
    const CASES: &[&str] = &[
        include_str!("fixtures/selected-template-literal-vue-3.5.35/decimal.json"),
        include_str!("fixtures/selected-template-literal-vue-3.5.35/hex.json"),
        include_str!("fixtures/selected-template-literal-vue-3.5.35/octal-modern.json"),
        include_str!("fixtures/selected-template-literal-vue-3.5.35/binary-modern.json"),
        include_str!("fixtures/selected-template-literal-vue-3.5.35/exponent.json"),
        include_str!("fixtures/selected-template-literal-vue-3.5.35/bigint.json"),
        include_str!("fixtures/selected-template-literal-vue-3.5.35/boolean.json"),
        include_str!("fixtures/selected-template-literal-vue-3.5.35/null.json"),
        include_str!("fixtures/selected-template-literal-vue-3.5.35/unicode-entity.json"),
        include_str!("fixtures/selected-template-literal-vue-3.5.35/escaped-string.json"),
        include_str!("fixtures/selected-template-literal-vue-3.5.35/comment-literal.json"),
        include_str!("fixtures/selected-template-literal-vue-3.5.35/mixed.json"),
    ];
    let expected = [
        "decimal",
        "hex",
        "octal-modern",
        "binary-modern",
        "exponent",
        "bigint",
        "boolean",
        "null",
        "unicode-entity",
        "escaped-string",
        "comment-literal",
        "mixed",
    ];
    equal(CASES.len(), expected.len())?;
    let mut captured = Vec::new();
    for (packet, id) in CASES.iter().zip(expected) {
        let fixture: serde_json::Value =
            serde_json::from_str(packet).map_err(|_| "original packet")?;
        equal(field(&fixture, "id")?, id)?;
        let source = field(&fixture, "source")?;
        let nodes = if id == "mixed" { 4 } else { 1 };
        equal(
            fixture.get("nodes").and_then(serde_json::Value::as_u64),
            Some(u64::from(nodes)),
        )?;
        let arena = Allocator::default();
        let original = completed(&arena, source)?;
        equal(
            original.selected().component().block().source(),
            field(&fixture, "template")?,
        )?;
        let analysis =
            build_native_dom_file_decisions(original.view().map_err(|_| "completed view")?)
                .map_err(|_| "sole L3 visit")?;
        let recorded = assemble_template(
            emit_template::<Recorded>(&analysis).map_err(|_| "original recorded entry")?,
            vocabulary(Runtime::VueDom),
        )
        .map_err(|_| "complete recorded module")?;
        let plain = assemble_template(
            emit_template::<NoLinks>(&analysis).map_err(|_| "original plain entry")?,
            vocabulary(Runtime::VueDom),
        )
        .map_err(|_| "complete plain module")?;
        equal(recorded.text.as_str(), field(&fixture, "code")?)?;
        equal(&recorded.text, &plain.text)?;
        equal(&recorded.helpers, &plain.helpers)?;
        check(plain.into_document().links().is_empty())?;
        let document = recorded.into_document();
        native_selected_template_literal_cases::inspect(
            id, &original, &analysis, &document, nodes,
        )?;
        let output = core::hint::black_box(
            emit_template_output(analysis).map_err(|_| "sealed recorded output")?,
        );
        equal(output.code(), document.as_str())?;
        equal(output.links(), document.links())?;
        check(core::ptr::eq(output.analysis().owner(), &original))?;
        check(core::ptr::eq(output.source_block().root_source(), source))?;
        let map: serde_json::Value =
            serde_json::from_str(&output.source_map("SelectedTemplate.vue"))
                .map_err(|_| "actual whole-source map")?;
        equal(&map["version"], &serde_json::json!(3))?;
        equal(&map["file"], &serde_json::json!("SelectedTemplate.vue"))?;
        equal(
            &map["sources"],
            &serde_json::json!(["SelectedTemplate.vue"]),
        )?;
        equal(&map["sourcesContent"], &serde_json::json!([source]))?;
        equal(&map["names"], &serde_json::json!([]))?;
        check(
            map["mappings"]
                .as_str()
                .is_some_and(|text| !text.is_empty()),
        )?;
        let links: Vec<_> = output
            .links()
            .iter()
            .map(|link| {
                serde_json::json!({
                    "generated": [link.generated.start, link.generated.end],
                    "authored": [link.authored.start, link.authored.end],
                    "name": link.name.as_ref().map(|name| name.as_str()),
                    "segment": link.segment,
                })
            })
            .collect();
        captured.push(serde_json::json!({
            "id": id, "source": source, "code": output.code(), "map": map,
            "nodes": nodes, "links": links,
        }));
    }
    capture(&captured)
}

fn capture(captured: &[serde_json::Value]) -> Result<(), &'static str> {
    let path = if let Ok(path) = std::env::var("VIZE_L4_SELECTED_LITERAL_CAPTURE") {
        Some(std::path::PathBuf::from(path))
    } else if let Ok(profile) = std::env::var("NEXTEST_PROFILE") {
        Some(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/nextest")
                .join(profile)
                .join("native-selected-template-literals.json"),
        )
    } else {
        None
    };
    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|_| "native capture directory")?;
        }
        let bytes =
            serde_json::to_vec_pretty(captured).map_err(|_| "complete native capture encoding")?;
        std::fs::write(path, bytes).map_err(|_| "complete native capture write")?;
    }
    Ok(())
}
