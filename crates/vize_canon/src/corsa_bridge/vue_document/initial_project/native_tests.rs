//! Actual native answers after an intervening mutation must restore the graph.
#![expect(clippy::disallowed_types, reason = "native test owns source packets")]

use std::io::Write;
use std::sync::atomic::Ordering;

use davinci_test_support::original_diagnosing_process as control;

use super::super::CorsaVueVirtualDocumentOptions;
use super::{CorsaBridge, CorsaVueWorkspaceRequest};
use crate::corsa_bridge::CorsaBridgeConfig;
use crate::virtual_ts::VirtualTsOptions;

#[test]
fn intervening_native_operations_restore_exact_initial_graph() {
    if !control::enabled() {
        return;
    }
    let native = control::runtime();
    for operation in ["none", "metadata", "otherOpen", "otherConfig"] {
        let root = tempfile::tempdir().unwrap();
        let authored = root.path().canonicalize().unwrap();
        let config = r#"{"compilerOptions":{"strict":true,"target":"ES2020","module":"ESNext","moduleResolution":"Bundler","types":[]},"include":["**/*.ts","**/*.vue"]}"#;
        std::fs::write(authored.join("tsconfig.json"), config).unwrap();
        let support = "export const support: number = 42;\n";
        std::fs::write(authored.join("support.ts"), support).unwrap();
        let source = "<script setup lang=\"ts\">import { support } from './support'; const answer = support;</script><template>{{ answer }}</template>";
        let source_path = authored.join("App.vue");
        let bridge = CorsaBridge::with_config_and_package_routes(
            CorsaBridgeConfig {
                corsa_path: Some(native.clone()),
                working_dir: Some(authored.clone()),
                ..Default::default()
            },
            crate::PackageRouteResolver::default(),
        );
        let options = CorsaVueVirtualDocumentOptions::default();
        let seed = bridge
            .prepare_initial_vue_project(CorsaVueWorkspaceRequest {
                source_path: source_path.clone(),
                content: source.into(),
                options,
                overlays: Vec::new(),
                virtual_ts_options: VirtualTsOptions::default(),
                requested_sources: Vec::new(),
            })
            .unwrap()
            .unwrap();
        let receipt =
            corsa::runtime::block_on(bridge.spawn_with_initial_project(seed, || true)).unwrap();
        let support_uri = receipt
            .project
            .documents
            .iter()
            .find(|(_, text)| text.as_str() == support)
            .unwrap()
            .0
            .clone();
        let initial_epoch = receipt.operation_epoch;
        let other = tempfile::tempdir().unwrap();
        match operation {
            "none" => {}
            "metadata" => {
                let selected = corsa::runtime::block_on(bridge.with_client(|client| {
                    client
                        .initial_project_config_for_test()
                        .map_err(crate::corsa_bridge::CorsaBridgeError::CommunicationError)
                }))
                .unwrap();
                assert_eq!(
                    std::path::Path::new(&selected),
                    receipt.project.session_config_path.as_ref().unwrap()
                );
            }
            "otherOpen" => {
                corsa::runtime::block_on(bridge.open_virtual_document(
                    &support_uri,
                    "export const support: string = 'changed';\n",
                ))
                .unwrap();
            }
            "otherConfig" => {
                let other_root = other.path().canonicalize().unwrap();
                std::fs::write(other_root.join("tsconfig.json"), config).unwrap();
                std::fs::write(other_root.join("other.ts"), "export {};\n").unwrap();
                corsa::runtime::block_on(bridge.with_client(move |client| {
                    client
                        .synchronize_materialized_project(
                            &other_root,
                            Some(&other_root.join("tsconfig.json")),
                            &Default::default(),
                        )
                        .map_err(crate::corsa_bridge::CorsaBridgeError::CommunicationError)
                }))
                .unwrap();
            }
            _ => unreachable!(),
        }
        let before_consume = bridge.client_operation_epoch.load(Ordering::SeqCst);
        let document = corsa::runtime::block_on(bridge.open_initial_vue_project(
            receipt,
            &source_path,
            source,
            options,
            &[],
            &VirtualTsOptions::default(),
            &[],
        ))
        .unwrap();
        let after_consume = bridge.client_operation_epoch.load(Ordering::SeqCst);
        let query_uri = support_uri.clone();
        let actual = corsa::runtime::block_on(bridge.with_client(move |client| {
            client
                .hover_raw(&query_uri, 0, 13)
                .map_err(crate::corsa_bridge::CorsaBridgeError::CommunicationError)
        }))
        .unwrap();
        let expected = serde_json::json!({
            "contents":{"kind":"markdown","value":"```typescript\nconst support: number\n```\n"},
            "range":{"start":{"line":0,"character":13},"end":{"line":0,"character":20}}
        });
        let packet = serde_json::json!({"operation":operation,"native":native,"initialEpoch":initial_epoch,"beforeConsume":before_consume,"afterConsume":after_consume,"documentUri":document.request_uri,"supportUri":support_uri,"actual":actual,"expected":expected});
        std::io::stdout()
            .write_all(vize_carton::cstr!("INITIAL_CANON_OPERATION {}\n", packet).as_bytes())
            .unwrap();
        assert_eq!(actual, Some(expected));
        if operation == "none" {
            assert_eq!(before_consume, initial_epoch);
            assert_eq!(
                after_consume,
                before_consume + 1,
                "only the queue-owned reuse probe runs"
            );
        } else {
            assert!(before_consume > initial_epoch);
            assert!(
                after_consume >= before_consume + 2,
                "intervening native state requires ordinary synchronization"
            );
        }
        corsa::runtime::block_on(bridge.shutdown()).unwrap();
    }
}
