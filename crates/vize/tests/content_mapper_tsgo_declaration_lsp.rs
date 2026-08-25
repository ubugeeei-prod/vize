use std::path::PathBuf;
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
    definition, editor_capabilities, file_uri, hover, install_packages, output_text, position,
    type_definition, workspace_root,
};

const TSGO_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSGO";

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
    let public_props_import = position(
        consumer_source,
        consumer_source
            .find("PublicProps")
            .expect("consumer imports PublicProps"),
    );
    let public_props_usage_offset = consumer_source
        .match_indices("PublicProps")
        .nth(1)
        .expect("consumer uses PublicProps")
        .0;
    let public_props_usage = position(consumer_source, public_props_usage_offset);
    let public_props_usage_end = position(
        consumer_source,
        public_props_usage_offset + "PublicProps".len(),
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
            assert_maps_to_app(
                definition(&client, &consumer_uri, &public_props_import).await,
                &package_app_uri,
                &public_props_target,
                "definition",
            );
            assert_maps_to_app(
                type_definition(&client, &consumer_uri, &public_props_usage).await,
                &package_app_uri,
                &public_props_target,
                "typeDefinition",
            );
            let mapped_hover = hover(&client, &consumer_uri, &public_props_usage).await;
            assert_no_generated_uri(&mapped_hover);
            assert_eq!(
                mapped_hover["contents"],
                json!({
                    "kind": "plaintext",
                    "value": "(alias) interface PublicProps"
                }),
                "hover should describe the authored Vue public props type:\n{mapped_hover:#}"
            );
            assert_eq!(
                mapped_hover["range"],
                json!({
                    "start": public_props_usage,
                    "end": public_props_usage_end
                }),
                "hover range should cover the consumer PublicProps usage:\n{mapped_hover:#}"
            );

            stop.store(true, Ordering::Relaxed);
            client.graceful_close().await.unwrap();
            responder.join().unwrap();
        });
    });
}

fn assert_maps_to_app(
    response: serde_json::Value,
    package_app_uri: &str,
    public_props_target: &serde_json::Value,
    label: &str,
) {
    assert!(
        contains_location(&response, package_app_uri, public_props_target),
        "{label} should use declaration maps to land on authored App.vue:\n{response:#}"
    );
    assert_no_generated_uri(&response);
}
