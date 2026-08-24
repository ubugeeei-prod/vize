use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use corsa_lsp::{LspClient, LspSpawnConfig, VirtualDocument, jsonrpc::InboundEvent};
use lsp_types::Uri;
use serde_json::json;

mod content_mapper_lsp_support;
use content_mapper_lsp_support::raw_requests::{
    RawInitialize, RawInitialized, RawSetContentMapperContributions,
};
use content_mapper_lsp_support::{
    EditorResponder, StopOnDrop, assert_no_generated_uri, contains_location, copy_fixture,
    definition, editor_capabilities, file_uri, install_packages, output_text, position,
    pull_diagnostics, workspace_root,
};

const TSGO_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSGO";

#[test]
fn standard_tsgo_lsp_accepts_authored_vue_content_mapper_contribution() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper LSP conformance: {TSGO_ENV} is not set");
        return;
    };
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content_mapper_project");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-lsp-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_packages(project.path());

    let child_path = project.path().join("src/Child.vue");
    let source = std::fs::read_to_string(&child_path).unwrap();
    let child_uri = file_uri(&child_path);
    let root_uri = file_uri(project.path());
    let app_path = project.path().join("src/App.vue");
    let app_source = std::fs::read_to_string(&app_path).unwrap();
    let app_uri = file_uri(&app_path);

    let stop = AtomicBool::new(false);
    let editor = EditorResponder::default();
    std::thread::scope(|scope| {
        let _stop_on_drop = StopOnDrop(&stop);
        corsa::runtime::block_on(async {
            let client = LspClient::spawn(
                LspSpawnConfig::new(&tsgo)
                    .with_cwd(project.path())
                    .with_request_timeout(Some(Duration::from_secs(30))),
            )
            .await
            .unwrap();
            let responder_client = client.clone();
            let events = responder_client.subscribe();
            let stop_ref = &stop;
            let editor_ref = &editor;
            let responder = scope.spawn(move || {
                while !stop_ref.load(Ordering::Relaxed) {
                    if let Ok(InboundEvent::Request { id, method, params }) =
                        events.recv_timeout(Duration::from_millis(50))
                    {
                        let result = editor_ref.respond_to(method.as_str(), &params);
                        let _ = responder_client.respond(id, result);
                    }
                }
            });
            let initialize = client
                .request::<RawInitialize>(json!({
                    "processId": std::process::id(),
                    "rootUri": root_uri,
                    "workspaceFolders": [{ "uri": root_uri, "name": "content-mapper-lsp" }],
                    "capabilities": editor_capabilities(),
                    "initializationOptions": { "runExternalCode": true }
                }))
                .await
                .unwrap();
            assert!(initialize["capabilities"].is_object(), "{initialize:#}");
            client.notify::<RawInitialized>(json!({})).unwrap();

            let contributed = client
                .request::<RawSetContentMapperContributions>(json!({
                    "contributions": [{
                        "contributorId": "vize",
                        "extensions": [".vue"],
                        "inferredProjectContribution": {
                            "options": {},
                            "manifest": {
                                "name": "vize",
                                "exec": [env!("CARGO_BIN_EXE_vize"), "content-mapper"],
                                "compilerOptions": ["noUnusedLocals"]
                            }
                        }
                    }],
                    "openDocuments": [{ "uri": child_uri }, { "uri": app_uri }]
                }))
                .await
                .unwrap();
            assert!(contributed.is_null(), "{contributed:#}");
            editor.assert_vue_did_open_registration();

            let uri = Uri::from_str(&child_uri).unwrap();
            let overlay = client.overlay();
            overlay
                .open(VirtualDocument::new(uri.clone(), "vue", source.as_str()))
                .unwrap();
            let app_document_uri = Uri::from_str(&app_uri).unwrap();
            overlay
                .open(VirtualDocument::new(
                    app_document_uri.clone(),
                    "vue",
                    app_source.as_str(),
                ))
                .unwrap();
            // Keep this exact-upstream LSP lane focused on mapper wiring.
            // The pinned native-preview server accepts the Content Mapper
            // contribution here, while authored Vue diagnostics and symbol
            // behavior are covered by Vize's editor/LSP oracles plus the exact
            // tsgo CLI conformance tests.

            let clean = pull_diagnostics(&client, &child_uri).await;
            assert_eq!(clean["items"], json!([]), "{clean:#}");
            stop.store(true, Ordering::Relaxed);
            client.graceful_close().await.unwrap();
            responder.join().unwrap();
        });
    });
}

#[test]
fn standard_tsgo_lsp_uses_vue_declaration_maps_from_packed_consumer() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper declaration-map consumer: {TSGO_ENV} is not set");
        return;
    };
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let library = tempfile::Builder::new()
        .prefix("content-mapper-declaration-library-")
        .tempdir_in(&cases_root)
        .unwrap();
    install_packages(library.path());
    std::fs::create_dir_all(library.path().join("src")).unwrap();
    let app_source = r#"<script setup lang="ts">
export interface PublicProps {
  label: string;
}

defineProps<PublicProps>();
</script>

<template>
  <p>{{ label }}</p>
</template>
"#;
    let index_source = r#"export { default as App } from "./App.vue";
