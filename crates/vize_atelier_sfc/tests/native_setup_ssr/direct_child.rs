//! Whole original direct-child modules and refusal packets; never a substring oracle.
use vize_atelier_sfc::{
    NativeSetupSsrSfcCompileError, NativeSsrSfcCompileOptions, compile_native_setup_ssr_sfc,
};
use vize_l0::Allocator;
use vize_l3::decision::ssr::build_native_selected_setup_ssr_decisions;
use vize_l4::targets::ssr::SsrErrorKind;

fn inputs() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../fixtures/native-setup-ssr-direct-child-sources.json"
    ))
    .unwrap()
}

fn module(fixture: &serde_json::Value) -> serde_json::Value {
    let arena = Allocator::default();
    let source = fixture["source"].as_str().unwrap();
    let filename = fixture["filename"].as_str().unwrap();
    let compilation = compile_native_setup_ssr_sfc(
        &arena,
        source,
        NativeSsrSfcCompileOptions {
            source_map: true,
            filename,
            ..NativeSsrSfcCompileOptions::default()
        },
    );
    let moved = core::hint::black_box(compilation);
    let observation = moved.observation();
    assert!(
        observation.original().issues().is_empty(),
        "{}",
        fixture["name"]
    );
    assert!(core::ptr::eq(
        observation.original().descriptor().source(),
        source
    ));
    let admitted = observation.admitted().unwrap();
    let setup = admitted.setup();
    let file = setup.file();
    assert!(file.is_complete());
    let analysis = build_native_selected_setup_ssr_decisions(setup).unwrap();
    assert!(core::ptr::eq(analysis.file(), file));
    let facts = analysis.ssr().unwrap();
    assert!(facts.unsupported().is_empty());
    assert!(!facts.fragment());
    let provenance = file.artifact().provenance().to_vec();
    let setup_rows = super::rows::setup(setup, &analysis, false);
    assert_eq!(setup_rows["interpolations"].as_array().unwrap().len(), 1);
    if fixture["name"] == "original-nested" {
        super::nested_custody::assert_original(setup, source);
    }
    let output = moved
        .result()
        .unwrap_or_else(|error| panic!("{}: {error:?}", fixture["name"]));
    let map: serde_json::Value = serde_json::from_str(output.source_map().unwrap()).unwrap();
    assert_eq!(map["sourcesContent"], serde_json::json!([source]));
    assert_eq!(map["sources"], serde_json::json!([filename]));
    let links: Vec<_> = output
        .document()
        .links()
        .iter()
        .map(|link| {
            assert!(
                source
                    .get(link.authored.start as usize..link.authored.end as usize)
                    .is_some()
            );
            assert!(
                output
                    .code()
                    .get(link.generated.start as usize..link.generated.end as usize)
                    .is_some()
            );
            serde_json::json!({"authored":{"start":link.authored.start,"end":link.authored.end},
            "generated":{"start":link.generated.start,"end":link.generated.end},
            "name":link.name.as_deref(),"segment":link.segment})
        })
        .collect();
    let plain = compile_native_setup_ssr_sfc(
        &arena,
        source,
        NativeSsrSfcCompileOptions {
            filename,
            ..NativeSsrSfcCompileOptions::default()
        },
    );
    let plain = plain.result().unwrap();
    assert_eq!(plain.code(), output.code());
    assert!(plain.source_map().is_none());
    assert!(plain.document().links().is_empty());
    assert_eq!(file.artifact().provenance(), provenance);
    serde_json::json!({"name":fixture["name"],"source":source,"filename":filename,
        "code":output.code(),"noLinksCode":plain.code(),"mapText":output.source_map().unwrap(),
        "mapValue":map,"links":links,"setup":setup_rows})
}

