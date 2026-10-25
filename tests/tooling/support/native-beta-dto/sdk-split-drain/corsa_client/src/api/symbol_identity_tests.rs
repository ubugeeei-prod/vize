use super::{FileMemo, RequestScope, SymbolIdentity, UNKNOWN_SYMBOL_OWNER};
use crate::{ApiClient, CorsaError};
use serde_json::{Value, json};

const PROJECT: &str = "/workspace/tsconfig.json";

fn descriptor(node_id: &str) -> Value {
    json!({
        "fileName": "/workspace/src/main.ts",
        "path": "/workspace/src/main.ts",
        "contentHash": "e124f257e46e948e015e80de9a4d5942",
        "parseOptionsKey": "0",
        "scriptKind": 3,
        "nodeId": node_id,
    })
}

fn file_symbol(id: u64, node_id: &str, name: &str) -> Value {
    json!({
        "reference": { "kind": 0, "file": descriptor(node_id), "id": id },
        "name": name,
        "flags": 2,
        "checkFlags": 0,
    })
}

fn snapshot_symbol(id: u64, snapshot: u64, name: &str) -> Value {
    json!({
        "reference": { "kind": 1, "snapshot": snapshot, "project": PROJECT, "id": id },
        "name": name,
        "flags": 33554432,
        "checkFlags": 0,
    })
}

fn scope(snapshot: u64) -> RequestScope {
    RequestScope::from_params(&json!({ "snapshot": snapshot, "project": PROJECT }))
}

fn adopt(identity: &SymbolIdentity, mut response: Value, scope: &RequestScope) -> Value {
    identity.adopt(&mut response, scope);
    response
}

fn encode(identity: &SymbolIdentity, mut params: Value) -> crate::Result<Value> {
    identity.encode_params(&mut params)?;
    Ok(params)
}

