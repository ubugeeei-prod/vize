//! TS-48 — the WIT contract round trip, expression-dialect world (P6-1b).
//!
//! The expression echo guest (`tests/guests/expression-echo`, a
//! `wasm32-wasip2` component on the SDK runtime) crosses the
//! `expression-dialect` world out of process and in process; the host checks
//! negotiation for the world's required pages, byte-equal facts and
//! projection payloads against the committed goldens, their acceptance, and
//! the exact trap on a batch the guest does not know.

#![expect(clippy::expect_used, reason = "tests assert by panicking")]

mod support;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use support::build_guest;
use support::expression::{batch, committed, facts, projection};
use vize_extension_host::expression::{
    ExpressionBatch, ExpressionDialectGuest, ExpressionError, ExpressionSession,
};
use vize_extension_host::outproc::{OutOfProcessGuest, serve_expression_command};
use vize_extension_host::wasm::WasmExpressionGuest;
use vize_extension_host::{GuestError, GuestLimits};
use vize_s0::String;

fn guest() -> &'static Path {
    static GUEST: OnceLock<PathBuf> = OnceLock::new();
    GUEST.get_or_init(|| {
        build_guest(
            "expression-echo",
            "vize_contract_expression_echo_guest",
            &[],
        )
    })
}

fn both_modes() -> [(&'static str, Box<dyn ExpressionDialectGuest>); 2] {
    let runner = Path::new(env!("CARGO_BIN_EXE_vize-extension-host"));
    let limits = GuestLimits::default();
    [
        (
            "out-of-process",
            Box::new(
                OutOfProcessGuest::spawn(serve_expression_command(runner, guest(), limits))
                    .expect("the guest loads"),
            ),
        ),
        (
            "in-process",
            Box::new(WasmExpressionGuest::load_with(guest(), limits).expect("the guest loads")),
        ),
    ]
}

#[test]
fn the_expression_guest_imports_no_host_function() {
    let engine = wasmtime::Engine::default();
    let component =
        wasmtime::component::Component::from_file(&engine, guest()).expect("a component");
    let imports: Vec<_> = component
        .component_type()
        .imports(&engine)
        .map(|(name, _)| name.to_owned())
        .collect();
    assert_eq!(imports, ["vize:contracts/types@0.1.2"]);
}

#[test]
fn the_expression_world_negotiates_and_exchanges_byte_equal_pages_in_both_modes() {
    let [facts_golden, projection_golden] = committed();
    for (mode, guest) in both_modes() {
        let mut session = ExpressionSession::open(guest).unwrap_or_else(|e| panic!("{mode}: {e}"));
        assert_eq!(session.negotiated().protocol_version, 1, "{mode}");
        assert_eq!(session.negotiated().langs, Vec::<String>::new(), "{mode}");
        assert_eq!(session.negotiated().ignored, Vec::<String>::new(), "{mode}");
        let accepted = session
            .analyze(&batch())
            .unwrap_or_else(|e| panic!("{mode}: {e}"));
        assert_eq!(accepted.analysis.facts.text, facts_golden, "{mode}");
        assert_eq!(
            accepted.analysis.projection.text, projection_golden,
            "{mode}"
        );
        assert_eq!(accepted.analysis.diagnostics, [], "{mode}");
        assert_eq!(accepted.facts, facts(), "{mode}");
        assert_eq!(accepted.projection, projection(), "{mode}");
    }
}

#[test]
fn an_unknown_batch_traps_the_expression_guest_in_both_modes() {
    let empty = ExpressionBatch {
        environment: Vec::new(),
        expressions: Vec::new(),
    };
    for (mode, guest) in both_modes() {
        let mut session = ExpressionSession::open(guest).expect("negotiates");
        assert_eq!(
            session.analyze(&empty),
            Err(ExpressionError::Guest(GuestError::Trap(String::from(
                "wasm trap: wasm `unreachable` instruction executed"
            )))),
            "{mode}"
        );
    }
}
