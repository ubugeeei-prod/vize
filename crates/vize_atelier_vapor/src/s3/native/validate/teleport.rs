//! The Vapor Teleport contract: known props and ordinary default slot content.

use super::super::{Content, Node};
use super::{Result, at};
use crate::ir::{ComponentKind, PropValueKind};
use crate::s3::LegacyReason;

pub(super) fn check(nodes: &[Node<'_>]) -> Result<()> {
    for node in nodes {
        let Content::Component {
            kind: ComponentKind::Teleport,
            props,
            is,
            ..
        } = &node.content
        else {
            continue;
        };
        if is.is_some()
            || !node.bindings.is_empty()
            || !props.iter().any(|prop| prop.key == "to")
            || props.iter().any(|prop| {
                !matches!(prop.key, "to" | "disabled" | "defer")
                    || prop.dynamic_name.is_some()
                    || prop.handler
                    || prop.value_kind != PropValueKind::Expression
            })
        {
            return Err(LegacyReason::Component.into());
        }
        for child in &node.children {
            if matches!(
                at(nodes, *child)?.content,
                Content::Element {
                    tag: "template",
                    ..
                }
            ) {
                return Err(LegacyReason::Component.into());
            }
        }
    }
    Ok(())
}
