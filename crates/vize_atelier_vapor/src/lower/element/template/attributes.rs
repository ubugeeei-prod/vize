//! Existing dynamic attribute names and static-attribute presence.

use vize_atelier_core::{ElementNode, ExpressionNode, PropNode};
use vize_carton::FxHashSet;

pub(super) fn collect_dynamic_attrs<'a>(
    el: &ElementNode<'a>,
    has_static_attr: &mut bool,
) -> FxHashSet<&'a str> {
    if matches!(el.props.as_slice(), [PropNode::Attribute(_)]) {
        *has_static_attr = true;
        vize_carton::FxHashSet::default()
    } else {
        el.props
            .iter()
            .filter_map(|p| match p {
                PropNode::Attribute(_) => {
                    *has_static_attr = true;
                    None
                }
                PropNode::Directive(dir) if dir.name == "bind" => match dir.arg.as_ref() {
                    Some(ExpressionNode::Simple(key)) => Some(key.content),
                    _ => None,
                },
                _ => None,
            })
            .collect()
    }
}
