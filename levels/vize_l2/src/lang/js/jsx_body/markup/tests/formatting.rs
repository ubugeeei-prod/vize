#![expect(
    clippy::panic_in_result_fn,
    reason = "complete native differential assertions"
)]

use super::{Allocator, Law, producer};
use crate::{lang::js::JsxMarkupIssueKind, op::Op, walk::NodeRef};
use alloc::vec::Vec;
use serde_json::{Value, json};

pub(super) fn dom(ops: &[Op<'_>]) -> Result<Value, &'static str> {
    let mut nodes = Vec::new();
    for op in ops {
        nodes.push(match op {
            Op::Element(element) => {
                assert_eq!(element.bindings.len(), 0);
                assert_eq!(element.namespace, crate::op::Namespace::Html);
                let attributes: Vec<_> = element.attributes.iter().map(|a| json!([a.name, a.value.unwrap_or("")])).collect();
                json!({"kind":"element", "tag":element.tag, "namespace":"http://www.w3.org/1999/xhtml", "attributes":attributes, "children":dom(&element.children.ops)?})
            }
            Op::Text(text) => json!({"kind":"text", "value":text.content}),
            _ => return Err("non-static op"),
        });
    }
    Ok(Value::Array(nodes))
}

#[test]
fn original_canonical_nodes_match_complete_pinned_mounted_dom_and_independent_source_spans() -> Law
{
    let oracle: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/jsx-formatting-whitespace/oracle.json"
    )))
    .map_err(|_| "oracle json")?;
    let inputs: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/jsx-formatting-whitespace/cases.json"
    )))
    .map_err(|_| "input json")?;
    let cases = inputs
        .get("cases")
        .and_then(Value::as_array)
        .ok_or("inputs")?;
    let outputs = oracle
        .get("cases")
        .and_then(Value::as_array)
        .ok_or("outputs")?;
    assert_eq!(cases.len(), 15);
    assert_eq!(outputs.len(), cases.len());
    for typescript in [false, true] {
        for (input, output) in cases.iter().zip(outputs) {
            assert_eq!(input.get("id"), output.get("id"));
            let source = input
                .get("source")
                .and_then(Value::as_str)
                .ok_or("source")?;
            let arena = Allocator::default();
            let result = producer(&arena, source, typescript)?.finish_markup();
            if input.get("admit").and_then(Value::as_bool) == Some(false) {
                let rejected = match result {
                    Err(rejected) => rejected,
                    Ok(_) => return Err("unsupported normalized text admitted"),
                };
                assert_eq!(
                    rejected
                        .issues()
                        .iter()
                        .map(|issue| issue.kind)
                        .collect::<Vec<_>>(),
                    [JsxMarkupIssueKind::ValueNormalization]
                );
                assert_eq!(Value::Array(rejected.issues().iter().map(|issue| json!({"kind":"value-normalization","start":issue.span.start,"end":issue.span.end})).collect()), *output.get("refusal").ok_or("whole refusal packet")?);
                assert_eq!(
                    rejected
                        .owner()
                        .ok_or("retained refused owner")?
                        .file()
                        .artifact()
                        .node_count(),
                    0
                );
                continue;
            }
            let native = result.map_err(|_| "formatted static markup rejected")?;
            let artifact = native.file().artifact();
            assert_eq!(
                dom(&artifact.root().ops)?,
                *output.get("dom").ok_or("complete DOM")?
            );
            let mut spans = Vec::new();
            artifact
                .visit_nodes(&mut |_, node| {
                    let kind = match node {
                        NodeRef::Op(Op::Element(_)) => "element",
                        NodeRef::Op(Op::Text(_)) => "text",
                        _ => "unexpected",
                    };
                    let span = node.span();
                    spans.push(json!({"kind":kind,"start":span.start,"end":span.end}));
                })
                .map_err(|_| "node accounting")?;
            assert_eq!(
                Value::Array(spans),
                *output
                    .get("sourceOps")
                    .ok_or("independent original spans")?
            );
            assert_eq!(
                artifact.node_count() as usize,
                output
                    .get("sourceOps")
                    .and_then(Value::as_array)
                    .ok_or("node count")?
                    .len()
            );
            assert!(core::ptr::eq(artifact.source(), source));
        }
    }
    Ok(())
}
