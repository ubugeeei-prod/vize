//! Real-native completion lifetime around canonical project synchronization.

use super::*;
use crate::ide::{DiagnosticService, IdeContext};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf, process::Command};
use tower_lsp::lsp_types::{
    CompletionItemKind, CompletionResponse, Documentation, MarkupKind,
    TextDocumentContentChangeEvent,
};

const EXPECTED_DETAIL: &str = "(method) header(props: {\n    title: string;\n}): any";
const EXPECTED_MARKDOWN: &str = "```typescript\n(method) header(props: {\n    title: string;\n}): any\n```\n\n**Header** shown above the invoice.\n\n*@param* `props` — Customer-visible heading.\n\n*@example*\n```tsx\nheader({ title: 'Summary' })\n```";

#[test]
#[ignore = "requires preparation of the exact real-host fixtures and native runtime"]
fn exact_host_candidate_resolves_immediately() {
    run("immediate");
}

#[test]
#[ignore = "requires preparation of the exact real-host fixtures and native runtime"]
fn exact_host_candidate_resolves_after_unchanged_same_document_diagnostics() {
    run("same-document");
}

#[test]
#[ignore = "requires preparation of the exact real-host fixtures and native runtime"]
fn exact_host_candidate_resolves_after_unchanged_sibling_diagnostics() {
    run("sibling");
}

#[test]
#[ignore = "requires preparation of the exact real-host fixtures and native runtime"]
fn actual_sibling_edit_refuses_the_previously_issued_candidate() {
    run("edited-sibling");
}

#[test]
#[ignore = "requires preparation of the exact real-host fixtures and native runtime"]
fn actual_dependency_deletion_refuses_the_previously_issued_candidate() {
    run("deleted-dependency");
}

#[test]
#[ignore = "requires preparation of the exact real-host fixtures and native runtime"]
fn identical_reopened_host_document_refuses_the_previously_issued_candidate() {
    run("reopened");
}

fn hash(bytes: &[u8]) -> vize_l0::String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| vize_l0::cstr!("{byte:02x}"))
        .collect()
}