export type { PublicProps } from "./App.vue";
"#;
    std::fs::write(library.path().join("src/App.vue"), app_source).unwrap();
    std::fs::write(library.path().join("src/index.ts"), index_source).unwrap();
    std::fs::write(
        library.path().join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "declaration": true,
    "declarationMap": true,
    "emitDeclarationOnly": true,
    "rootDir": "src",
    "outDir": "dist"
  },
  "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    let emit = Command::new(&tsgo)
        .current_dir(library.path())
        .args([
            "--runExternalCode",
            "-p",
            "tsconfig.json",
            "--pretty",
            "false",
        ])
        .output()
        .unwrap();
    assert!(emit.status.success(), "{}", output_text(&emit));

    let consumer = tempfile::Builder::new()
        .prefix("content-mapper-declaration-consumer-")
        .tempdir_in(cases_root)
        .unwrap();
    install_packages(consumer.path());
    std::fs::create_dir_all(consumer.path().join("src")).unwrap();
    let consumer_source = r#"import { App } from "@scope/emitted-vue";
import type { PublicProps } from "@scope/emitted-vue";

const props: PublicProps = { label: "ok" };
type ComponentProps = InstanceType<typeof App>["$props"];
const componentProps: ComponentProps = props;
void componentProps;
"#;
    std::fs::write(consumer.path().join("src/index.ts"), consumer_source).unwrap();
    std::fs::write(
        consumer.path().join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();

    let package_root = consumer.path().join("node_modules/@scope/emitted-vue");
    std::fs::create_dir_all(&package_root).unwrap();
    std::fs::write(
        package_root.join("package.json"),
        r#"{
  "name": "@scope/emitted-vue",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "default": "./index.js"
    }
  }
}"#,
    )
    .unwrap();
    copy_fixture(&library.path().join("dist"), &package_root.join("dist"));
    copy_fixture(&library.path().join("src"), &package_root.join("src"));

    let consumer_path = consumer.path().join("src/index.ts");
    let consumer_uri = file_uri(&consumer_path);
    let package_app_path = package_root.join("src/App.vue");
    let package_app_uri = file_uri(&package_app_path);
    let root_uri = file_uri(consumer.path());
    let public_props_position = position(
        consumer_source,
        consumer_source
            .find("PublicProps")
            .expect("consumer imports PublicProps"),
    );
    let public_props_target = position(
        app_source,
        app_source
            .find("PublicProps")
            .expect("authored Vue source exports PublicProps"),
    );

    let stop = AtomicBool::new(false);
    let editor = EditorResponder::default();
    std::thread::scope(|scope| {
        let _stop_on_drop = StopOnDrop(&stop);
        corsa::runtime::block_on(async {
            let client = LspClient::spawn(
                LspSpawnConfig::new(&tsgo)
                    .with_cwd(consumer.path())
                    .with_request_timeout(Some(Duration::from_secs(30))),
            )
            .await
            .unwrap();
            let responder_client = client.clone();
            let events = responder_client.subscribe();
            let stop_ref = &stop;
            let editor_ref = &editor;
            let responder = scope.spawn(move || {
                while !stop_ref.load(Ordering::Relaxed) {
                    if let Ok(InboundEvent::Request { id, method, params }) =
                        events.recv_timeout(Duration::from_millis(50))
                    {
                        let result = editor_ref.respond_to(method.as_str(), &params);
                        let _ = responder_client.respond(id, result);
                    }
                }
            });
            let initialize = client
                .request::<RawInitialize>(json!({
                    "processId": std::process::id(),
                    "rootUri": root_uri,
                    "workspaceFolders": [{ "uri": root_uri, "name": "content-mapper-consumer" }],
                    "capabilities": editor_capabilities(),
                    "initializationOptions": { "runExternalCode": true }
                }))
                .await
                .unwrap();
            assert!(initialize["capabilities"].is_object(), "{initialize:#}");
            client.notify::<RawInitialized>(json!({})).unwrap();
            client
                .request::<RawSetContentMapperContributions>(json!({
                    "contributions": [{
                        "contributorId": "vize",
                        "extensions": [".vue"],
                        "inferredProjectContribution": {
                            "options": {},
                            "manifest": {
                                "name": "vize",
                                "exec": [env!("CARGO_BIN_EXE_vize"), "content-mapper"],
                                "compilerOptions": ["noUnusedLocals"]
                            }
                        }
                    }],
                    "openDocuments": [{ "uri": consumer_uri }, { "uri": package_app_uri }]
                }))
                .await
                .unwrap();

            let overlay = client.overlay();
            let uri = Uri::from_str(&consumer_uri).unwrap();
            overlay
                .open(VirtualDocument::new(uri, "typescript", consumer_source))
                .unwrap();
            let mapped_definition =
                definition(&client, &consumer_uri, &public_props_position).await;
            assert!(
                contains_location(&mapped_definition, &package_app_uri, &public_props_target),
                "definition should use declaration maps to land on authored App.vue:\n{mapped_definition:#}"
            );
            assert_no_generated_uri(&mapped_definition);

            stop.store(true, Ordering::Relaxed);
            client.graceful_close().await.unwrap();
            responder.join().unwrap();
        });
    });
}
