//! TS-48 — the WIT contract round trip, output-target world (P6-1c).
//!
//! The output echo guest (`tests/guests/output-echo`, a `wasm32-wasip2`
//! component on the SDK runtime) crosses the `output-target` world out of
//! process and in process; the host checks negotiation for the world's
//! required pages, a byte-equal emit document against the committed golden,
//! and the exact trap on a request the guest does not know.

mod support;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use support::build_guest;
use support::output::{document, request};
use vize_extension_host::outproc::{OutOfProcessGuest, serve_output_command};
use vize_extension_host::output::{EmitRequest, OutputError, OutputSession, OutputTargetGuest};
use vize_extension_host::wasm::WasmOutputGuest;
use vize_extension_host::{GuestError, GuestLimits};
use vize_s0::String;

fn guest() -> &'static Path {
    static GUEST: OnceLock<PathBuf> = OnceLock::new();
    GUEST.get_or_init(|| build_guest("output-echo", "vize_contract_output_echo_guest", &[]))
}

fn both_modes() -> [(&'static str, Box<dyn OutputTargetGuest>); 2] {
    let runner = Path::new(env!("CARGO_BIN_EXE_vize-extension-host"));
    let limits = GuestLimits::default();
    [
        (
            "out-of-process",
            Box::new(
                OutOfProcessGuest::spawn(serve_output_command(runner, guest(), limits))
                    .expect("the guest loads"),
            ),
        ),
        (
            "in-process",
            Box::new(WasmOutputGuest::load_with(guest(), limits).expect("the guest loads")),
        ),
    ]
}

#[test]
fn the_output_guest_imports_no_host_function() {
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
fn the_output_world_negotiates_and_exchanges_a_byte_equal_document_in_both_modes() {
    let golden = document();
    for (mode, guest) in both_modes() {
        let mut session = OutputSession::open(guest).unwrap_or_else(|e| panic!("{mode}: {e}"));
        assert_eq!(session.negotiated().protocol_version, 1, "{mode}");
        assert_eq!(session.negotiated().langs, Vec::<String>::new(), "{mode}");
        assert_eq!(session.negotiated().ignored, Vec::<String>::new(), "{mode}");
        let accepted = session
            .emit(&request())
            .unwrap_or_else(|e| panic!("{mode}: {e}"));
        assert_eq!(accepted.emitted.document.text, golden, "{mode}");
        assert_eq!(accepted.emitted.diagnostics, [], "{mode}");
        assert_eq!(
            accepted.document.text.as_str(),
            "return \"ok\";\n",
            "{mode}"
        );
        assert_eq!(accepted.document.links.len(), 1, "{mode}");
    }
}

#[test]
fn an_unknown_request_traps_the_output_guest_in_both_modes() {
    let unknown = EmitRequest {
        s2: request().s2,
        s3: vize_extension_host::Page {
            schema_version: 1,
            text: String::from("other"),
        },
    };
    for (mode, guest) in both_modes() {
        let mut session = OutputSession::open(guest).expect("negotiates");
        assert_eq!(
            session.emit(&unknown),
            Err(OutputError::Guest(GuestError::Trap(String::from(
                "wasm trap: wasm `unreachable` instruction executed"
            )))),
            "{mode}"
        );
    }
}
