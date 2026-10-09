//! Resolve the parser-admitted static Vue component cast in the existing walk.
use crate::{ElementNode, ElementType, PropNode};

pub(super) fn resolve<'a>(element: &mut ElementNode<'a>) {
    if element.tag_type != ElementType::Component
        || matches!(element.tag, "component" | "Component")
    {
        return;
    }
    let Some((ordinal, name)) = element
        .props
        .iter()
        .enumerate()
        .find_map(|(ordinal, prop)| {
            let PropNode::Attribute(attribute) = prop else {
                return None;
            };
            if attribute.name != "is" {
                return None;
            }
            attribute
                .value
                .as_ref()?
                .content
                .strip_prefix("vue:")
                .map(|name| (ordinal, name))
        })
    else {
        return;
    };
    // Keep the original locations; only the semantic identity and selector change.
    element.tag = name;
    element.props.remove(ordinal);
}
