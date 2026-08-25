use std::path::PathBuf;
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
    EditorResponder, StopOnDrop, anchored_position, anchored_range, assert_hover,
    assert_location_range, assert_no_generated_uri, contains_location, definition,
    editor_capabilities, emit_vue_declaration_library, file_uri, hover,
    install_packed_vue_consumer, position_range, references, type_definition, workspace_root,
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
    let app_source = r#"<script setup lang="ts">
export interface PublicProps {
  label: string;
  emojiLabel?: "💥";
}
defineProps<PublicProps>();
</script>
<template>
  <p>{{ label }} {{ emojiLabel }}</p>
</template>
"#;
    let index_source = r#"export { default as App } from "./App.vue";
export type { PublicProps } from "./App.vue";
"#;
    emit_vue_declaration_library(&tsgo, library.path(), app_source, index_source);

    let consumer = tempfile::Builder::new()
        .prefix("content-mapper-declaration-consumer-")
        .tempdir_in(cases_root)
        .unwrap();
    let consumer_source = r#"import { App } from "@scope/emitted-vue";
import type { PublicProps } from "@scope/emitted-vue";
const props: PublicProps = { label: "ok", emojiLabel: "💥" };
type ComponentProps = InstanceType<typeof App>["$props"];
const componentProps: ComponentProps = props;
const labelFromPublicInstance: ComponentProps["label"] = componentProps.label;
const emojiFromPublicInstance: ComponentProps["emojiLabel"] = componentProps.emojiLabel;
void componentProps;
void labelFromPublicInstance;
void emojiFromPublicInstance;
"#;
    let package_root =
        install_packed_vue_consumer(consumer.path(), library.path(), consumer_source);

    let consumer_path = consumer.path().join("src/index.ts");
    let consumer_uri = file_uri(&consumer_path);
    let package_app_path = package_root.join("src/App.vue");
    let package_app_uri = file_uri(&package_app_path);
    let root_uri = file_uri(consumer.path());
    let public_props_import = anchored_position(
        consumer_source,
        "PublicProps",
        "consumer imports PublicProps",
    );
    let public_props_usage_offset = consumer_source
        .match_indices("PublicProps")
        .nth(1)
        .expect("consumer uses PublicProps")
        .0;
    let (public_props_usage, public_props_usage_end) = position_range(
        consumer_source,
        public_props_usage_offset,
        "PublicProps".len(),
    );
    let (public_props_target, public_props_target_end) = anchored_range(
        app_source,
        "PublicProps",
        0,
        "PublicProps".len(),
        "authored Vue source exports PublicProps",
    );
    let (label_property_usage, label_property_usage_end) = anchored_range(
        consumer_source,
        "componentProps.label",
        "componentProps.".len(),
        "label".len(),
        "consumer reads public instance label",
    );
    let props_value_usage = anchored_range(
        consumer_source,
        "= props;",
        2,
        "props".len(),
        "consumer assigns typed props",
    )
    .0;
    let (label_property_target, label_property_target_end) = anchored_range(
        app_source,
        "label: string",
        0,
        "label".len(),
        "authored Vue source declares label prop",
    );
    let (emoji_property_usage, emoji_property_usage_end) = anchored_range(
        consumer_source,
        "componentProps.emojiLabel",
        "componentProps.".len(),
        "emojiLabel".len(),
        "consumer reads public instance emojiLabel",
    );
    let (emoji_property_target, emoji_property_target_end) = anchored_range(
        app_source,
        "emojiLabel?:",
        0,
        "emojiLabel".len(),
        "authored Vue source declares emojiLabel prop",
    );
    let (emoji_template_usage, emoji_template_usage_end) = anchored_range(
        app_source,
        "{{ label }} {{ emojiLabel }}",
        "{{ label }} {{ ".len(),
        "emojiLabel".len(),
        "authored Vue template reads emojiLabel",
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
            // The pinned upstream server returns an empty typeDefinition for
            // imported package types even without content mappers. This lane
            // guards declaration-map source mapping through definition and hover.
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
            let props_value_type_definition =
                type_definition(&client, &consumer_uri, &props_value_usage).await;
            assert_location_range(
                &props_value_type_definition,
                &package_app_uri,
                &public_props_target,
                &public_props_target_end,
                "typed props value typeDefinition",
            );

            let label_property_definition =
                definition(&client, &consumer_uri, &label_property_usage).await;
            assert_location_range(
                &label_property_definition,
                &package_app_uri,
                &label_property_target,
                &label_property_target_end,
                "public instance label property definition",
            );
            let label_property_hover = hover(&client, &consumer_uri, &label_property_usage).await;
            assert_hover(
                &label_property_hover,
                &label_property_usage,
                &label_property_usage_end,
                json!({
                    "kind": "plaintext",
                    "value": "(property) PublicProps.label: string"
                }),
                "public instance label hover",
            );
            let emoji_property_definition =
                definition(&client, &consumer_uri, &emoji_property_usage).await;
            assert_location_range(
                &emoji_property_definition,
                &package_app_uri,
                &emoji_property_target,
                &emoji_property_target_end,
                "public instance emojiLabel property definition",
            );
            let emoji_property_hover = hover(&client, &consumer_uri, &emoji_property_usage).await;
            assert_hover(
                &emoji_property_hover,
                &emoji_property_usage,
                &emoji_property_usage_end,
                json!({
                    "kind": "plaintext",
                    "value": "(property) PublicProps.emojiLabel?: \"💥\" | undefined"
                }),
                "public instance emojiLabel hover",
            );
            let emoji_template_references =
                references(&client, &package_app_uri, &emoji_template_usage).await;
            assert_location_range(
                &emoji_template_references,
                &package_app_uri,
                &emoji_template_usage,
                &emoji_template_usage_end,
                "authored template emojiLabel references",
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
