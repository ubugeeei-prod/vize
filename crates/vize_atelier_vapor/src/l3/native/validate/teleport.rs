//! The Vapor Teleport contract: a static `to`, known static props or computed
//! prop names, and ordinary default slot content.

use super::super::{Content, Node};
use super::{Result, at};
use crate::ir::{ComponentKind, PropValueKind};
use crate::l3::LegacyReason;

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
            || !props
                .iter()
                .any(|prop| prop.dynamic_name.is_none() && prop.key == "to")
            || props.iter().any(|prop| {
                prop.handler
                    || prop.value_kind != PropValueKind::Expression
                    || prop.dynamic_name.is_none()
                        && !matches!(prop.key, "to" | "disabled" | "defer")
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
