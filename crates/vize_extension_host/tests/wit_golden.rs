//! TS-48 — the WIT contract round trip, input-dialect world.
//!
//! A real `wasm32-wasip2` component (the echo guest, `tests/guests/echo`,
//! built here with `--locked`) crosses the `vize:contracts` world through
//! the canonical ABI, out of process (a child `vize-extension-host serve`)
//! and in process (wasmtime in this test), and the host checks, by exact
//! equality only:
//!
//! - capability negotiation, including the version-mismatch refusal of a
//!   guest built to offer protocol 2, with its exact message;
//! - the golden exchange: every page the guest returns is byte-equal to the
//!   committed golden, and the whole answer equals the in-tree Vue
//!   dialect's answer for the same block (`vue_contract` pins the goldens
//!   to that output);
//! - the guest imports no host function, a block it does not know traps it, and a
//!   file that is not a component is refused at load.

mod support;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use vize_extension_host::outproc::{OutOfProcessGuest, serve_command};
use vize_extension_host::vue::VueDialect;
use vize_extension_host::wasm::WasmGuest;
use vize_extension_host::{
    ContractError, Diagnostic, DiagnosticPart, GuestError, HandshakeError, InputDialectGuest,
    PartKind, Session, Severity, SourceBlock, Span, Stage, Witness,
};
use vize_s0::{String, cstr};

use support::{CASES, Case, build_echo_guest, diagnostics_text, golden_texts};

fn echo_guest() -> &'static Path {
    static GUEST: OnceLock<PathBuf> = OnceLock::new();
    GUEST.get_or_init(|| build_echo_guest(&[]))
}

fn protocol_2_guest() -> &'static Path {
    static GUEST: OnceLock<PathBuf> = OnceLock::new();
    GUEST.get_or_init(|| build_echo_guest(&["offer-protocol-2"]))
}

fn out_of_process(component: &Path) -> Result<OutOfProcessGuest, GuestError> {
    let runner = Path::new(env!("CARGO_BIN_EXE_vize-extension-host"));
    OutOfProcessGuest::spawn(serve_command(runner, component))
}

