use serde_json::json;
use sha2::{Digest, Sha256};
use vize_l0::profiler::{Metrics, global_profiler};

use crate::{
    ContentMapperProjectConfiguration, generate_vue_content_mapper_transform_with_project_context,
};

fn observed(metrics: &Metrics) -> serde_json::Value {
    json!({
        "calls": metrics.count,
        "totalNs": metrics.total_duration.as_nanos(),
        "selfNs": metrics.self_duration.as_nanos(),
        "minNs": metrics.min_duration.as_nanos(),
        "maxNs": metrics.max_duration.as_nanos(),
    })
}

/// Run alone so unrelated tests never share the profiler's observation window.
#[test]
#[ignore = "isolated source profiling campaign; explicitly executed by Content Mapper Conformance"]
fn typed_router_context_admission_is_outside_repeated_projection_work() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    let config = root.join("tsconfig.json");
    std::fs::write(
        &config,
        r#"{"compilerOptions":{"moduleResolution":"Bundler"},"vueCompilerOptions":{"plugins":["vue-router/volar/sfc-typed-router"]}}"#,
    )
    .unwrap();
    let fixture = &super::super::content_mapper_config::typed_router_tests::provider::PAGE;
    let source = fixture.to_string();
    super::super::content_mapper_config::typed_router_tests::provider::install(&root, "preview");
    let file = root.join("src/pages/[id].vue");
    let compiler = json!({"rootDir":root,"moduleResolution":100});
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    let (configuration, _) = ContentMapperProjectConfiguration::load_with_compiler_options(
        &config,
        compiler.as_object().unwrap(),
    )
    .unwrap();
    let admission_reads = profiler
        .get("canon.typed_router.provider.read")
        .unwrap()
        .count;
    assert!(admission_reads > 0);
    std::fs::remove_file(config).unwrap();
    std::fs::remove_dir_all(root.join("node_modules/vue-router")).unwrap();
    let mut first = None;
    let mut last = None;
    for _ in 0..64 {
        let output = generate_vue_content_mapper_transform_with_project_context(
            &file,
            &source,
            configuration.transform_options,
            &configuration.project_context,
        )
        .unwrap();
        if first.is_none() {
            first = Some(output);
        } else {
            last = Some(output);
        }
    }
    profiler.disable();
    assert_eq!(
        profiler
            .get("canon.typed_router.provider.admission")
            .unwrap()
            .count,
        1
    );
    assert_eq!(
        profiler
            .get("canon.typed_router.provider.read")
            .unwrap()
            .count,
        admission_reads
    );
    let configuration = profiler
        .get("canon.content_mapper.project_configuration")
        .unwrap();
    let projection = profiler.get("canon.content_mapper.transform").unwrap();
    let phases: serde_json::Value = profiler
        .summary()
        .entries
        .iter()
        .map(|entry| {
            json!({
                "name":entry.name,
                "calls":entry.count,
                "totalNs":entry.total.as_nanos(),
                "selfNs":entry.self_total.as_nanos(),
                "childNs":entry.child_total.as_nanos(),
                "approxP95Ns":entry.p95.as_nanos(),
            })
        })
        .collect();
    let report = json!({
        "schema":"vize.content-mapper.router-source-profile",
        "version":1,
        "scope":"filesystem provider observations and file-only projection; native provider authority excluded",
        "sourceSha256":super::super::identity::encode_digest(Sha256::digest(source.as_bytes())),
        "projectConfiguration":observed(&configuration),
        "projection":observed(&projection),
        "providerObservationCalls":1,
        "providerObservationReads":admission_reads,
        "providerReadsDuringTransforms":0,
        "phases":phases,
    });
    let report = serde_json::to_string_pretty(&report).unwrap();
    if let Some(path) = std::env::var_os("VIZE_TEST_ROUTER_PROFILE_OUTPUT") {
        std::fs::write(path, &report).unwrap();
    }
    eprintln!("{report}");
    assert_eq!(configuration.count, 1);
    assert_eq!(projection.count, 64);
    assert_eq!(
        serde_json::to_value(first.unwrap()).unwrap(),
        serde_json::to_value(last.unwrap()).unwrap()
    );
}
