//! Single-root default/fallback slots with the published renderer primitive.

use super::super::{BindingKind, Content, Node};
use super::{Result, at};
use crate::ir::{ComponentKind, PropValueKind};
use crate::s3::LegacyReason;

pub(super) fn check(nodes: &[Node<'_>]) -> Result<()> {
    for node in nodes {
        let Content::Component {
            kind: ComponentKind::Suspense,
            props,
            is,
            ..
        } = &node.content
        else {
            continue;
        };
        if is.is_some()
            || !node.bindings.is_empty()
            || props.iter().any(|prop| {
                prop.dynamic_name.is_some()
                    || prop.value_kind != PropValueKind::Expression
                    || if prop.handler {
                        !matches!(prop.key, "pending" | "fallback" | "resolve")
                    } else {
                        prop.key != "timeout"
                    }
            })
        {
            return Err(LegacyReason::Component.into());
        }
        let mut default = false;
        for &child in &node.children {
            let child = at(nodes, child)?;
            if matches!(
                child.content,
                Content::Element {
                    tag: "template",
                    ..
                }
            ) {
                let [slot] = child.bindings.as_slice() else {
                    return Err(LegacyReason::Component.into());
                };
                let [root_index] = child.children.as_slice() else {
                    return Err(LegacyReason::Component.into());
                };
                if slot.kind != BindingKind::Slot
                    || slot.dynamic_name.is_some()
                    || !slot.value.text.is_empty()
                    || !matches!(slot.name, "default" | "fallback")
                    || !root(at(nodes, *root_index)?)
                {
                    return Err(LegacyReason::Component.into());
                }
                default |= slot.name == "default";
            } else if node.children.len() == 1 && root(child) {
                default = true;
            } else {
                return Err(LegacyReason::Component.into());
            }
        }
        if !default {
            return Err(LegacyReason::Component.into());
        }
    }
    Ok(())
}

fn root(node: &Node<'_>) -> bool {
    matches!(
        node.content,
        Content::Element { .. }
            | Content::Component {
                kind: ComponentKind::Regular,
                ..
            }
    ) && !matches!(
        node.content,
        Content::Element {
            tag: "template",
            ..
        }
    )
}
