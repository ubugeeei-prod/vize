//! Typed environment goldens and exact foreign span links, no toolchain.

mod support;

use vize_dialect_moonbit::host::Replay;
use vize_dialect_moonbit::typed::{EnvironmentError, MoonBitTypedGuest, project};
use vize_extension_contract::typed_expression::{TypedExpressionGuest, TypedExpressionSession};
use vize_extension_host::wire::{Request, read_message, write_message};
use vize_l0::Allocator;

#[test]
fn typed_environment_projection_and_facts_are_exact() {
    let batch = support::typed::batch();
    let allocator = Allocator::new();
    let typed = project(&allocator, &batch).unwrap();
    support::golden("typed", ".mbti", &typed.environment);
    support::golden("typed", ".mbt", &typed.projection.text);
    assert_eq!(typed.projection.positions.len(), 12);
    assert_eq!(typed.projection.links.len(), 12);
    for (expression, link) in batch.expressions.iter().zip(&typed.projection.links) {
        assert_eq!(link.source, expression.span.into());
        assert_eq!(
            typed
                .projection
                .text
                .get(link.generated.start as usize..link.generated.end as usize)
                .unwrap(),
            expression.source
        );
    }
    let host = Replay::new(&support::pinned_toolchain(), "");
    let mut session = TypedExpressionSession::open(MoonBitTypedGuest::new(host)).unwrap();
    let accepted = session.analyze(&batch).unwrap();
    support::golden("typed", ".facts.folio", &accepted.analysis.facts.text);
    support::golden(
        "typed",
        ".projection.folio",
        &accepted.analysis.projection.text,
    );
    assert_eq!(accepted.analysis.diagnostics, []);
    assert_eq!(&accepted.facts.alpha.references[&7], "item,title");
    assert_eq!(&accepted.facts.alpha.references[&8], "title,index");
    assert_eq!(accepted.facts.alpha.exact.get(&10), Some(&false));
}

#[test]
fn a_typed_batch_round_trips_without_losing_signatures_scope_or_unicode() {
    let request = Request::AnalyzeTyped {
        batch: support::typed::batch(),
    };
    let mut bytes = Vec::new();
    write_message(&mut bytes, &request).unwrap();
    assert_eq!(
        read_message::<_, Request>(&mut bytes.as_slice()).unwrap(),
        Some(request)
    );
}

#[test]
fn missing_types_and_scaffolding_injection_are_refused() {
    for signature in [
        "",
        " ",
        "Int\n}",
        "Int, injected : String",
        "Int) -> Unit {",
        "@foreign.Type",
        "Ref[Int",
        "Array[Int)]",
    ] {
        let mut batch = support::typed::batch();
        batch.environment.first_mut().unwrap().signature = signature.into();
        let allocator = Allocator::new();
        let error = project(&allocator, &batch).unwrap_err();
        assert_eq!(
            error,
            EnvironmentError("binding \"title\" has a missing or unsupported signature".into()),
            "{signature:?}"
        );
    }
    let mut batch = support::typed::batch();
    batch
        .environment
        .push(batch.environment.first().unwrap().clone());
    assert_eq!(
        project(&Allocator::new(), &batch).unwrap_err(),
        EnvironmentError("duplicate binding \"title\"".into())
    );
    let mut batch = support::typed::batch();
    batch
        .expressions
        .push(batch.expressions.first().unwrap().clone());
    assert_eq!(
        project(&Allocator::new(), &batch).unwrap_err(),
        EnvironmentError("duplicate expression id 0".into())
    );
    batch.expressions.pop();
    batch.expressions.first_mut().unwrap().span.end += 1;
    assert_eq!(
        project(&Allocator::new(), &batch).unwrap_err(),
        EnvironmentError("expression 0 does not have an exact authored byte span".into())
    );
}

#[test]
fn a_siblings_local_is_not_an_exact_binding_in_this_scope() {
    let batch = support::typed::batch();
    let host = Replay::new(&support::pinned_toolchain(), "");
    let mut guest = MoonBitTypedGuest::new(host);
    let mut analysis = guest.analyze_typed(&batch).unwrap();
    // Change the parsed canonical document to avoid relying on its spelling.
    use vize_davinci::fact::{AlphaDocument, ExpressionFacts};
    use vize_davinci::folio::{Folio, FolioMode};
    let mut facts = AlphaDocument::<ExpressionFacts>::parse(&analysis.facts.text).unwrap();
    facts.alpha.references.insert(0, "item".into());
    facts.alpha.exact.insert(0, true);
    analysis.facts.text = facts.print_to_string(FolioMode::Full);
    let error = vize_extension_contract::typed_expression::accept_typed_analysis(&batch, analysis)
        .unwrap_err();
    assert_eq!(
        error,
        vize_extension_contract::expression::AnalysisError::UnknownBinding {
            id: 0,
            name: "item".into()
        }
    );
}

#[test]
fn a_large_typed_body_is_refused_before_projection_offset_conversion() {
    let mut batch = support::typed::batch();
    batch.expressions = vec![support::typed::expression(
        0,
        &"x".repeat(128 * 1024),
        vize_extension_contract::typed_expression::Demand::Value,
    )];
    assert_eq!(
        project(&Allocator::new(), &batch).unwrap_err(),
        EnvironmentError("typed projection exceeds the 128 KiB transport budget".into())
    );
}
