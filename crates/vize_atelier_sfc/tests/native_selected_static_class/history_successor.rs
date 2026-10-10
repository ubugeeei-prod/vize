//! A separate whole successor leaves the frozen 78-row packet byte-exact.
use super::super::{capture, prepared};
use vize_atelier_sfc::{
    NativeSelectedSfcDomOptions, NativeSsrSfcCompileOptions, compile_native_selected_sfc_dom,
    compile_native_ssr_sfc,
};
use vize_l0::Allocator;

#[test]
fn exact_original_late_class_header_has_whole_public_dom_ssr_successors() {
    let pack: serde_json::Value = serde_json::from_str(include_str!("../../../../davinci/vize_l4/tests/fixtures/native-static-class-history-successor-vue-3.5.35.json")).unwrap();
    let source = pack["source"].as_str().unwrap();
    assert_eq!(
        source,
        "<template>prefix<div data-first='kept' class='a  b'>unvisited</div>tail</template>"
    );
    let mut captures = Vec::new();
    for source_map in [true, false] {
        let arena = Allocator::default();
        let dom = compile_native_selected_sfc_dom(
            &arena,
            source,
            NativeSelectedSfcDomOptions {
                source_map,
                filename: "NativeClassHistory.vue",
                ..Default::default()
            },
        );
        let ssr = compile_native_ssr_sfc(
            &arena,
            source,
            NativeSsrSfcCompileOptions {
                source_map,
                filename: "NativeClassHistory.vue",
                ..Default::default()
            },
        );
        for observation in [dom.observation(), ssr.observation()] {
            assert!(core::ptr::eq(observation.descriptor().source(), source));
            let file = observation.template().unwrap().file().unwrap();
            assert!(file.is_complete());
            assert_eq!(file.native_attribute_values().len(), 2);
        }
        let dom_output = dom.result().unwrap();
        assert_eq!(
            dom_output.code(),
            prepared(pack["dom"]["development"]["code"].as_str().unwrap(), "dom")
        );
        let ssr_output = ssr.result().unwrap();
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
            captures.push(serde_json::json!({"id":"original-late-class-header","source":source,"filename":"NativeClassHistory.vue","target":target,"sourceMap":source_map,"code":output["code"],"map":output["map"],"links":output["links"]}));
        }
    }
    assert_eq!(captures.len(), 4);
    if let Ok(path) = std::env::var("VIZE_NATIVE_SELECTED_STATIC_CLASS_CAPTURE") {
        std::fs::write(
            format!("{path}.late-header.json"),
            serde_json::to_vec_pretty(&captures).unwrap(),
        )
        .unwrap();
    }
    assert!(
        !pack["nativeCapture"].is_null(),
        "whole original late class successor remains unreviewed"
    );
    assert_eq!(serde_json::json!(captures), pack["nativeCapture"]);
}
