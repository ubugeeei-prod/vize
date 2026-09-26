//! One cached regular/dynamic component with the known KeepAlive props.

use super::super::{Content, Node};
use super::{Result, at};
use crate::ir::{ComponentKind, PropValueKind};
use crate::s3::LegacyReason;

pub(super) fn check(nodes: &[Node<'_>]) -> Result<()> {
    for node in nodes {
        let Content::Component {
            kind: ComponentKind::KeepAlive,
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
                !matches!(prop.key, "include" | "exclude" | "max")
                    || prop.handler
                    || prop.value_kind != PropValueKind::Expression
            })
            || node.children.len() != 1
        {
            return Err(LegacyReason::Component.into());
        }
        let [child] = node.children.as_slice() else {
            return Err(LegacyReason::Component.into());
        };
        let child = at(nodes, *child)?;
        if !matches!(
            child.content,
            Content::Component {
                kind: ComponentKind::Regular,
                ..
            }
        ) {
            return Err(LegacyReason::Component.into());
        }
    }
    Ok(())
}