#[test]
fn file_owned_symbol_becomes_a_short_handle_that_encodes_back_to_its_reference() {
    let identity = SymbolIdentity::default();

    let symbol = adopt(&identity, file_symbol(6, "142", "answer"), &scope(1));

    assert_eq!(symbol.get("reference"), None);
    assert_eq!(symbol["name"], json!("answer"));
    assert_eq!(symbol["id"], json!(r#"{"file":"142","id":6}"#));
    let params = encode(
        &identity,
        json!({ "snapshot": 1, "project": PROJECT, "symbol": symbol["id"].clone() }),
    )
    .unwrap();
    assert_eq!(
        params["symbol"],
        json!({ "kind": 0, "file": descriptor("142"), "id": 6 })
    );
}

#[test]
fn snapshot_owned_symbol_is_completed_with_the_project_of_the_request() {
    let identity = SymbolIdentity::default();

    let symbol = adopt(&identity, snapshot_symbol(9, 3, "Promise"), &scope(3));

    assert_eq!(symbol["id"], json!(r#"{"id":9,"snapshot":3}"#));
    let params = encode(
        &identity,
        json!({ "snapshot": 3, "project": "/other/tsconfig.json", "symbols": [symbol["id"].clone()] }),
    )
    .unwrap();
    assert_eq!(
        params["symbols"],
        json!([{ "kind": 1, "snapshot": 3, "project": "/other/tsconfig.json", "id": 9 }])
    );
}

#[test]
fn a_mention_is_the_same_handle_as_the_symbol_itself() {
    let identity = SymbolIdentity::default();
    let file_owned = adopt(&identity, file_symbol(6, "142", "Greeter"), &scope(1));
    let checker_owned = adopt(&identity, snapshot_symbol(9, 1, "Promise"), &scope(1));

    let mentions = adopt(
        &identity,
        json!([
            { "id": 87, "flags": 524288, "symbol": { "id": 6, "file": "142" } },
            { "id": 40, "flags": 524288, "symbol": { "id": 9 } },
        ]),
        &scope(1),
    );

    assert_eq!(mentions[0]["symbol"], file_owned["id"]);
    assert_eq!(mentions[1]["symbol"], checker_owned["id"]);
}

#[test]
fn reference_missing_what_its_owner_kind_requires_is_handled_conservatively() {
    let identity = SymbolIdentity::default();
    let unowned = json!({ "reference": { "kind": 1, "id": 9 }, "name": "Promise" });
    let fileless = json!({ "reference": { "kind": 0, "id": 6 }, "name": "answer" });

    // A snapshot-owned reference falls back to the snapshot the request named,
    // so it is still the handle a mention renders to.
    let adopted = adopt(&identity, unowned, &scope(3));
    assert_eq!(adopted["id"], json!(r#"{"id":9,"snapshot":3}"#));

    // A file-owned reference without its file cannot be completed later, so it
    // is not turned into a handle that would pretend otherwise.
    assert_eq!(adopt(&identity, fileless.clone(), &scope(3)), fileless);
}

#[test]
fn signature_parameters_become_handles_in_order() {
    let identity = SymbolIdentity::default();

    let signature = adopt(
        &identity,
        json!({
            "id": 5,
            "flags": 0,
            "parameters": [{ "id": 21 }, { "id": 20, "file": "142" }],
            "thisParameter": { "id": 22 },
        }),
        &scope(1),
    );

    assert_eq!(
        signature["parameters"],
        json!([r#"{"id":21,"snapshot":1}"#, r#"{"file":"142","id":20}"#])
    );
    assert_eq!(
        signature["thisParameter"],
        json!(r#"{"id":22,"snapshot":1}"#)
    );
}

#[test]
fn mention_of_a_file_never_seen_in_full_is_a_stale_handle() {
    let identity = SymbolIdentity::default();
    let mention = adopt(
        &identity,
        json!({ "id": 87, "flags": 524288, "symbol": { "id": 6, "file": "142" } }),
        &scope(1),
    );

    let error = encode(&identity, json!({ "symbol": mention["symbol"].clone() })).unwrap_err();

    assert!(error.to_string().contains(UNKNOWN_SYMBOL_OWNER), "{error}");
    assert!(ApiClient::is_stale_handle_error(&error), "{error}");

    // Any full reference into that file is enough to complete the handle.
    adopt(&identity, file_symbol(11, "142", "other"), &scope(1));
    let params = encode(&identity, json!({ "symbol": mention["symbol"].clone() })).unwrap();
    assert_eq!(
        params["symbol"],
        json!({ "kind": 0, "file": descriptor("142"), "id": 6 })
    );
}

#[test]
fn file_handle_outlives_the_snapshot_that_produced_it() {
    let identity = SymbolIdentity::default();
    let symbol = adopt(&identity, file_symbol(6, "142", "answer"), &scope(1));

    let params = encode(
        &identity,
        json!({ "snapshot": 2, "project": PROJECT, "symbol": symbol["id"].clone() }),
    )
    .unwrap();

    assert_eq!(params["symbol"]["kind"], json!(0));
    assert_eq!(params["symbol"].get("snapshot"), None);
}

#[test]
fn snapshot_handle_keeps_naming_the_snapshot_that_owns_it() {
    let identity = SymbolIdentity::default();
    let symbol = adopt(&identity, snapshot_symbol(9, 3, "Promise"), &scope(3));

    let params = encode(
        &identity,
        json!({ "snapshot": 4, "project": PROJECT, "symbol": symbol["id"].clone() }),
    )
    .unwrap();

    // The runtime is the one to say a snapshot-owned symbol went stale.
    assert_eq!(params["symbol"]["snapshot"], json!(3));
}

#[test]
fn snapshot_owned_symbol_without_a_project_is_left_for_the_runtime_to_reject() {
    let params = encode(
        &SymbolIdentity::default(),
        json!({ "snapshot": 3, "symbol": r#"{"id":9,"snapshot":3}"# }),
    )
    .unwrap();

    assert_eq!(
        params["symbol"],
        json!({ "kind": 1, "snapshot": 3, "project": "", "id": 9 })
    );
}

#[test]
fn mentions_outside_the_typed_api_pass_through_as_upstream_sent_them() {
    let identity = SymbolIdentity::default();
    let mut symbol = file_symbol(6, "142", "answer");
    symbol["parent"] = json!({ "id": 7, "file": "142" });
    symbol["exportSymbol"] = json!({ "id": 8, "file": "142" });

    let symbol = adopt(&identity, symbol, &scope(1));
    let aliased = adopt(
        &identity,
        json!({ "id": 87, "flags": 524288, "aliasSymbol": { "id": 12, "file": "999" } }),
        &scope(1),
    );

    assert_eq!(symbol["parent"], json!({ "id": 7, "file": "142" }));
    assert_eq!(symbol["exportSymbol"], json!({ "id": 8, "file": "142" }));
    assert_eq!(aliased["aliasSymbol"], json!({ "id": 12, "file": "999" }));
}

#[test]
fn values_that_only_resemble_symbols_are_left_alone() {
    let response = json!({
        // No `flags`: not a type or signature, so `symbol` is someone else's field.
        "id": 1,
        "symbol": { "id": 6, "file": "142" },
        "nested": {
            "id": 2,
            "flags": 1,
            "symbol": { "id": 6, "file": "142", "extra": true },
            "parameters": [{ "id": 1 }, "not-a-mention"],
        },
        "reference": "not-a-reference",
    });

    assert_eq!(
        adopt(&SymbolIdentity::default(), response.clone(), &scope(1)),
        response
    );
}

#[test]
fn well_known_symbol_ids_become_handles_owned_by_the_request_snapshot() {
    let mut response = json!({ "unknown": 3, "undefined": 4, "arguments": 5 });

    SymbolIdentity::adopt_well_known_symbols(&mut response, &scope(2));

    assert_eq!(response["undefined"], json!(r#"{"id":4,"snapshot":2}"#));
}

#[test]
fn encode_leaves_handles_from_other_dialects_untouched() {
    let params = json!({
        "snapshot": "1",
        "symbol": "5",
        "symbols": ["6", "s0000000000000001"],
        "type": "87",
    });

    assert_eq!(
        encode(&SymbolIdentity::default(), params.clone()).unwrap(),
        params
    );
}

#[test]
fn encode_reaches_symbols_inside_batched_sub_requests() {
    let params = encode(
        &SymbolIdentity::default(),
        json!({
            "requests": [{
                "method": "getTypeOfSymbol",
                "params": { "snapshot": 3, "project": PROJECT, "symbol": r#"{"id":9}"# },
            }],
        }),
    )
    .unwrap();

    // The sub-request, not the batch, names the snapshot and project.
    assert_eq!(
        params["requests"][0]["params"]["symbol"],
        json!({ "kind": 1, "snapshot": 3, "project": PROJECT, "id": 9 })
    );
}

#[test]
fn malformed_handle_is_rejected_before_it_reaches_the_runtime() {
    for handle in [
        "{",
        r#"{"file":7,"id":1}"#,
        r#"{"file":"142"}"#,
        r#"{"id":1}"#,
    ] {
        let error = encode(&SymbolIdentity::default(), json!({ "symbol": handle })).unwrap_err();
        assert!(
            matches!(error, CorsaError::InvalidHandle(_)),
            "{handle}: {error}"
        );
    }
}

#[test]
fn file_memo_keeps_descriptors_in_use_and_drops_idle_generations() {
    let mut memo = FileMemo::with_generation(2);
    let remember = |memo: &mut FileMemo, node_id: &str| {
        memo.remember(node_id, descriptor(node_id).as_object().unwrap());
    };

    remember(&mut memo, "1");
    remember(&mut memo, "2");
    // The third entry starts a new generation; the first two are now "old".
    remember(&mut memo, "3");
    // Using an old entry carries it forward.
    assert_eq!(memo.get("1"), Some(descriptor("1")));
    // The young generation is full again, so the next entry retires "2".
    remember(&mut memo, "4");

    assert_eq!(memo.get("1"), Some(descriptor("1")));
    assert_eq!(memo.get("2"), None);
    assert_eq!(memo.get("4"), Some(descriptor("4")));
}
