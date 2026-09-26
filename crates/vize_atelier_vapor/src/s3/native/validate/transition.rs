//! Published JS-hook transitions with one HTML root or a keyed HTML loop.

use super::super::{BindingKind, Content, Node};
use super::{Result, at};
use crate::ir::{ComponentKind, PropValueKind};
use crate::s3::LegacyReason;

pub(super) fn check(nodes: &[Node<'_>]) -> Result<()> {
    for node in nodes {
        let Content::Component {
            kind, props, is, ..
        } = &node.content
        else {
            continue;
        };
        if !matches!(
            kind,
            ComponentKind::Transition | ComponentKind::TransitionGroup
        ) {
            continue;
        }
        let group = *kind == ComponentKind::TransitionGroup;
        if is.is_some()
            || !node.bindings.is_empty()
            || props.iter().any(|prop| {
                prop.dynamic_name.is_some()
                    || prop.value_kind != PropValueKind::Expression
                    || if prop.handler {
                        !matches!(
                            prop.key,
                            "before-enter"
                                | "enter"
                                | "after-enter"
                                | "enter-cancelled"
                                | "before-leave"
                                | "leave"
                                | "after-leave"
                                | "leave-cancelled"
                        )
                    } else if prop.key == "css" {
                        !prop.dynamic || prop.value.is_none_or(|value| value.text.trim() != "false")
                    } else {
                        !group
                            || prop.key != "tag"
                            || prop.dynamic
                            || prop
                                .value
                                .is_none_or(|value| !matches!(value.text, "ul" | "div"))
                    }
            })
            || !props.iter().any(|prop| prop.key == "css")
            || group && !props.iter().any(|prop| prop.key == "tag")
        {
            return Err(LegacyReason::Component.into());
        }
        let [child] = node.children.as_slice() else {
            return Err(LegacyReason::Component.into());
        };
        let child = at(nodes, *child)?;
        let root = if group {
            let Content::For(body) = child.content else {
                return Err(LegacyReason::Component.into());
            };
            if body.template || body.key_prop.is_none() || body.source.text.parse::<f64>().is_ok() {
                return Err(LegacyReason::Component.into());
            }
            let [root] = child.children.as_slice() else {
                return Err(LegacyReason::Component.into());
            };
            at(nodes, *root)?
        } else if let Content::If { branches } = &child.content {
            let [branch] = branches.as_slice() else {
                return Err(LegacyReason::Component.into());
            };
            let [root] = branch.roots.as_slice() else {
                return Err(LegacyReason::Component.into());
            };
            at(nodes, *root)?
        } else {
            child
        };
        if !matches!(
            root.content,
            Content::Element {
                tag: "p" | "button" | "div" | "span" | "li",
                ..
            }
        ) {
            return Err(LegacyReason::Component.into());
        }
        // Descendant element IDs inside conditional roots have a separate
        // retained/native ordering obligation. Keep this slice text-only.
        if root
            .bindings
            .iter()
            .any(|binding| !matches!(binding.kind, BindingKind::Prop | BindingKind::Event))
            || root.children.iter().any(|index| {
                !matches!(
                    nodes.get(*index).map(|node| &node.content),
                    Some(Content::Text { .. })
                )
            })
        {
            return Err(LegacyReason::Component.into());
        }
    }
    Ok(())
}
