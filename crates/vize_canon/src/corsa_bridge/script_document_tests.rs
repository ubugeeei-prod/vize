use oxc_span::SourceType;

use super::script_document::build_script_virtual_project;
use super::vue_document::CorsaVueVirtualDocumentOptions;
use crate::file_uri::path_to_file_uri;

#[test]
fn script_virtual_project_syncs_vue_dependencies_without_opening_the_sfc() {
    let project = tempfile::TempDir::new().expect("temp project");
    let host_path = project.path().join("Consumer.tsx");
    let child_path = project.path().join("Counter.vue");
    let source = "import Counter from './Counter.vue';\nexport const view = Counter;\n";
    std::fs::write(&host_path, source).expect("host");
    std::fs::write(
        &child_path,
        "<script setup lang=\"ts\">defineProps<{ count: number }>()</script>",
    )
    .expect("child");

    let (request_uri, documents) = build_script_virtual_project(
        &host_path,
        host_path.with_extension("tsx.jsx.ts").to_str().unwrap(),
        source,
        SourceType::ts(),
        CorsaVueVirtualDocumentOptions::default(),
        &[],
    );
    let uris = documents
        .iter()
        .map(|(uri, _)| uri.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        request_uri,
        path_to_file_uri(&host_path.with_extension("tsx.jsx.ts"))
    );
    assert!(uris.contains(&request_uri.as_str()), "{uris:?}");
    assert!(
        uris.contains(&path_to_file_uri(&child_path.with_extension("vue.ts")).as_str()),
        "script hosts must materialize unopened Vue dependencies: {uris:?}",
    );
}
