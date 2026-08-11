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

    let (request_uri, documents, _, _) = build_script_virtual_project(
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

#[test]
fn package_script_host_queries_the_importer_mirror_without_rewriting_the_bare_specifier() {
    let project = tempfile::TempDir::new().expect("temp project");
    let host_path = project.path().join("src/Consumer.tsx");
    let package = project.path().join("node_modules/@scope/ui");
    let source = "import Widget from '@scope/ui';\nexport const view = <Widget exact=\"yes\" />;\n";
    std::fs::create_dir_all(host_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        project.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"moduleResolution":"bundler","allowArbitraryExtensions":true,"jsx":"preserve"}}"#,
    )
    .unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@scope/ui","exports":"./Widget.vue"}"#,
    )
    .unwrap();
    std::fs::write(
        package.join("Widget.vue"),
        "<script setup lang=\"ts\">defineProps<{ exact: string }>()</script>\n",
    )
    .unwrap();
    std::fs::write(&host_path, source).unwrap();
    let legacy_request = host_path.with_extension("tsx.jsx.ts");

    let (request_uri, documents, session_root, _) = build_script_virtual_project(
        &host_path,
        legacy_request.to_str().unwrap(),
        source,
        SourceType::tsx(),
        CorsaVueVirtualDocumentOptions::default(),
        &[],
    );
    let request_path = crate::file_uri::file_uri_to_path(&request_uri).unwrap();

    assert_ne!(request_path, legacy_request);
    assert!(session_root.is_some());
    assert!(request_path.is_file());
    assert!(
        request_path
            .parent()
            .unwrap()
            .join("node_modules/@scope/ui/Widget.vue.ts")
            .is_file()
    );
    assert!(
        documents
            .iter()
            .find(|(uri, _)| uri == &request_uri)
            .is_some_and(|(_, code)| code.contains("from '@scope/ui'"))
    );
}