fn run(intervention: &str) {
    let root = PathBuf::from(std::env::var_os("VIZE_COMPLETION_PROBE_ROOT").unwrap())
        .join(intervention)
        .canonicalize()
        .unwrap();
    let config_path = root.join("vize.config.json");
    let original_config = fs::read(&config_path).unwrap();
    let mut config: Value = serde_json::from_slice(&original_config).unwrap();
    let native = PathBuf::from(config["typeChecker"]["corsaPath"].as_str().unwrap())
        .canonicalize()
        .unwrap();
    let version = Command::new(&native).arg("--version").output().unwrap();
    assert!(version.status.success());
    assert_eq!(version.stdout, b"Version 7.0.2\n");
    assert_eq!(version.stderr, b"");
    let (wrapper, traces, shutdown_gate) =
        crate::ide::rename::corsa_session_tests::harness::protocol::traced_corsa_executable(
            &root, &native, false,
        )
        .unwrap();
    assert!(shutdown_gate.is_none());
    config["typeChecker"]["corsaPath"] = json!(wrapper);
    fs::write(&config_path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();

    let slot_path = root.join("src/SlotAuthoring.vue");
    let sibling_path = root.join("src/RichAuthoring.vue");
    let slot_disk = fs::read_to_string(&slot_path).unwrap();
    let sibling_source = fs::read_to_string(&sibling_path).unwrap();
    let corpus = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/lsp/unchanged-sibling-completion");
    for name in ["SlotAuthoring.vue", "RichAuthoring.vue", "InvoiceCard.vue"] {
        assert_eq!(
            fs::read(root.join("src").join(name)).unwrap(),
            fs::read(corpus.join(vize_l0::cstr!("{name}.txt").as_str())).unwrap(),
            "the complete historical authored source is the regression input"
        );
    }
    let slot_source = slot_disk.replacen("ownSlots.header", "ownSlots.", 1);
    assert_ne!(slot_source, slot_disk);
    assert_eq!(
        hash(slot_source.as_bytes()).as_str(),
        "0f4735447304a3461db848f93325a07784bc91669d2cec7356eb902a9aabe670"
    );
    let slot_uri = Url::from_file_path(&slot_path).unwrap();
    let sibling_uri = Url::from_file_path(&sibling_path).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.clone());
    state.load_workspace_config(&root);
    state.apply_lsp_initialization_options(Some(&json!({
        "editor": true, "typecheck": true, "lint": false, "ecosystem": false,
    })));
    for (uri, source, version) in [
        (&sibling_uri, sibling_source.as_str(), 5),
        (&slot_uri, slot_source.as_str(), 6),
    ] {
        state
            .documents
            .open(uri.clone(), source.into(), version, "vue".into());
        state.update_virtual_docs(uri, source);
    }
    let slot_revision = state.documents.get(&slot_uri).unwrap().revision();
    let sibling_revision = state.documents.get(&sibling_uri).unwrap().revision();
    let mut receipt = json!({
        "schemaVersion": 1,
        "intervention": intervention,
        "sourceCommit": std::env::var("GITHUB_SHA").ok(),
        "native": {"path":native,"sha256":hash(&fs::read(&native).unwrap()),
            "stdout":String::from_utf8(version.stdout).unwrap()},
        "slot": {"uri":slot_uri,"disk":slot_disk,"source":slot_source,
            "version":6,"revision":slot_revision,"sha256":hash(slot_source.as_bytes())},
        "sibling": {"uri":sibling_uri,"source":sibling_source,
            "version":5,"revision":sibling_revision,"sha256":hash(sibling_source.as_bytes())},
        "originalConfig":serde_json::from_slice::<Value>(&original_config).unwrap(),
        "tsconfig":serde_json::from_slice::<Value>(&fs::read(root.join("tsconfig.json")).unwrap()).unwrap(),
        "vuePackage":serde_json::from_slice::<Value>(&fs::read(root.join("node_modules/vue/package.json")).unwrap()).unwrap(),
        "protocolTraces":traces,
    });
    let save = |receipt: &Value| {
        fs::write(
            root.join("receipt.json"),
            serde_json::to_vec_pretty(receipt).unwrap(),
        )
        .unwrap();
    };
    save(&receipt);
    crate::runtime::block_on(async {
        let bridge = state.get_corsa_bridge().await.expect("real native bridge");
        // Warm Rich first. Slot then becomes a new member of that same Canon
        // project before the candidate is issued. The sibling case performs
        // only an unchanged diagnostic; explicit mutation controls are separate.
        receipt["warmSiblingDiagnostics"] =
            json!(DiagnosticService::collect_async(&state, &sibling_uri).await);
        receipt["warmSlotDiagnostics"] =
            json!(DiagnosticService::collect_async(&state, &slot_uri).await);
        save(&receipt);
        let offset = slot_source.find("ownSlots.").unwrap() + "ownSlots.".len();
        let context = IdeContext::new(&state, &slot_uri, offset).unwrap();
        let response = CompletionService::complete_with_corsa(&context, Some(bridge.clone()))
            .await
            .expect("completion response");
        let items = match response {
            CompletionResponse::Array(items) => items,
            CompletionResponse::List(list) => list.items,
        };
        receipt["issuedItems"] = json!(items);
        save(&receipt);
        let selected = items
            .iter()
            .position(|item| item.label == "header")
            .unwrap();
        let original = items[selected].clone();
        receipt["selectedIndex"] = json!(selected);
        receipt["originalItem"] = json!(original);
        save(&receipt);
        let diagnostics = match intervention {
            "immediate" => vec![],
            "same-document" => DiagnosticService::collect_async(&state, &slot_uri).await,
            "sibling" => DiagnosticService::collect_async(&state, &sibling_uri).await,
            "edited-sibling" => {
                let edited = sibling_source.replace("amount: number", "amount: string");
                assert_ne!(edited, sibling_source);
                assert!(state.documents.apply_changes(
                    &sibling_uri,
                    vec![TextDocumentContentChangeEvent {
                        range: None,
                        range_length: None,
                        text: edited.clone(),
                    }],
                    6
                ));
                state.update_virtual_docs(&sibling_uri, &edited);
                receipt["editedSiblingSource"] = json!(edited);
                DiagnosticService::collect_async(&state, &sibling_uri).await
            }
            "deleted-dependency" => {
                let deleted = root.join("src/InvoiceCard.vue");
                fs::remove_file(&deleted).unwrap();
                // Apply the same source retirement and deferred disk dirty bit
                // used by the watched-file/delete handlers, then collect once.
                state.forget_workspace_vue_files(Url::from_file_path(&deleted).unwrap().as_str());
                state.invalidate_batch_cache();
                state.retire_corsa_bridge_for_deleted_vue_files();
                state.mark_corsa_disk_state_dirty();
                receipt["deletedDependency"] = json!(deleted);
                DiagnosticService::collect_async(&state, &sibling_uri).await
            }
            "reopened" => {
                state.close_document(&slot_uri);
                state
                    .documents
                    .open(slot_uri.clone(), slot_source.clone(), 6, "vue".into());
                state.update_virtual_docs(&slot_uri, &slot_source);
                vec![]
            }
            _ => panic!("unknown completion intervention: {intervention}"),
        };
        receipt["interveningDiagnostics"] = json!(diagnostics);
        save(&receipt);
        let resolved = CompletionService::resolve(&state, original.clone()).await;
        receipt["resolvedItem"] = json!(resolved);
        receipt["slotRevisionAfter"] = json!(state.documents.get(&slot_uri).unwrap().revision());
        receipt["siblingRevisionAfter"] =
            json!(state.documents.get(&sibling_uri).unwrap().revision());
        save(&receipt);
        let current = state
            .get_corsa_bridge()
            .await
            .expect("current native bridge");
        bridge.shutdown().await.unwrap();
        if !std::sync::Arc::ptr_eq(&bridge, &current) {
            current.shutdown().await.unwrap();
        }

        assert_eq!(state.documents.text(&slot_uri).unwrap(), slot_source);
        if intervention == "edited-sibling" {
            assert_eq!(
                state.documents.text(&sibling_uri).unwrap(),
                sibling_source.replace("amount: number", "amount: string")
            );
            assert_ne!(
                state.documents.get(&sibling_uri).unwrap().revision(),
                sibling_revision
            );
        } else {
            assert_eq!(state.documents.text(&sibling_uri).unwrap(), sibling_source);
            assert_eq!(
                state.documents.get(&sibling_uri).unwrap().revision(),
                sibling_revision
            );
        }
        if intervention == "reopened" {
            assert_ne!(
                state.documents.get(&slot_uri).unwrap().revision(),
                slot_revision
            );
        } else {
            assert_eq!(
                state.documents.get(&slot_uri).unwrap().revision(),
                slot_revision
            );
        }
        assert_eq!(fs::read_to_string(&slot_path).unwrap(), slot_disk);
        assert_eq!(fs::read_to_string(&sibling_path).unwrap(), sibling_source);
        if matches!(
            intervention,
            "edited-sibling" | "deleted-dependency" | "reopened"
        ) {
            assert_eq!(original.detail, None);
            assert_eq!(original.documentation, None);
            assert_eq!(
                resolved, original,
                "an obsolete item must remain wholly unresolved"
            );
            return;
        }
        assert_eq!(resolved.label, "header");
        assert_eq!(resolved.kind, Some(CompletionItemKind::METHOD));
        assert_eq!(resolved.detail.as_deref(), Some(EXPECTED_DETAIL));
        let Some(Documentation::MarkupContent(documentation)) = &resolved.documentation else {
            panic!("{intervention}: exact issued header lost Markdown: {resolved:#?}");
        };
        assert_eq!(documentation.kind, MarkupKind::Markdown);
        assert_eq!(documentation.value, EXPECTED_MARKDOWN);
        let mut insertion = resolved;
        insertion.detail = original.detail.clone();
        insertion.documentation = original.documentation.clone();
        assert_eq!(insertion, original);
    });
}
