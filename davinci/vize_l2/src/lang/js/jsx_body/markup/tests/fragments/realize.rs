use super::{Law, dom};
use crate::{
    lang::js::{JsxFile, JsxMarkupFile, JsxNode},
    resolution::SyntaxKind as Syntax,
};
use alloc::{collections::BTreeMap, vec::Vec};
use serde_json::{Value, json};

pub(super) fn original(left: &JsxFile<'_>, right: &JsxFile<'_>) -> Law {
    assert_eq!(left.nodes().count(), right.nodes().count());
    for (left, right) in left.nodes().zip(right.nodes()) {
        assert_eq!(left.kind(), right.kind());
        let fields = |node: JsxNode<'_, '_>| {
            (
                node.span(),
                node.unit(),
                node.scope(),
                node.index(),
                node.parent().map(JsxNode::index),
                node.subtree().len(),
            )
        };
        assert_eq!(fields(left), fields(right));
        assert_eq!(left.source(), right.source());
        let left = left.references().ok_or("left original references")?;
        let right = right.references().ok_or("right original references")?;
        assert_eq!(left.len(), right.len());
        for (left, right) in left.iter().zip(right) {
            assert_eq!(
                (
                    left.unit,
                    left.scope,
                    left.name.as_str(),
                    left.span,
                    left.usage,
                    left.shorthand,
                    left.constructor,
                    left.namespace,
                    left.target
                ),
                (
                    right.unit,
                    right.scope,
                    right.name.as_str(),
                    right.span,
                    right.usage,
                    right.shorthand,
                    right.constructor,
                    right.namespace,
                    right.target
                )
            );
        }
    }
    Ok(())
}

fn range(node: JsxNode<'_, '_>) -> Result<Value, &'static str> {
    let span = node.span().ok_or("original span")?;
    Ok(json!({"start":span.start,"end":span.end}))
}
pub(super) fn boundaries(owner: &JsxMarkupFile<'_>) -> Result<Value, &'static str> {
    let mut packets = Vec::new();
    for node in owner.fragment_boundaries() {
        let kind = match node.kind() {
            Some(Syntax::FragmentOpening) => "opening",
            Some(Syntax::FragmentClosing) => "closing",
            _ => return Err("non-boundary witness"),
        };
        let parent = node.parent().ok_or("fragment parent")?;
        assert_eq!(parent.kind(), Some(Syntax::Fragment));
        assert!(core::ptr::eq(node.owner(), owner.owner()));
        assert!(node.same_owner(parent));
        assert_eq!(node.unit(), parent.unit());
        assert_eq!(node.scope(), parent.scope());
        assert_eq!(
            node.source(),
            Some(if kind == "opening" { "<>" } else { "</>" })
        );
        let mut ancestor = parent.parent();
        while ancestor.is_some_and(|node| node.kind() != Some(Syntax::Fragment)) {
            ancestor = ancestor.and_then(JsxNode::parent);
        }
        let span = node.span().ok_or("boundary span")?;
        packets.push(json!({"kind":kind,"start":span.start,"end":span.end,"fragment":range(parent)?,"parentFragment":ancestor.map(range).transpose()?}));
    }
    Ok(Value::Array(packets))
}

// Test-only realization consumes checked canonical values and actual retained
// boundary records. Its backend-specific anchors never enter semantic IR.
pub(super) fn mounted(owner: &JsxMarkupFile<'_>) -> Result<Value, &'static str> {
    let mut values = BTreeMap::new();
    fn collect(ops: &[crate::op::Op<'_>], values: &mut BTreeMap<(u32, u32), Value>) -> Law {
        for op in ops {
            let (span, value) = match op {
                crate::op::Op::Element(element) => {
                    collect(&element.children.ops, values)?;
                    (
                        element.span,
                        dom(core::slice::from_ref(op))?
                            .as_array()
                            .and_then(|v| v.first())
                            .ok_or("element packet")?
                            .clone(),
                    )
                }
                crate::op::Op::Text(text) => {
                    (text.span, json!({"kind":"text","value":text.content}))
                }
                _ => return Err("unexpected semantic node"),
            };
            assert!(values.insert((span.start, span.end), value).is_none());
        }
        Ok(())
    }
    collect(&owner.file().artifact().root().ops, &mut values)?;
    fn realize(
        node: JsxNode<'_, '_>,
        values: &BTreeMap<(u32, u32), Value>,
    ) -> Result<Vec<Value>, &'static str> {
        let span = node.span().ok_or("source node span")?;
        match node.kind().ok_or("source node kind")? {
            Syntax::Element => {
                let mut value = values
                    .get(&(span.start, span.end))
                    .ok_or("canonical element")?
                    .clone();
                let mut children = Vec::new();
                for child in node.children() {
                    children.extend(realize(child, values)?);
                }
                *value.get_mut("children").ok_or("element children")? = Value::Array(children);
                Ok(alloc::vec![value])
            }
            Syntax::Text(_) => Ok(values
                .get(&(span.start, span.end))
                .cloned()
                .into_iter()
                .collect()),
            Syntax::FragmentOpening | Syntax::FragmentClosing => {
                Ok(alloc::vec![json!({"kind":"text","value":""})])
            }
            Syntax::Expression | Syntax::Fragment => {
                let mut children = Vec::new();
                for child in node.children() {
                    children.extend(realize(child, values)?);
                }
                Ok(children)
            }
            Syntax::Opening { .. } | Syntax::Closing | Syntax::Container => Ok(Vec::new()),
            _ => Err("unexpected retained realization syntax"),
        }
    }
    let mut nodes = Vec::new();
    for root in owner.owner().nodes().filter(|node| {
        node.parent().is_none()
            && node
                .subtree()
                .any(|node| matches!(node.kind(), Some(Syntax::Element | Syntax::Fragment)))
    }) {
        nodes.extend(realize(root, &values)?);
    }
    Ok(Value::Array(nodes))
}
