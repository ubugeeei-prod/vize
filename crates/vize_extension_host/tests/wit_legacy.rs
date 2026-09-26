//! Real SDK/WIT 0.1.2 components against the current host, in both modes.

mod support;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use vize_extension_host::expression::{ExpressionDialectGuest, ExpressionSession};
use vize_extension_host::outproc::{OutOfProcessGuest, serve_command, serve_expression_command};
use vize_extension_host::wasm::{WasmExpressionGuest, WasmGuest};
use vize_extension_host::{GuestLimits, InputDialectGuest, Session, SourceBlock};
use vize_l0::String;

fn input_guest() -> &'static Path {
    static GUEST: OnceLock<PathBuf> = OnceLock::new();
    GUEST.get_or_init(|| {
        support::build_guest("sdk-hello-0-1-2", "vize_contract_legacy_hello_guest", &[])
    })
}

fn expression_guest() -> &'static Path {
    static GUEST: OnceLock<PathBuf> = OnceLock::new();
    GUEST.get_or_init(|| {
        support::build_guest(
            "expression-echo-0-1-2",
            "vize_contract_legacy_expression_guest",
            &[],
        )
    })
}

#[test]
fn compiled_components_retain_the_historical_interface_names() {
    let engine = wasmtime::Engine::default();
    for (path, interface) in [
        (input_guest(), "vize:contracts/input-lowering@0.1.2"),
        (
            expression_guest(),
            "vize:contracts/expression-analysis@0.1.2",
        ),
    ] {
        let component =
            wasmtime::component::Component::from_file(&engine, path).expect("a component");
        let ty = component.component_type();
        let imports: Vec<_> = ty
            .imports(&engine)
            .map(|(name, _)| name.to_owned())
            .collect();
        assert_eq!(imports, ["vize:contracts/types@0.1.2"]);
        let mut exports: Vec<_> = ty
            .exports(&engine)
            .map(|(name, _)| name.to_owned())
            .collect();
        exports.sort_unstable();
        let mut expected = ["vize:contracts/handshake@0.1.2", interface];
        expected.sort_unstable();
        assert_eq!(exports, expected);
    }
}

const L1: &str =
    "[s1]\nbytes=14\n\n[s1.tree]\ntext 0:0:4\ntext 4:4:10\ntext 10:10:11\ntext 11:11:14\n\n";
const L2: &str = "[disegno]\nops=7\n\n[disegno.ops]\nui.element ul @40:54\n  ui.element li @40:43\n    \
                  ui.text \"Hello, Ada!\" @40:43\n  ui.element li @44:49\n    ui.text \"Hello, Grace!\" @44:49\n  \
                  ui.element li @51:54\n    ui.text \"Hello, Lin!\" @51:54\n\n";

#[test]
fn the_old_default_sdk_export_exchanges_exact_pages_in_both_modes() {
    let runner = Path::new(env!("CARGO_BIN_EXE_vize-extension-host"));
    let guests: [(&str, Box<dyn InputDialectGuest>); 2] = [
        (
            "out-of-process",
            Box::new(
                OutOfProcessGuest::spawn(serve_command(runner, input_guest())).expect("loads"),
            ),
        ),
        (
            "in-process",
            Box::new(WasmGuest::load(input_guest()).expect("loads")),
        ),
    ];
    let block = SourceBlock {
        source: String::from("Ada\nGrace\n\nLin"),
        base: 40,
        lang: Some(String::from("hello")),
    };
    for (mode, guest) in guests {
        let mut session = Session::open(guest).unwrap_or_else(|error| panic!("{mode}: {error}"));
        assert_eq!(
            session.negotiated().langs,
            [String::from("hello")],
            "{mode}"
        );
        let accepted = session
            .lower_block(&block)
            .unwrap_or_else(|error| panic!("{mode}: {error}"));
        assert_eq!(accepted.lowered.surface.text, L1, "{mode}");
        assert_eq!(accepted.lowered.semantic.text, L2, "{mode}");
        assert_eq!(accepted.lowered.diagnostics, [], "{mode}");
    }
}

#[test]
fn the_old_expression_world_exchanges_exact_pages_in_both_modes() {
    let runner = Path::new(env!("CARGO_BIN_EXE_vize-extension-host"));
    let limits = GuestLimits::default();
    let guests: [(&str, Box<dyn ExpressionDialectGuest>); 2] = [
        (
            "out-of-process",
            Box::new(
                OutOfProcessGuest::spawn(serve_expression_command(
                    runner,
                    expression_guest(),
                    limits,
                ))
                .expect("loads"),
            ),
        ),
        (
            "in-process",
            Box::new(WasmExpressionGuest::load_with(expression_guest(), limits).expect("loads")),
        ),
    ];
    let [facts, projection] = support::expression::committed();
    for (mode, guest) in guests {
        let mut session =
            ExpressionSession::open(guest).unwrap_or_else(|error| panic!("{mode}: {error}"));
        let accepted = session
            .analyze(&support::expression::batch())
            .unwrap_or_else(|error| panic!("{mode}: {error}"));
        assert_eq!(accepted.analysis.facts.text, facts, "{mode}");
        assert_eq!(accepted.analysis.projection.text, projection, "{mode}");
        assert_eq!(accepted.analysis.diagnostics, [], "{mode}");
        assert_eq!(accepted.facts, support::expression::facts(), "{mode}");
        assert_eq!(
            accepted.projection,
            support::expression::projection(),
            "{mode}"
        );
    }
}