/// Both hosting modes over the same guest binary.
fn both_modes(component: &Path) -> [(&'static str, Box<dyn InputDialectGuest>); 2] {
    [
        (
            "out-of-process",
            Box::new(out_of_process(component).expect("the guest loads")),
        ),
        (
            "in-process",
            Box::new(WasmGuest::load(component).expect("the guest loads")),
        ),
    ]
}

#[test]
fn the_echo_guest_imports_no_host_function() {
    let engine = wasmtime::Engine::default();
    let component = wasmtime::component::Component::from_file(&engine, echo_guest())
        .expect("the guest is a component");
    let imports: Vec<_> = component
        .component_type()
        .imports(&engine)
        .map(|(name, _)| name.to_owned())
        .collect();
    // The shared `types` interface holds only type definitions: importing it
    // asks the host for nothing, and the empty linker satisfies it.
    assert_eq!(imports, ["vize:contracts/types@0.1.2"]);
}

#[test]
fn the_echo_guest_negotiates_in_both_modes() {
    for (mode, guest) in both_modes(echo_guest()) {
        let session = Session::open(guest).unwrap_or_else(|error| panic!("{mode}: {error}"));
        let negotiated = session.negotiated();
        assert_eq!(negotiated.protocol_version, 1, "{mode}");
        assert_eq!(negotiated.langs, [String::from("html")], "{mode}");
        assert_eq!(negotiated.ignored, Vec::<String>::new(), "{mode}");
    }
}

#[test]
fn golden_exchanges_are_byte_equal_in_both_modes() {
    for (mode, guest) in both_modes(echo_guest()) {
        let mut session = Session::open(guest).expect("negotiates");
        let mut exchanged = 0usize;
        for case in CASES {
            let block = case.block();
            let accepted = session
                .lower_block(&block)
                .unwrap_or_else(|error| panic!("{mode} {}: {error}", case.name));
            assert_eq!(
                golden_texts(&accepted.lowered),
                case.committed(),
                "{mode} {}",
                case.name
            );
            let in_tree = VueDialect::default()
                .lower_block(&block)
                .expect("the in-tree dialect is total");
            assert_eq!(accepted.lowered, in_tree, "{mode} {}", case.name);
            exchanged += 1;
        }
        assert_eq!(exchanged, 4, "{mode}");
    }
}

#[test]
fn a_mismatched_protocol_version_is_refused_exactly_in_both_modes() {
    for (mode, guest) in both_modes(protocol_2_guest()) {
        let error = Session::open(guest).expect_err("protocol 2 must be refused");
        assert_eq!(
            error,
            ContractError::Handshake(HandshakeError::ProtocolMismatch { host: 1, guest: 2 }),
            "{mode}"
        );
        assert_eq!(
            cstr!("{error}"),
            "handshake refused: protocol version mismatch: the host speaks 1, the guest offered 2",
            "{mode}"
        );
    }
}

#[test]
fn an_unknown_block_traps_the_guest_in_both_modes() {
    let block = SourceBlock {
        source: String::from("<p>no golden</p>"),
        base: 0,
        lang: None,
    };
    for (mode, guest) in both_modes(echo_guest()) {
        let mut session = Session::open(guest).expect("negotiates");
        let error = session.lower_block(&block).expect_err("the guest traps");
        assert_eq!(
            error,
            ContractError::Guest(GuestError::Trap(String::from(
                "wasm trap: wasm `unreachable` instruction executed"
            ))),
            "{mode}"
        );
    }
}

#[test]
fn a_file_that_is_not_a_component_is_refused_at_load() {
    let path = std::env::temp_dir()
        .join(cstr!("vize-not-a-component-{}.wasm", std::process::id()).as_str());
    std::fs::write(&path, b"not wasm").expect("writes the probe");
    let out = out_of_process(&path).expect_err("the child refuses the file");
    let inside = WasmGuest::load(&path).expect_err("wasmtime refuses the file");
    std::fs::remove_file(&path).expect("removes the probe");
    assert_eq!(out, inside, "both modes report the same load error");
    let GuestError::Instantiate(message) = out else {
        panic!("a load failure is an instantiation error: {out:?}");
    };
    assert_eq!(
        message,
        "failed to parse WebAssembly module: magic header not detected: bad magic number - \
         expected=[\n    0x0,\n    0x61,\n    0x73,\n    0x6d,\n] \
         actual=[\n    0x6e,\n    0x6f,\n    0x74,\n    0x20,\n] (at offset 0x0)"
    );
}

fn probe_diagnostics() -> Vec<Diagnostic> {
    let at = Span { start: 5, end: 5 };
    let part = |kind, message: &str| DiagnosticPart {
        kind,
        span: at,
        message: String::from(message),
    };
    let plain = |severity, stage, message: &str| Diagnostic {
        severity,
        stage,
        span: at,
        message: String::from(message),
        parts: Vec::new(),
        witness: None,
    };
    let every_part = Diagnostic {
        parts: vec![
            part(PartKind::Primary, "the primary label"),
            part(PartKind::Secondary, "a secondary label"),
            part(PartKind::Help, "help text"),
            part(PartKind::Suggestion, "replacement text"),
        ],
        witness: Some(Witness::LegacyExempt(String::from(
            "vize_extension_host::probe",
        ))),
        ..plain(
            Severity::Error,
            Stage::Surface,
            "every severity, stage, part kind and the witness cross the ABI",
        )
    };
    vec![
        every_part,
        plain(Severity::Warning, Stage::Source, "a warning from S0"),
        plain(Severity::Info, Stage::Semantic, "information from S2"),
        plain(Severity::Hint, Stage::Lowered, "a hint from S3"),
        plain(
            Severity::Error,
            Stage::Emit,
            "unicode — 日本語 👋 \"quoted\" \\backslash",
        ),
    ]
}

#[test]
fn every_diagnostic_shape_crosses_the_abi_in_both_modes() {
    let probe = Case {
        name: "probe",
        base: 5,
        lang: None,
    };
    let expected = probe_diagnostics();
    let committed = probe.committed();
    assert_eq!(
        diagnostics_text(&expected),
        committed[2],
        "the probe file spells the list"
    );
    for (mode, guest) in both_modes(echo_guest()) {
        let mut session = Session::open(guest).expect("negotiates");
        let accepted = session
            .lower_block(&probe.block())
            .unwrap_or_else(|error| panic!("{mode}: {error}"));
        assert_eq!(accepted.lowered.diagnostics, expected, "{mode}");
        assert_eq!(golden_texts(&accepted.lowered), committed, "{mode}");
    }
}
