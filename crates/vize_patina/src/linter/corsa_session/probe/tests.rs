use super::{is_snapshot_registry_handle_error, select_probe_type_response};
use corsa::{
    CorsaError,
    api::{TypeHandle, TypeResponse},
    error::RpcResponseError,
    fast::CompactString,
};
use serde_json::json;

fn type_response(id: &str) -> TypeResponse {
    TypeResponse {
        id: TypeHandle::from(id),
        flags: 0,
        object_flags: None,
        value: None,
        target: None,
        type_parameters: Vec::new(),
        outer_type_parameters: Vec::new(),
        local_type_parameters: Vec::new(),
        element_flags: Vec::new(),
        fixed_length: None,
        readonly: None,
        object_type: None,
        index_type: None,
        check_type: None,
        extends_type: None,
        base_type: None,
        subst_constraint: None,
        texts: Vec::new(),
        symbol: None,
    }
}

#[test]
fn symbol_type_response_wins_over_position_type_response() {
    let selected = select_probe_type_response(
        Some(type_response("symbol-type")),
        Some(type_response("position-type")),
    )
    .expect("expected a selected type");

    assert_eq!(selected.id.as_str(), "symbol-type");
}

#[test]
fn position_type_response_is_used_when_symbol_lookup_misses() {
    let selected = select_probe_type_response(None, Some(type_response("position-type")))
        .expect("expected a selected type");

    assert_eq!(selected.id.as_str(), "position-type");
}

#[test]
fn recognizes_structured_snapshot_registry_rpc_errors() {
    assert!(is_snapshot_registry_handle_error(&CorsaError::Rpc(
        RpcResponseError {
            code: -32603,
            message: CompactString::from(
                "api: client error: type handle \"t0000000000000057\" not found in snapshot registry",
            ),
            data: Some(json!({ "method": "getPropertiesOfType" })),
        },
    )));
    assert!(is_snapshot_registry_handle_error(&CorsaError::Rpc(
        RpcResponseError {
            code: -32603,
            message: CompactString::from("not found in snapshot registry"),
            data: None,
        },
    )));
    assert!(is_snapshot_registry_handle_error(&CorsaError::Rpc(
        RpcResponseError {
            code: -32603,
            message: CompactString::from(
                "api: client error: symbol handle 162 not found in snapshot registry",
            ),
            data: Some(json!({ "method": "getTypeOfSymbol" })),
        },
    )));
}

#[test]
fn unrelated_errors_are_not_snapshot_registry_handles() {
    assert!(!is_snapshot_registry_handle_error(&CorsaError::Rpc(
        RpcResponseError {
            code: -32601,
            message: CompactString::from(
                "api: invalid request: unknown API method \"getPropertiesOfType\"",
            ),
            data: Some(json!({ "method": "getPropertiesOfType" })),
        },
    )));
    assert!(!is_snapshot_registry_handle_error(&CorsaError::Protocol(
        CompactString::from("api: client error: snapshot registry handle not found"),
    )));
    assert!(!is_snapshot_registry_handle_error(&CorsaError::Protocol(
        CompactString::from("api: client error: symbol handle not found in snapshot registry"),
    )));
}
