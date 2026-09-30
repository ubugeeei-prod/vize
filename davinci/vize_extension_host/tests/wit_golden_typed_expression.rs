//! Typed signatures survive the component ABI in both hosting modes.
#![expect(clippy::unwrap_used, reason = "tests assert by panicking")]
mod support;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use vize_extension_host::outproc::{OutOfProcessGuest, serve_typed_expression_command};
use vize_extension_host::typed_expression::{
    Demand, TypedBinding, TypedExpression, TypedExpressionBatch, TypedExpressionGuest,
    TypedExpressionSession,
};
use vize_extension_host::wasm::WasmTypedExpressionGuest;
use vize_extension_host::{GuestError, GuestLimits};

fn batch() -> TypedExpressionBatch {
    let original = support::expression::batch();
    TypedExpressionBatch {
        environment: original
            .environment
            .into_iter()
            .zip(["Int", "String", "String"])
            .map(|(binding, signature)| TypedBinding {
                name: binding.name,
                kind: binding.kind,
                signature: signature.into(),
            })
            .collect(),
        expressions: original
            .expressions
            .into_iter()
            .map(|expression| TypedExpression {
                id: expression.id,
                source: expression.source,
                span: expression.span,
                locals: Vec::new(),
                expected: Demand::Show,
            })
            .collect(),
    }
}

fn guest() -> &'static Path {
    static GUEST: OnceLock<PathBuf> = OnceLock::new();
    GUEST.get_or_init(|| {
        support::build_guest(
            "typed-expression-echo",
            "vize_contract_typed_expression_echo_guest",
            &[],
        )
    })
}

fn both_modes() -> [Box<dyn TypedExpressionGuest>; 2] {
    let runner = Path::new(env!("CARGO_BIN_EXE_vize-extension-host"));
    let limits = GuestLimits::default();
    [
        Box::new(
            OutOfProcessGuest::spawn(serve_typed_expression_command(runner, guest(), limits))
                .unwrap(),
        ),
        Box::new(WasmTypedExpressionGuest::load_with(guest(), limits).unwrap()),
    ]
}

#[test]
fn signatures_cross_both_modes_and_keep_the_existing_analysis_pages() {
    let [facts, projection] = support::expression::committed();
    for guest in both_modes() {
        let mut session = TypedExpressionSession::open(guest).unwrap();
        let accepted = session.analyze(&batch()).unwrap();
        assert_eq!(accepted.analysis.facts.text, facts);
        assert_eq!(accepted.analysis.projection.text, projection);
        assert_eq!(accepted.analysis.diagnostics, []);
    }
}

#[test]
fn changed_types_and_scope_locals_are_observed_by_the_guest_in_both_modes() {
    for mut guest in both_modes() {
        let mut changed = batch();
        changed.environment.first_mut().unwrap().signature = "Bool".into();
        assert!(matches!(
            guest.analyze_typed(&changed),
            Err(GuestError::Trap(_))
        ));
        let mut changed = batch();
        changed.expressions.first_mut().unwrap().locals = vec![TypedBinding {
            name: "msg".into(),
            kind: "local".into(),
            signature: "Int".into(),
        }];
        assert!(matches!(
            guest.analyze_typed(&changed),
            Err(GuestError::Trap(_))
        ));
    }
}

#[test]
fn the_typed_component_imports_no_host_function() {
    let engine = wasmtime::Engine::default();
    let component = wasmtime::component::Component::from_file(&engine, guest()).unwrap();
    let imports: Vec<_> = component
        .component_type()
        .imports(&engine)
        .map(|(name, _)| name.to_owned())
        .collect();
    assert_eq!(imports, ["vize:contracts/types@0.1.3"]);
}
