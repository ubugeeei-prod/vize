//! Pinned compiler: typed props, refs, composables and lexical local scopes.

mod support;

use vize_dialect_moonbit::cache::CachedMoonc;
use vize_dialect_moonbit::host::{CheckUnit, MooncHost};
use vize_dialect_moonbit::native::NativeMoonc;
use vize_dialect_moonbit::typed::{MoonBitTypedGuest, project};
use vize_extension_contract::typed_expression::TypedExpressionSession;
use vize_l0::Allocator;

#[expect(
    clippy::unwrap_used,
    reason = "the live suite requires its pinned toolchain"
)]
fn host() -> NativeMoonc {
    let host = NativeMoonc::discover().unwrap();
    assert_eq!(host.toolchain(), support::pinned_toolchain());
    host
}

#[test]
fn the_pinned_compiler_checks_the_generated_interface_and_every_typed_demand() {
    let batch = support::typed::batch();
    let mut session =
        TypedExpressionSession::open(MoonBitTypedGuest::new(CachedMoonc::new(host()))).unwrap();
    let accepted = session.analyze(&batch).unwrap();
    assert_eq!(accepted.analysis.diagnostics, []);
    assert_eq!(accepted.projection.rows.len(), 12);
}

#[test]
fn compiler_type_errors_map_to_the_authored_expressions_exactly() {
    let batch = support::typed::typo_batch();
    let allocator = Allocator::new();
    let typed = project(&allocator, &batch).unwrap();
    let mut host = host();
    let raw = host
        .check(&CheckUnit {
            package: vize_dialect_moonbit::projection::PACKAGE,
            file_name: &typed.projection.file_name,
            source: &typed.projection.text,
            environment: Some(&typed.environment),
        })
        .unwrap();
    let mut text = raw.lines.join("\n");
    if !text.is_empty() {
        text.push('\n');
    }
    support::golden("typed-typos", ".moonc.jsonl", &text);
    let mut session = TypedExpressionSession::open(MoonBitTypedGuest::new(host)).unwrap();
    let accepted = session.analyze(&batch).unwrap();
    assert_eq!(accepted.analysis.diagnostics.len(), 7);
    for (diagnostic, expression) in accepted.analysis.diagnostics.iter().zip(&batch.expressions) {
        assert_eq!(
            diagnostic.severity,
            vize_extension_contract::Severity::Error
        );
        assert!(
            expression.span.start <= diagnostic.span.start
                && diagnostic.span.end <= expression.span.end
        );
    }
    support::golden(
        "typed-typos",
        ".diagnostics.json",
        &serde_json::to_string_pretty(&accepted.analysis.diagnostics).unwrap(),
    );
}

#[test]
fn changing_only_a_producer_signature_changes_the_checked_answer() {
    let mut batch = support::typed::batch();
    batch.expressions.retain(|expression| expression.id == 2);
    let mut session =
        TypedExpressionSession::open(MoonBitTypedGuest::new(CachedMoonc::new(host()))).unwrap();
    assert_eq!(session.analyze(&batch).unwrap().analysis.diagnostics, []);
    batch
        .environment
        .iter_mut()
        .find(|binding| binding.name == "visible")
        .unwrap()
        .signature = "Int".into();
    let changed = session.analyze(&batch).unwrap();
    assert_eq!(changed.analysis.diagnostics.len(), 1);
    assert_eq!(
        changed.analysis.diagnostics.first().unwrap().message,
        "Expr Type Mismatch\n        has type : Int\n        wanted   : Bool"
    );
}

#[test]
fn unsupported_custom_types_fail_in_the_interface_without_guessing() {
    let mut batch = support::typed::batch();
    batch.environment.first_mut().unwrap().signature = "UnknownCustomType".into();
    let mut session = TypedExpressionSession::open(MoonBitTypedGuest::new(host())).unwrap();
    let error = session.analyze(&batch).unwrap_err();
    support::golden("typed-unknown", ".host-error", &vize_l0::cstr!("{error}"));
}
