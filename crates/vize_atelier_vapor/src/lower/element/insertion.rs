//! Placeholder planning for blocks that need insertion.
//!
//! Vue 3.6 inserts components, slot outlets, `v-if` and `v-for` blocks with
//! `setInsertionState`. A block keeps a `<!---->` placeholder in its parent's
//! template only when a template-rendered sibling (text, interpolation or
//! element) follows it: the placeholder is the insertion point on the client
//! and the block's hydration unit on the server. Trailing blocks have no
//! placeholder; they append, carrying their logical unit index instead.

use vize_carton::ensure_sufficient_stack;

use super::template::is_template_backed_element;
use super::{ElementType, TemplateChildNode};

fn collect_placeholder_flags(
    children: &[TemplateChildNode<'_>],
    rendered_after: &mut bool,
    flags: &mut std::vec::Vec<bool>,
) {
    for child in children.iter().rev() {
        match child {
            TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_) => {
                *rendered_after = true;
            }
            TemplateChildNode::Element(el) if el.tag_type == ElementType::Template => {
                ensure_sufficient_stack(|| {
                    collect_placeholder_flags(&el.children, rendered_after, flags)
                });
            }
            TemplateChildNode::Element(el) if is_template_backed_element(el) => {
                *rendered_after = true;
            }
            TemplateChildNode::Element(_)
            | TemplateChildNode::If(_)
            | TemplateChildNode::For(_) => flags.push(*rendered_after),
            _ => {}
        }
    }
}

/// For each block among `children` (template wrappers flattened, document
/// order), whether it keeps a template placeholder because a
/// template-rendered sibling follows it.
pub(crate) fn block_placeholders(children: &[TemplateChildNode<'_>]) -> std::vec::Vec<bool> {
    let mut rendered_after = false;
    let mut flags = std::vec::Vec::new();
    // Walk the existing children backwards, carrying visibility through
    // transparent templates instead of allocating an intermediate unit list.
    collect_placeholder_flags(children, &mut rendered_after, &mut flags);
    flags.reverse();
    flags
}

#[cfg(test)]
mod tests {
    use super::block_placeholders;
    use vize_atelier_core::parser::Parser;
    use vize_carton::Allocator;

    #[test]
    fn flattened_siblings_keep_authored_placeholder_order() {
        let cases: [(&str, &[bool]); 5] = [
            ("<Comp></Comp>", &[false]),
            (
                "<Comp></Comp><template><slot></slot>text<template><Comp></Comp></template></template><slot></slot>",
                &[true, true, false, false],
            ),
            (
                "<template><!--ignored--><Comp></Comp><template>{{ value }}</template></template><slot></slot><i></i><Comp></Comp>",
                &[true, true, false],
            ),
            ("<template>{{ value }} text</template><i></i>", &[]),
            (
                "<template><Comp></Comp><!--ignored--><slot></slot></template>",
                &[false, false],
            ),
        ];
        for (source, expected) in cases {
            let allocator = Allocator::new();
            let (root, errors) = Parser::new(&allocator, source).parse();
            assert!(errors.is_empty(), "{source}: {errors:?}");
            assert_eq!(block_placeholders(&root.children), expected, "{source}");
        }
    }
}
