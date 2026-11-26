//! Locations for `v-for` aliases, for a compile that records a map.
//!
//! The `v-for` step builds its alias nodes without a location: nothing but a
//! source map reads one, and the step runs for every compile. A compile that
//! asked for a map locates them on the transformed tree before it generates
//! code (see [`super::prepare_mapped_template`]). The generator then anchors an alias like any
//! located expression, and a compile without a map never runs any of this.

use super::document::authored_offset;
use crate::{ExpressionNode, RootNode, SourceLocation, TemplateChildNode};

/// Give every `v-for` alias that is still the slice of `source` the parser
/// cut its location. An alias a transform rewrote (an erased annotation, a
/// prefixed default) is an arena copy and keeps none.
pub(super) fn locate_for_aliases(root: &mut RootNode<'_>, source: &str) {
    locate_in(&mut root.children, source);
}

fn locate_in(children: &mut [TemplateChildNode<'_>], source: &str) {
    vize_l0::ensure_sufficient_stack(|| locate_children(children, source));
}

fn locate_children(children: &mut [TemplateChildNode<'_>], source: &str) {
    for child in children {
        match child {
            TemplateChildNode::Element(element) => locate_in(&mut element.children, source),
            TemplateChildNode::If(node) => {
                for branch in node.branches.iter_mut() {
                    locate_in(&mut branch.children, source);
                }
            }
            TemplateChildNode::IfBranch(branch) => locate_in(&mut branch.children, source),
            TemplateChildNode::For(node) => {
                let node = &mut **node;
                for alias in [
                    &mut node.value_alias,
                    &mut node.key_alias,
                    &mut node.object_index_alias,
                ] {
                    if let Some(ExpressionNode::Simple(alias)) = alias
                        && alias.loc.span.is_empty()
                        && let Some(start) = authored_offset(alias.content, source)
                    {
                        alias.loc = SourceLocation::new(start, start + alias.content.len() as u32);
                    }
                }
                locate_in(&mut node.children, source);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::locate_for_aliases;
    use crate::{ExpressionNode, TemplateChildNode};

    #[test]
    fn aliases_are_located_at_the_bytes_they_were_parsed_from() {
        let source =
            r#"<ul><li v-for="(item, key, i) in xs"><b v-for="n in item">{{ n }}</b></li></ul>"#;
        let allocator = vize_l0::Allocator::new();
        let (mut root, errors) = crate::parser::parse(&allocator, source);
        assert!(errors.is_empty());
        crate::lane::transform(&allocator, &mut root, Default::default(), None);
        locate_for_aliases(&mut root, source);

        let mut located = std::vec::Vec::new();
        let mut pending = std::vec::Vec::from_iter(root.children.iter());
        while let Some(child) = pending.pop() {
            match child {
                TemplateChildNode::Element(element) => pending.extend(element.children.iter()),
                TemplateChildNode::For(node) => {
                    for alias in [&node.value_alias, &node.key_alias, &node.object_index_alias]
                        .into_iter()
                        .flatten()
                    {
                        let ExpressionNode::Simple(alias) = alias else {
                            continue;
                        };
                        let span = alias.loc.span;
                        located.push((alias.content, span.start, span.end));
                    }
                    pending.extend(node.children.iter());
                }
                _ => {}
            }
        }
        located.sort_unstable();
        assert_eq!(
            located,
            [
                ("i", 27, 28),
                ("item", 16, 20),
                ("key", 22, 25),
                ("n", 47, 48)
            ]
        );
    }
}
