use std::path::Path;

use serde_json::json;
use sha2::{Digest, Sha256};
use vize_l0::profiler::{Metrics, global_profiler};

use crate::{
    ContentMapperProjectConfiguration, generate_vue_content_mapper_transform_with_project_options,
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
        r#"{"vueCompilerOptions":{"plugins":["vue-router/volar/sfc-typed-router"]}}"#,
    )
    .unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../tests/fixtures/typechecker/experimental-router-5.4/project/src/pages/products/[id=int].vue.txt",
    );
    let source = std::fs::read_to_string(fixture).unwrap();
    let file = root.join("src/pages/products/[id=int].vue");
    let compiler = json!({"rootDir":root});
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    let (configuration, typed_router_root) =
        ContentMapperProjectConfiguration::load_with_compiler_options(
            &config,
            compiler.as_object().unwrap(),
        )
        .unwrap();
    std::fs::remove_file(config).unwrap();
    let mut first = None;
    let mut last = None;
    for _ in 0..64 {
        let output = generate_vue_content_mapper_transform_with_project_options(
            &file,
            &source,
            configuration.transform_options,
            typed_router_root.as_deref(),
        )
        .unwrap();
        if first.is_none() {
            first = Some(output);
        } else {
            last = Some(output);
        }
    }
    profiler.disable();
    let configuration = profiler
        .get("canon.content_mapper.project_configuration")
        .unwrap();
    let projection = profiler.get("canon.content_mapper.transform").unwrap();
    let report = json!({
        "schema":"vize.content-mapper.router-source-profile",
        "version":1,
        "scope":"projection only; provider type checking and process startup excluded",
        "sourceSha256":super::super::identity::encode_digest(Sha256::digest(source.as_bytes())),
        "projectConfiguration":observed(&configuration),
        "projection":observed(&projection),
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
