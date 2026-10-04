use vize_atelier_sfc::{NativeSsrSfcCompileOptions, compile_native_setup_ssr_sfc};
use vize_l0::Allocator;
use vize_l3::decision::ssr::build_native_selected_setup_ssr_decisions;

#[path = "native_setup_ssr/refusals.rs"]
mod refusals;
#[path = "native_setup_ssr/rows.rs"]
mod rows;

#[test]
fn whole_original_setup_program_and_ssr_reads_keep_full_modules_and_maps() {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native_sfc_setup_ssr_sources.json")).unwrap();
    let capture_path = std::env::var("VIZE_NATIVE_SETUP_SSR_CAPTURE").ok();
    let mut modules = Vec::new();
    for fixture in fixtures["modules"].as_array().unwrap() {
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
            "{}: {:?}",
            fixture["name"],
            observation.original().issues()
        );
        assert!(core::ptr::eq(
            observation.original().descriptor().source(),
            source
        ));
        let admitted = observation.admitted().unwrap();
        assert!(core::ptr::eq(admitted.observation(), observation));
        let setup = admitted.setup();
        let analysis = build_native_selected_setup_ssr_decisions(setup).unwrap();
        assert!(core::ptr::eq(analysis.setup(), setup));
        assert!(core::ptr::eq(
            analysis.owner(),
            observation.original().template().unwrap()
        ));
        let file = setup.file();
        assert!(file.is_complete());
        assert!(core::ptr::eq(analysis.file(), file));
        assert!(core::ptr::eq(file.artifact().source(), source));
        let provenance = file.artifact().provenance().to_vec();
        let setup_rows = rows::setup(setup, &analysis, fixture["handler"] == true);
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
        modules.push(serde_json::json!({"name":fixture["name"],"source":source,
            "filename":filename,"code":output.code(),"noLinksCode":plain.code(),
            "mapText":output.source_map().unwrap(),"mapValue":map,"links":links,"setup":setup_rows}));
        if let Some(path) = &capture_path {
            write(
                &format!("{path}.positive.json"),
                &serde_json::json!({
                "schema":1,"complete":false,"suiteCompletion":"positive_modules_only","modules":&modules}),
            );
        }
    }
    let capture = serde_json::json!({"schema":1,"complete":true,
        "modules":modules,"refusals":refusals::capture()});
    // The whole real packet is durable before the frozen equality can fail.
    if let Some(path) = capture_path {
        write(&path, &capture);
    }
    let frozen_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/native-setup-ssr-output.json"
    );
    let frozen: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(frozen_path)
            .expect("genuine hosted complete setup SSR output has not been frozen"),
    )
    .unwrap();
    assert_eq!(capture, frozen["capture"]);
}

fn write(path: &str, value: &serde_json::Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
