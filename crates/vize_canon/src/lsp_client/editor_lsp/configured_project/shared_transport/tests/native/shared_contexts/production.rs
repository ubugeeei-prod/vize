//! New hypothesis: actual production editor fallback, never current derived API types.

use super::*;
use crate::lsp_client::CorsaProjectClient;

// Typechecking only: this expression is never executed and remains boolean, not literal true.
const PRODUCTION_NEXT: &str =
    "export const answer: boolean = Math.random() > 0.5;\nanswer.valueOf();\n";

fn publish(client: &mut CorsaProjectClient, a: Option<(&str, &str)>, b: Option<(&str, &str)>) {
    let desired = union(a, b).unwrap();
    let previous = client.document_texts.keys().cloned().collect::<Vec<_>>();
    for uri in previous {
        if !desired.contains_key(uri.as_str()) {
            client.did_close(uri.as_str()).unwrap();
        }
    }
    let mut ordered = desired.iter().collect::<Vec<_>>();
    ordered.sort_unstable_by(|left, right| left.0.cmp(right.0));
    for (uri, text) in ordered {
        if client
            .document_texts
            .get(uri.as_str())
            .is_some_and(|current| current == text)
        {
            continue;
        }
        if client.document_texts.contains_key(uri.as_str()) {
            client.did_change(uri.as_str(), text.as_str()).unwrap();
        } else {
            client.did_open_fast(uri.as_str(), text.as_str()).unwrap();
        }
    }
    client
        .editor_lsp_session()
        .unwrap()
        .ready_workspace_request()
        .unwrap();
    assert_eq!(client.document_texts, desired);
    assert!(client.logical_workspace_overlays);
    assert!(!client.can_use_attached_semantic_api());
    for uri in desired.keys() {
        assert!(!client.can_use_api_for_uri(uri.as_str()));
    }
}

fn production_packet(
    client: &mut CorsaProjectClient,
    name: &str,
    uri: &str,
    config: &Path,
    column: u32,
) -> Value {
    let observed = client.logical_configuration_for_test(uri);
    record(
        cstr!("{name}-editor-project").as_str(),
        &json!({"uri":uri,"expectedConfig":config,"observed":observed}),
    );
    assert_eq!(observed.unwrap(), config);
    assert!(!client.can_use_api_for_uri(uri));
    let documents = client.document_texts.clone();
    let diagnostics = client.diagnostics_via_editor_lsp(uri, &documents);
    record(cstr!("{name}-diagnostics").as_str(), &json!(diagnostics));
    let diagnostics = diagnostics.unwrap();
    let hover = client.hover_raw(uri, 1, 2);
    record(cstr!("{name}-hover").as_str(), &json!(hover));
    let hover = hover.unwrap();
    let completion = client.completion_raw(uri, 1, column);
    record(cstr!("{name}-completion").as_str(), &json!(completion));
    let completion = completion.unwrap();
    let value = json!({"uri":uri,"diagnostics":diagnostics,"hover":hover,"completion":completion});
    record(name, &value);
    value
}

fn config_metadata(
    client: &mut CorsaProjectClient,
    name: &str,
    config: &Path,
    uri: &str,
) -> ProjectSession {
    let current = client.project_session().unwrap();
    let session = open(current.client(), config, uri);
    record(
        name,
        &json!({"scope":"readonly config/snapshot metadata; no current semantic API authority",
        "project":session.project(),"snapshot":session.snapshot().handle,
        "productionCurrentApiAllowed":client.can_use_api_for_uri(uri)}),
    );
    session
}

