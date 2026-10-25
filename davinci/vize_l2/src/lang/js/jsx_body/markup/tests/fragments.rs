#![expect(clippy::panic_in_result_fn, reason = "complete original fragment laws")]

use super::{Allocator, Law, facts, formatting::dom, producer};
use crate::{
    lang::js::{JsxFileError, JsxMarkupIssueKind},
    op::Op,
    walk::NodeRef,
};
use alloc::vec::Vec;
use serde_json::{Value, json};

mod coordinates;
mod realize;

#[test]
fn transparent_fragments_match_full_semantics_ranges_and_every_mounted_boundary() -> Law {
    let input: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/jsx-static-fragments/cases.json"
    )))
    .map_err(|_| "input json")?;
    let oracle: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/jsx-static-fragments/oracle.json"
    )))
    .map_err(|_| "oracle json")?;
    let cases = input
        .get("cases")
        .and_then(Value::as_array)
        .ok_or("cases")?;
    let outputs = oracle
        .get("cases")
        .and_then(Value::as_array)
        .ok_or("outputs")?;
    assert_eq!(cases.len(), 22);
    assert_eq!(outputs.len(), cases.len());
    for typescript in [false, true] {
        for (case, output) in cases.iter().zip(outputs) {
            assert_eq!(case.get("id"), output.get("id"));
            let source = case.get("source").and_then(Value::as_str).ok_or("source")?;
            let arena = Allocator::default();
            let incomplete = case.get("fileIncomplete").and_then(Value::as_bool) == Some(true);
            let plain = producer(&arena, source, typescript)?.finish();
            let baseline_file = match &plain {
                Ok(owner) => {
                    assert!(!incomplete);
                    owner.file()
                }
                Err(owner) => {
                    assert!(
                        incomplete,
                        "ordinary owner of {:?} refused: {:?}",
                        case.get("id"),
                        owner.error()
                    );
                    assert_eq!(owner.error(), JsxFileError::IncompleteFile);
                    owner.file().ok_or("retained baseline File")?
                }
            };
            let native = producer(&arena, source, typescript)?;
            let body = native
                .observation()
                .admitted()
                .ok_or("original admission")?
                .program()
                .body
                .as_ptr();
            let result = native.finish_markup();
            if case.get("admit").and_then(Value::as_bool) == Some(false) {
                let rejected = result.err().ok_or("unsupported fragment child admitted")?;
                let packets: Vec<_> = rejected
                    .issues()
                    .iter()
                    .map(|issue| {
                        let kind = match issue.kind {
                            JsxMarkupIssueKind::Component => "component",
                            JsxMarkupIssueKind::Namespace => "namespace",
                            JsxMarkupIssueKind::DynamicChild => "dynamic-child",
                            JsxMarkupIssueKind::DynamicAttribute => "dynamic-attribute",
                            JsxMarkupIssueKind::SpreadAttribute => "spread-attribute",
                            JsxMarkupIssueKind::ValueNormalization => "value-normalization",
                            JsxMarkupIssueKind::SpecialAttribute => "special-attribute",
                            JsxMarkupIssueKind::RootExpression => "root-expression",
                            JsxMarkupIssueKind::File(JsxFileError::IncompleteFile) => {
                                "file-incomplete"
                            }
                            _ => "unexpected-refusal",
                        };
                        json!({"kind":kind,"start":issue.span.start,"end":issue.span.end})
                    })
                    .collect();
                assert_eq!(
                    Value::Array(packets),
                    *output.get("refusal").ok_or("refusal")?
                );
                let retained_file = if incomplete {
                    assert!(rejected.owner().is_none());
                    let retained = rejected
                        .rejected_owner()
                        .ok_or("whole incomplete original owner")?;
                    assert_eq!(retained.error(), JsxFileError::IncompleteFile);
                    assert!(retained.observation().admitted().is_some());
                    retained.file().ok_or("retained incomplete File")?
                } else {
                    assert!(rejected.rejected_owner().is_none());
                    rejected
                        .owner()
                        .ok_or("completed whole refused owner")?
                        .file()
                };
                assert_eq!(
                    retained_file.artifact().node_count(),
                    0,
                    "a preceding valid module root cannot partially publish"
                );
                assert_eq!(facts(retained_file), facts(baseline_file));
                continue;
            }
            let native = result.map_err(|_| "static fragment refused")?;
            assert_eq!(facts(native.file()), facts(baseline_file));
            realize::original(
                native.owner(),
                plain.as_ref().map_err(|_| "completed baseline original")?,
            )?;
            assert_eq!(
                native
                    .owner()
                    .observation()
                    .admitted()
                    .ok_or("retained original")?
                    .program()
                    .body
                    .as_ptr(),
                body
            );
            assert_eq!(
                dom(&native.file().artifact().root().ops)?,
                *output.get("semantic").ok_or("complete semantic tree")?
            );
            assert_eq!(
                realize::boundaries(&native)?,
                *output.get("boundaries").ok_or("boundaries")?
            );
            assert_eq!(
                realize::mounted(&native)?,
                *output.get("dom").ok_or("whole mounted DOM")?
            );
            let mut source_ops = Vec::new();
            native
                .file()
                .artifact()
                .visit_nodes(&mut |_, node| {
                    let kind = match node {
                        NodeRef::Op(Op::Element(_)) => "element",
                        NodeRef::Op(Op::Text(_)) => "text",
                        _ => "unexpected",
                    };
                    let span = node.span();
                    source_ops.push(json!({"kind":kind,"start":span.start,"end":span.end}));
                })
                .map_err(|_| "node accounting")?;
            assert_eq!(
                native.file().artifact().node_count() as usize,
                source_ops.len()
            );
            assert_eq!(
                Value::Array(source_ops),
                *output.get("sourceOps").ok_or("source op ranges")?
            );
        }
    }
    Ok(())
}

#[test]
fn malformed_fragment_never_mints_a_completed_or_partial_owner() -> Law {
    use oxc_parser::Parser;
    use oxc_span::SourceType;
    use vize_l0::SourceRoot;
    for source in [
        "const view = <><p/>;",
        "const view = <><p/></div>;",
        "const view = <><><p/></>;",
    ] {
        for typescript in [false, true] {
            let arena = Allocator::default();
            let original = Parser::new(
                &arena,
                source,
                if typescript {
                    SourceType::tsx().with_module(true)
                } else {
                    SourceType::jsx()
                },
            )
            .parse_observed();
            assert!(original.admitted().is_none());
            let rejected = super::JsxFileProducer::new(
                &arena,
                original,
                SourceRoot::new(source).map_err(|_| "source")?.whole_block(),
                0,
            )
            .err()
            .ok_or("malformed fragment admitted")?;
            assert_eq!(rejected.error(), JsxFileError::SyntaxAdmission);
            assert!(rejected.file().is_none());
            assert!(rejected.observation().admitted().is_none());
        }
    }
    Ok(())
}
