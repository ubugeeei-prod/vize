//! Complete standard envelopes from genuine actual-state route captures.
use super::{
    Context, Future, SOURCE, Service, block_on, error, linked, open, pair, request, send, service,
};
use crate::server::{NativeLinkedNamesRoute, ServerState};
use serde_json::json;
use vize_l0::config::VueDialect;

#[test]
fn standard_native_opt_in_aba_rejects_old_wire_and_keeps_fresh_native_response() {
    route_aba("nativeLinkedEditing");
}
#[test]
fn standard_native_rename_aba_rejects_old_wire_and_keeps_fresh_native_response() {
    route_aba("rename");
}
fn route_aba(key: &str) {
    let mut service = service(true, true);
    open(&mut service, SOURCE, 1);
    assert_eq!(
        send(&mut service, linked(2, 6, 6)),
        Some(pair(2, (6, 5, 8), (6, 32, 35)))
    );
    let mut pending = Box::pin(service.call(request(linked(9, 6, 6))));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    service
        .inner()
        .state
        .apply_lsp_initialization_options(Some(&json!({(key):false})));
    service
        .inner()
        .state
        .apply_lsp_initialization_options(Some(&json!({(key):true})));
    assert_eq!(
        serde_json::to_value(block_on(pending).unwrap().unwrap()).unwrap(),
        error(9, -32801, "Content modified")
    );
    assert_eq!(
        send(&mut service, linked(10, 6, 6)),
        Some(pair(10, (6, 5, 8), (6, 32, 35)))
    );
}

#[test]
fn standard_native_parser_aba_keeps_exact_superseded_error_and_fresh_pair() {
    let mut service = service(true, true);
    open(&mut service, SOURCE, 1);
    let mut pending = Box::pin(service.call(request(linked(9, 6, 6))));
    assert!(
        pending
            .as_mut()
            .poll(&mut Context::from_waker(futures::task::noop_waker_ref()))
            .is_pending()
    );
    service
        .inner()
        .state
        .set_dialect_config(Some(VueDialect::PetiteVue));
    service.inner().state.set_dialect_config(None);
    assert_eq!(
        serde_json::to_value(block_on(pending).unwrap().unwrap()).unwrap(),
        error(9, -32801, "Native Vue configuration superseded")
    );
    assert_eq!(
        send(&mut service, linked(10, 6, 6)),
        Some(pair(10, (6, 5, 8), (6, 32, 35)))
    );
}

#[test]
fn standard_native_real_no_op_configuration_keeps_original_complete_pending_wire() {
    let mut service = service(true, true);
    open(&mut service, SOURCE, 1);
    let mut pending = Box::pin(service.call(request(linked(9, 6, 6))));
    assert!(
        pending
            .as_mut()
            .poll(&mut Context::from_waker(futures::task::noop_waker_ref()))
            .is_pending()
    );
    service
        .inner()
        .state
        .apply_lsp_initialization_options(Some(&json!({"nativeLinkedEditing":true,"rename":true})));
    service.inner().state.set_dialect_config(None);
    assert_eq!(
        serde_json::to_value(block_on(pending).unwrap().unwrap()).unwrap(),
        pair(9, (6, 5, 8), (6, 32, 35))
    );
}

#[test]
fn foreign_equal_state_ticket_refuses_native_endpoint_and_fresh_rpc_still_succeeds() {
    let mut service = service(true, true);
    open(&mut service, SOURCE, 1);
    let foreign = ServerState::new();
    foreign
        .apply_lsp_initialization_options(Some(&json!({"nativeLinkedEditing":true,"rename":true})));
    assert_eq!(
        foreign.native_names_settings(),
        service.inner().state.native_names_settings()
    );
    let NativeLinkedNamesRoute::Native(ticket) = foreign.capture_native_linked_route() else {
        panic!("native route expected");
    };
    let params = serde_json::from_value(linked(9, 6, 6)["params"].clone()).unwrap();
    let refusal = block_on(service.inner().native_linked_editing(params, ticket)).unwrap_err();
    assert_eq!(
        refusal,
        tower_lsp::jsonrpc::Error {
            code: tower_lsp::jsonrpc::ErrorCode::ContentModified,
            message: "Native Vue configuration superseded".into(),
            data: None
        }
    );
    assert_eq!(
        send(&mut service, linked(10, 6, 6)),
        Some(pair(10, (6, 5, 8), (6, 32, 35)))
    );
}

#[test]
fn standard_route_uses_whole_real_loader_state_for_disabled_legacy_and_native_paths() {
    let mut service = service(true, true);
    open(&mut service, SOURCE, 1);
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("vize.config.json"),
        r#"{"vue":{"version":"3"},"dialect":"vue","languageServer":{"rename":false}}"#,
    )
    .unwrap();
    service
        .inner()
        .state
        .load_workspace_config(directory.path());
    assert!(matches!(
        service.inner().state.capture_native_linked_route(),
        NativeLinkedNamesRoute::Disabled
    ));
    assert_eq!(
        send(&mut service, linked(2, 6, 6)),
        Some(json!({"jsonrpc":"2.0","id":2,"result":null}))
    );
    service.inner().state.apply_lsp_initialization_options(Some(
        &json!({"rename":true,"nativeLinkedEditing":false}),
    ));
    assert!(matches!(
        service.inner().state.capture_native_linked_route(),
        NativeLinkedNamesRoute::Legacy
    ));
    assert_eq!(
        send(&mut service, linked(3, 6, 6)),
        Some(pair(3, (6, 5, 8), (6, 32, 35)))
    );
    service
        .inner()
        .state
        .apply_lsp_initialization_options(Some(&json!({"nativeLinkedEditing":true})));
    assert!(matches!(
        service.inner().state.capture_native_linked_route(),
        NativeLinkedNamesRoute::Native(_)
    ));
    assert_eq!(
        send(&mut service, linked(4, 6, 6)),
        Some(pair(4, (6, 5, 8), (6, 32, 35)))
    );
}
