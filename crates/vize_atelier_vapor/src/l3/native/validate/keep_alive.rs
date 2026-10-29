//! One cached regular/dynamic component with known static KeepAlive props
//! or computed prop names.

use super::super::{Content, Node};
use super::{Result, at};
use crate::ir::{ComponentKind, PropValueKind};
use crate::l3::LegacyReason;

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
                prop.handler
                    || prop.value_kind != PropValueKind::Expression
                    || prop.dynamic_name.is_none()
                        && !matches!(prop.key, "include" | "exclude" | "max")
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
                kind: ComponentKind::Regular | ComponentKind::Dynamic,
                ..
            }
        ) {
            return Err(LegacyReason::Component.into());
        }
    }
    Ok(())
}
