//! Existing attribute walk also retains key-only nonreactive ownership.

use vize_atelier_core::{ElementNode, ExpressionNode, PropNode};
use vize_carton::FxHashSet;

pub(super) fn collect_dynamic_attrs<'a>(
    el: &ElementNode<'a>,
    has_static_attr: &mut bool,
    non_reactive: &mut bool,
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
                PropNode::Directive(dir) => {
                    if !*non_reactive {
                        *non_reactive = dir.name == "once"
                            || dir.name == "memo"
                                && matches!(dir.exp.as_ref(), Some(ExpressionNode::Simple(exp))
                                if exp.content.trim() == "[]");
                    }
                    if dir.name == "bind" {
                        match dir.arg.as_ref() {
                            Some(ExpressionNode::Simple(key)) => Some(key.content),
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
            })
            .collect()
    }
}
