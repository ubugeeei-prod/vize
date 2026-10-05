//! Complete genuine primitive setup component modules, maps and refusals.
use vize_atelier_sfc::{
    NativeVaporSetupSfcCompileError, NativeVaporSfcCompileOptions, compile_native_vapor_setup_sfc,
};
use vize_l0::Allocator;
use vize_l3::decision::vapor::{VaporUnsupported, build_native_selected_setup_vapor_decisions};
use vize_l4::{
    module::AssemblyError,
    targets::vapor::{VaporErrorKind, emit_selected_setup_component},
    write::{NoLinks, Recorded},
};

#[test]
fn complete_original_setup_components_and_maps_match_independent_expectations() {
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/native_vapor_setup_sfc_vue_3_6_rc9.json"
    ))
    .unwrap();
    assert_eq!(pack["version"], "3.6.0-rc.9");
    let mut captures = Vec::new();
    for fixture in pack["fixtures"].as_array().unwrap() {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().unwrap();
        let options = NativeVaporSfcCompileOptions {
            filename: pack["filename"].as_str().unwrap(),
            source_map: true,
            ..Default::default()
        };
        let compilation = compile_native_vapor_setup_sfc(&arena, source, options);
        let output = compilation.result().unwrap();
        let observation = compilation.observation();
        assert!(core::ptr::eq(
            observation.original().descriptor().source(),
            source
        ));
        let selected = observation.admitted().unwrap();
        let setup = selected.setup();
        let file = setup.file();
        assert!(file.is_complete());
        assert!(core::ptr::eq(file.artifact().source(), source));
        assert!(core::ptr::eq(
            setup.program().source(),
            setup.source().source()
        ));
        let analysis = build_native_selected_setup_vapor_decisions(setup).unwrap();
        assert!(core::ptr::eq(analysis.owner(), setup.owner()));
        let recorded = emit_selected_setup_component::<Recorded>(&analysis, "3.6.0-rc.9").unwrap();
        let unrecorded = emit_selected_setup_component::<NoLinks>(&analysis, "3.6.0-rc.9").unwrap();
        assert_eq!(recorded.text, unrecorded.text);
        assert_eq!(recorded.helpers, unrecorded.helpers);
        assert_eq!(recorded.text.as_str(), output.code());
        let plain = compile_native_vapor_setup_sfc(
            &arena,
            source,
            NativeVaporSfcCompileOptions {
                source_map: false,
                ..options
            },
        );
        assert_eq!(plain.result().unwrap().code(), output.code());
        assert!(plain.result().unwrap().source_map().is_none());
        assert!(plain.result().unwrap().document().links().is_empty());
        assert_eq!(
            output.code(),
            fixture["code"].as_str().unwrap(),
            "{}",
            fixture["id"]
        );
        let map: serde_json::Value = serde_json::from_str(output.source_map().unwrap()).unwrap();
        assert_eq!(map, fixture["map"], "{}", fixture["id"]);
        let links = output.document().links();
        let expected = fixture["links"].as_array().unwrap();
        assert_eq!(links.len(), expected.len(), "{}", fixture["id"]);
        for (link, row) in links.iter().zip(expected) {
            assert_eq!(
                u64::from(link.generated.start),
                row["generated"].as_u64().unwrap()
            );
            assert_eq!(
                u64::from(link.generated.end),
                row["generatedEnd"].as_u64().unwrap()
            );
            assert_eq!(
                u64::from(link.authored.start),
                row["source"].as_u64().unwrap()
            );
            assert_eq!(
                u64::from(link.authored.end),
                row["sourceEnd"].as_u64().unwrap()
            );
            assert_eq!(link.name.as_deref(), row["name"].as_str());
            assert!(link.segment);
            assert!(source.is_char_boundary(link.authored.start as usize));
            assert!(source.is_char_boundary(link.authored.end as usize));
            assert!(
                output
                    .code()
                    .is_char_boundary(link.generated.start as usize)
            );
            assert!(output.code().is_char_boundary(link.generated.end as usize));
        }
        captures.push(serde_json::json!({ "id": fixture["id"], "source": source,
            "code": output.code(), "map": map,
            "roots": analysis.vapor().unwrap().roots().len(),
            "nodes": file.artifact().node_count(), "annotations": setup.type_annotations().count() }));
    }
    assert_eq!(captures.len(), 17);
    if let Some(path) = std::env::var_os("VIZE_NATIVE_VAPOR_SETUP_SFC_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
    }
}

mod vapor_setup_refusals;