#[test]
#[ignore = "private production-route feasibility; exact native 7.1 Actions only"]
fn native_two_config_shared_owner_uses_production_editor_fallback() {
    let native = runtime(
        "7.1.0-beta",
        "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
    );
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    fs::create_dir(root.join("a")).unwrap();
    let a_source = root.join("a/source.ts");
    fs::write(&a_source, A_BAD).unwrap();
    let a = root.join("tsconfig.json");
    fs::write(
        &a,
        serde_json::to_vec(&json!({
            "compilerOptions":{"strict":true,"noEmit":true,"types":[],
                "target":"ESNext","module":"ESNext","moduleResolution":"Bundler"},
            "files":["a/source.ts"]
        }))
        .unwrap(),
    )
    .unwrap();
    let b = config(&root, "b", false, B_GOOD);
    let a_config_before = fs::read(&a).unwrap();
    let b_config_before = fs::read(&b).unwrap();
    let a_uri = crate::file_uri::path_to_file_uri(&a_source);
    let b_uri = crate::file_uri::path_to_file_uri(&b.parent().unwrap().join("source.ts"));
    assert!(union(Some((&a_uri, A_BAD)), Some((&a_uri, A_GOOD))).is_err());
    let mut client = CorsaProjectClient::new_for_editor_workspace(native.to_str(), &root).unwrap();
    assert!(client.project_session_attached_to_editor);
    assert_eq!(
        client.project_session().unwrap().client().dialect(),
        Some(ApiDialect::DerivedSnapshots)
    );
    let first_owner = only_owner(&root, &native);
    publish(&mut client, Some((&a_uri, A_BAD)), Some((&b_uri, B_GOOD)));
    let a_metadata = config_metadata(&mut client, "production-a-initial-config", &a, &a_uri);
    let b_metadata = config_metadata(&mut client, "production-b-initial-config", &b, &b_uri);
    record(
        "production-owner",
        &json!({"physicalNativeChildren":1,"pid":first_owner.pid,
        "birth":first_owner.birth,"executable":first_owner.executable,
        "currentSemanticRoute":"actual CorsaProjectClient editor fallback",
        "currentAttachedDerivedApiAllowed":false}),
    );
    let a_bad = production_packet(&mut client, "production-a-error", &a_uri, &a, 7);
    let b_before = production_packet(&mut client, "production-b-before", &b_uri, &b, 6);
    assert_diagnostics(
        &a_bad,
        expected_error(19, "Type 'string' is not assignable to type 'number'."),
    );
    assert_diagnostics(&b_before, clean());
    publish(&mut client, Some((&a_uri, A_NULL)), Some((&b_uri, B_NULL)));
    let a_null_metadata = config_metadata(&mut client, "production-a-null-config", &a, &a_uri);
    let b_null_metadata = config_metadata(&mut client, "production-b-null-config", &b, &b_uri);
    assert_diagnostics(
        &production_packet(&mut client, "production-a-strict-null", &a_uri, &a, 7),
        expected_error(19, "Type 'null' is not assignable to type 'number'."),
    );
    assert_diagnostics(
        &production_packet(&mut client, "production-b-nonstrict-null", &b_uri, &b, 6),
        clean(),
    );
    drop(a_null_metadata);
    drop(b_null_metadata);
    publish(&mut client, Some((&a_uri, A_GOOD)), Some((&b_uri, B_GOOD)));
    let original_api = a_metadata.client().clone();
    let original_snapshot = a_metadata.snapshot_lease();
    let original_project = a_metadata.project().id.clone();
    let original_type = held_type(&original_api, &original_snapshot, &original_project, &a_uri);
    record(
        "production-original-readonly-type",
        &json!({"scope":"original readonly lease, not current editor world",
        "type":original_type,"project":original_project,"snapshot":original_snapshot.handle}),
    );
    assert_eq!(original_type, "number");
    assert_diagnostics(
        &production_packet(&mut client, "production-a-repaired", &a_uri, &a, 7),
        clean(),
    );
    assert_unchanged_semantic_packet(
        &production_packet(&mut client, "production-b-after-a-repair", &b_uri, &b, 6),
        &b_before,
    );
    publish(
        &mut client,
        Some((&a_uri, PRODUCTION_NEXT)),
        Some((&b_uri, B_GOOD)),
    );
    let current = production_packet(&mut client, "production-current-boolean", &a_uri, &a, 7);
    assert_diagnostics(&current, clean());
    assert_eq!(
        current["hover"],
        json!({
            "contents":{"kind":"markdown","value":"```typescript\nconst answer: boolean\n```\n"},
            "range":{"start":{"line":1,"character":0},"end":{"line":1,"character":6}}
        })
    );
    let old_type = held_type(&original_api, &original_snapshot, &original_project, &a_uri);
    record(
        "production-distinct-worlds",
        &json!({"oldReadonlyType":old_type,
        "currentEditorHover":current["hover"],"currentApiAllowed":client.can_use_api_for_uri(&a_uri)}),
    );
    assert_eq!(old_type, "number");
    drop(a_metadata);
    publish(&mut client, None, Some((&b_uri, B_GOOD)));
    let after_detach = production_packet(&mut client, "production-b-after-a-detach", &b_uri, &b, 6);
    assert_unchanged_semantic_packet(&after_detach, &b_before);
    let detached_type = held_type(&original_api, &original_snapshot, &original_project, &a_uri);
    record(
        "production-held-read-after-detach",
        &json!({"type":detached_type,"scope":"retained readonly lease; entered IPC cancellation not claimed"}),
    );
    assert_eq!(detached_type, "number");
    assert_eq!(only_owner(&root, &native), first_owner);
    publish(&mut client, Some((&a_uri, A_GOOD)), Some((&b_uri, B_BAD)));
    let b_current_metadata = config_metadata(&mut client, "production-b-edited-config", &b, &b_uri);
    assert_diagnostics(
        &production_packet(&mut client, "production-b-error", &b_uri, &b, 6),
        expected_error(18, "Type 'number' is not assignable to type 'string'."),
    );
    assert_diagnostics(
        &production_packet(&mut client, "production-a-reopened", &a_uri, &a, 7),
        clean(),
    );
    publish(&mut client, Some((&a_uri, A_GOOD)), Some((&b_uri, B_GOOD)));
    let b_clean = production_packet(&mut client, "production-b-repaired", &b_uri, &b, 6);
    assert_diagnostics(&b_clean, clean());
    assert_unchanged_semantic_packet(&b_clean, &b_before);
    let issued = completion_item(&b_clean, "toUpperCase");
    let resolved = client.completion_resolve_via_editor_lsp(&b_uri, issued.clone());
    record("production-resolve-before-restart", &json!(resolved));
    assert!(resolved.unwrap().is_some());
    drop(original_snapshot);
    drop(original_api);
    drop(b_metadata);
    drop(b_current_metadata);
    assert_eq!(only_owner(&root, &native), first_owner);
    client.shutdown().unwrap();
    control::assert_reaped(&first_owner);
    let mut replacement =
        CorsaProjectClient::new_for_editor_workspace(native.to_str(), &root).unwrap();
    let next_owner = only_owner(&root, &native);
    assert_ne!(next_owner, first_owner);
    publish(
        &mut replacement,
        Some((&a_uri, A_GOOD)),
        Some((&b_uri, B_GOOD)),
    );
    let stale = replacement.completion_resolve_via_editor_lsp(&b_uri, issued);
    record("production-resolve-stale-after-restart", &json!(stale));
    assert_eq!(stale.unwrap(), None);
    let next_a_metadata =
        config_metadata(&mut replacement, "production-a-restart-config", &a, &a_uri);
    let next_b_metadata =
        config_metadata(&mut replacement, "production-b-restart-config", &b, &b_uri);
    assert_diagnostics(
        &production_packet(&mut replacement, "production-a-restart", &a_uri, &a, 7),
        clean(),
    );
    assert_diagnostics(
        &production_packet(&mut replacement, "production-b-restart", &b_uri, &b, 6),
        clean(),
    );
    drop(next_a_metadata);
    drop(next_b_metadata);
    replacement.shutdown().unwrap();
    control::assert_reaped(&next_owner);
    assert_eq!(fs::read(&a_source).unwrap(), A_BAD.as_bytes());
    assert_eq!(
        fs::read(b.parent().unwrap().join("source.ts")).unwrap(),
        B_GOOD.as_bytes()
    );
    assert_eq!(fs::read(&a).unwrap(), a_config_before);
    assert_eq!(fs::read(&b).unwrap(), b_config_before);
}

mod resource;
