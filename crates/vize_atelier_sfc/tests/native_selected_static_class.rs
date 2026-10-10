//! Genuine public selected SFC entries consume the original retained class value.

use vize_atelier_sfc::{
    NativeScopedSsrSfcCompileOptions, NativeSelectedSfcDomOptions, NativeSsrSfcCompileOptions,
    compile_native_scoped_ssr_sfc, compile_native_selected_sfc_dom, compile_native_ssr_sfc,
};
use vize_l0::Allocator;
use vize_l4::write::EmitDocument;

fn prepared(code: &str, target: &str) -> String {
    let imports = code
        .lines()
        .filter(|line| line.starts_with("import "))
        .collect::<Vec<_>>()
        .join("\n");
    let declaration = code[imports.len()..]
        .trim_start()
        .strip_prefix("export ")
        .unwrap();
    let (binding, property) = if target == "dom" {
        ("render", "render")
    } else {
        ("ssrRender", "ssrRender")
    };
    format!(
        "{imports}\nconst _sfc_main = {{}}\n;\n{declaration}\n_sfc_main.{property} = {binding}\nexport default _sfc_main\n"
    )
}

fn capture(document: &EmitDocument, map: Option<&str>) -> serde_json::Value {
    serde_json::json!({"code":document.as_str(),"map":map.map(|map|serde_json::from_str::<serde_json::Value>(map).unwrap()),
        "links":document.links().iter().map(|link|serde_json::json!({
            "authored":{"start":link.authored.start,"end":link.authored.end},
            "generated":{"start":link.generated.start,"end":link.generated.end},
            "name":link.name.as_deref(),"segment":link.segment
        })).collect::<Vec<_>>()})
}

#[test]
fn whole_public_selected_class_components_preserve_once_decoding_and_complete_source_maps() {
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/native-selected-static-class-vue-3.5.35.json"
    ))
    .unwrap();
    let mut captures = Vec::new();
    for fixture in pack["fixtures"].as_array().unwrap() {
        let source = fixture["source"].as_str().unwrap();
        for source_map in [true, false] {
            let arena = Allocator::default();
            let dom = compile_native_selected_sfc_dom(
                &arena,
                source,
                NativeSelectedSfcDomOptions {
                    filename: "NativeSelectedClass.vue",
                    source_map,
                    ..Default::default()
                },
            );
            assert!(dom.observation().admitted().is_some(), "{}", fixture["id"]);
            let dom_output = dom.result().unwrap();
            assert_eq!(
                dom_output.code(),
                prepared(fixture["dom"]["code"].as_str().unwrap(), "dom")
            );
            let ssr = compile_native_ssr_sfc(
                &arena,
                source,
                NativeSsrSfcCompileOptions {
                    filename: "NativeSelectedClass.vue",
                    source_map,
                    ..Default::default()
                },
            );
            assert!(ssr.observation().admitted().is_some());
            let ssr_output = ssr.result().unwrap();
            for observation in [dom.observation(), ssr.observation()] {
                assert!(core::ptr::eq(observation.descriptor().source(), source));
                let file = observation.template().unwrap().file().unwrap();
                assert!(file.is_complete());
                for record in file.native_attribute_values() {
                    let value = record.observation().unwrap();
                    assert!(core::ptr::eq(value.source().authored_root(), source));
                    assert_eq!(
                        value.raw_value(),
                        &source[value.value_span().start as usize..value.value_span().end as usize]
                    );
                    assert!(matches!(
                        record.state(),
                        vize_l2::file::NativeFileAttributeValueState::Attached { .. }
                    ));
                }
            }
            for (target, output) in [
                (
                    "dom",
                    capture(dom_output.document(), dom_output.source_map()),
                ),
                (
                    "ssr",
                    capture(ssr_output.document(), ssr_output.source_map()),
                ),
            ] {
                assert_eq!(output["map"].is_null(), !source_map);
                assert_eq!(output["links"].as_array().unwrap().is_empty(), !source_map);
                if source_map {
                    assert_eq!(output["map"]["sourcesContent"], serde_json::json!([source]));
                }
                captures.push(serde_json::json!({"id":fixture["id"],"target":target,"sourceMap":source_map,
                    "source":source,"filename":"NativeSelectedClass.vue","code":output["code"],"map":output["map"],"links":output["links"]}));
            }
        }
    }
    assert_eq!(captures.len(), 76);
    let fixture = &pack["scoped"];
    for source_map in [true, false] {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().unwrap();
        let scope = fixture["scopeId"].as_str().unwrap();
        let compilation = compile_native_scoped_ssr_sfc(
            &arena,
            source,
            NativeScopedSsrSfcCompileOptions {
                filename: "NativeSelectedClass.vue",
                source_map,
                scope_id: Some(scope),
                ..Default::default()
            },
        );
        assert!(compilation.observation().admitted().is_some());
        let output = compilation.result().unwrap();
        assert_eq!(output.css(), Some(fixture["css"].as_str().unwrap()));
        let captured = capture(output.document(), output.source_map());
        let css = capture(output.css_document().unwrap(), output.css_source_map());
        captures.push(serde_json::json!({"id":fixture["id"],"target":"scoped-ssr","sourceMap":source_map,
            "source":source,"filename":"NativeSelectedClass.vue","scopeId":scope,"code":captured["code"],"map":captured["map"],"links":captured["links"],
            "css":css["code"],"cssMap":css["map"],"cssLinks":css["links"]}));
    }
    assert_eq!(captures.len(), 78);
    if let Ok(path) = std::env::var("VIZE_NATIVE_SELECTED_STATIC_CLASS_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
    }
    // Retain the source-built whole packet before the fail-closed reviewed-byte law.
    // Native SSR uses the canonical four-argument ABI and @vue/server-renderer
    // imports; the independent official raw module remains intact in the fixture.
    assert!(
        !pack["nativeCapture"].is_null(),
        "native class packet is unreviewed"
    );
    assert_eq!(serde_json::json!(captures), pack["nativeCapture"]);
}