fn refuse(fixture: &serde_json::Value) -> serde_json::Value {
    let arena = Allocator::default();
    let source = fixture["source"].as_str().unwrap();
    let compilation =
        compile_native_setup_ssr_sfc(&arena, source, NativeSsrSfcCompileOptions::default());
    let error = compilation.result().unwrap_err();
    assert!(core::ptr::eq(
        compilation.observation().original().descriptor().source(),
        source
    ));
    if let Some(expected) = fixture["ssrReason"].as_str() {
        let NativeSetupSsrSfcCompileError::Ssr(error) = error else {
            panic!(
                "{}: genuine target refusal missing: {error:?}",
                fixture["name"]
            )
        };
        let SsrErrorKind::Unsupported(reason) = error.kind else {
            panic!("{}: unsupported target reason missing", fixture["name"])
        };
        assert_eq!(format!("{reason:?}"), expected);
        assert!(
            source
                .get(error.span.start as usize..error.span.end as usize)
                .is_some()
        );
    } else {
        assert!(
            matches!(error, NativeSetupSsrSfcCompileError::Lowering(_)),
            "{}: original source refusal missing: {error:?}",
            fixture["name"]
        );
    }
    serde_json::json!({"name":fixture["name"],"source":source,
        "reason":format!("{error:?}"),"completeModule":false})
}

#[test]
fn whole_direct_child_modules_keep_original_custody_and_final_refusals() {
    let corpus = inputs();
    assert_eq!(corpus["modules"].as_array().unwrap().len(), 8);
    assert_eq!(corpus["refusals"].as_array().unwrap().len(), 15);
    let modules: Vec<_> = corpus["modules"]
        .as_array()
        .unwrap()
        .iter()
        .map(module)
        .collect();
    let refusals: Vec<_> = corpus["refusals"]
        .as_array()
        .unwrap()
        .iter()
        .map(refuse)
        .collect();
    let capture = serde_json::json!({"schema":"vize.native-ssr.direct-child-capture", "complete":true,
        "modules":modules,"refusals":refusals});
    if let Ok(path) = std::env::var("VIZE_NATIVE_SETUP_SSR_CAPTURE") {
        std::fs::write(
            format!("{path}.direct-child.json"),
            serde_json::to_vec_pretty(&capture).unwrap(),
        )
        .unwrap();
    }
    // A pending fixture deliberately fails after the complete actual packet is retained.
    let frozen: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/native-setup-ssr-direct-child-output.json"
    ))
    .unwrap();
    assert_eq!(capture, frozen["capture"]);
}

#[test]
fn equal_source_foreign_setup_bindings_cannot_substitute_for_the_original_owner() {
    let arena_a = Allocator::default();
    let arena_b = Allocator::default();
    let source_a =
        String::from("<script setup>const value=1</script><template><i>{{value}}</i></template>");
    let source_b = source_a.clone();
    let a =
        compile_native_setup_ssr_sfc(&arena_a, &source_a, NativeSsrSfcCompileOptions::default());
    let b =
        compile_native_setup_ssr_sfc(&arena_b, &source_b, NativeSsrSfcCompileOptions::default());
    assert_eq!(a.result().unwrap().code(), b.result().unwrap().code());
    let admitted_a = a.observation().admitted().unwrap();
    let admitted_b = b.observation().admitted().unwrap();
    let setup_a = admitted_a.setup();
    let setup_b = admitted_b.setup();
    let binding_a = setup_a.bindings().next().unwrap();
    let binding_b = setup_b.bindings().next().unwrap();
    assert!(setup_a.binding(binding_a).is_ok());
    assert!(setup_b.binding(binding_b).is_ok());
    assert!(setup_a.binding(binding_b).is_err());
    assert!(setup_b.binding(binding_a).is_err());
    assert!(!binding_a.same_owner(binding_b));
    let vize_l2::file::NativeFileInterpolationState::Admitted(node_a) =
        setup_a.file().native_interpolations()[0].state()
    else {
        panic!("original node")
    };
    let resolution_a = setup_a.file().expression(node_a).unwrap();
    assert!(resolution_a.accepts(binding_a));
    assert!(!resolution_a.accepts(binding_b));
}
