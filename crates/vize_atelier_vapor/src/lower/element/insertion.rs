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

fn collect_placeholders(
    children: &[TemplateChildNode<'_>],
    flags: &mut std::vec::Vec<bool>,
    rendered_after: &mut bool,
    non_reactive: bool,
) {
    for child in children.iter().rev() {
        match child {
            TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_) => {
                *rendered_after = true;
            }
            TemplateChildNode::Element(el) if el.tag_type == ElementType::Template => {
                ensure_sufficient_stack(|| {
                    collect_placeholders(
                        &el.children,
                        flags,
                        rendered_after,
                        super::super::key::is_non_reactive(el, non_reactive),
                    )
                });
            }
            TemplateChildNode::Element(el) if is_template_backed_element(el, non_reactive) => {
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
pub(crate) fn block_placeholders(
    children: &[TemplateChildNode<'_>],
    non_reactive: bool,
) -> std::vec::Vec<bool> {
    let mut flags = std::vec::Vec::new();
    collect_placeholders(children, &mut flags, &mut false, non_reactive);
    flags.reverse();
    flags
}

#[cfg(test)]
mod tests {
    use super::{ElementType, TemplateChildNode, block_placeholders};
    use vize_atelier_core::ElementNode;
    use vize_atelier_core::parser::{Parser, parse};
    use vize_carton::{Allocator, ensure_sufficient_stack};

    // Verbatim planner from 9515b781ffb466720fd23e1d5270bc08cf02bf2a;
    // keep its forward unit collection independent of the current reverse walk.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Unit {
        Rendered,
        Block,
    }

    // Verbatim 9515 no-key classifier for the independently retained planner.
    fn is_template_backed_element(el: &ElementNode<'_>) -> bool {
        matches!(el.tag_type, ElementType::Element)
    }

    fn collect_units(children: &[TemplateChildNode<'_>], units: &mut std::vec::Vec<Unit>) {
        for child in children {
            match child {
                TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_) => {
                    units.push(Unit::Rendered);
                }
                TemplateChildNode::Element(el) if el.tag_type == ElementType::Template => {
                    ensure_sufficient_stack(|| collect_units(&el.children, units));
                }
                TemplateChildNode::Element(el) if is_template_backed_element(el) => {
                    units.push(Unit::Rendered);
                }
                TemplateChildNode::Element(_)
                | TemplateChildNode::If(_)
                | TemplateChildNode::For(_) => units.push(Unit::Block),
                _ => {}
            }
        }
    }

    fn original_block_placeholders(children: &[TemplateChildNode<'_>]) -> std::vec::Vec<bool> {
        let mut units = std::vec::Vec::new();
        collect_units(children, &mut units);
        let mut rendered_after = false;
        let mut flags = std::vec::Vec::new();
        for unit in units.iter().rev() {
            match unit {
                Unit::Rendered => rendered_after = true,
                Unit::Block => flags.push(rendered_after),
            }
        }
        flags.reverse();
        flags
    }

    // These are helper-boundary IR controls, not additional valid-source claims.
    // Bare authored templates parse as Element. Explicitly build the transparent
    // Template classification the original expectations were intended to cover.
    fn make_templates_transparent(children: &mut [TemplateChildNode<'_>]) {
        for child in children {
            if let TemplateChildNode::Element(element) = child {
                if element.tag == "template" {
                    assert_eq!(element.tag_type, ElementType::Element);
                    element.tag_type = ElementType::Template;
                }
                make_templates_transparent(&mut element.children);
            }
        }
    }

    #[test]
    fn flattened_siblings_keep_authored_placeholder_order() {
        let cases: [(&str, &[bool], &[bool]); 5] = [
            ("<Comp></Comp>", &[false], &[false]),
            (
                "<Comp></Comp><template><slot></slot>text<template><Comp></Comp></template></template><slot></slot>",
                &[true, false],
                &[true, true, false, false],
            ),
            (
                "<template><!--ignored--><Comp></Comp><template>{{ value }}</template></template><slot></slot><i></i><Comp></Comp>",
                &[true, false],
                &[true, true, false],
            ),
            ("<template>{{ value }} text</template><i></i>", &[], &[]),
            (
                "<template><Comp></Comp><!--ignored--><slot></slot></template>",
                &[],
                &[false, false],
            ),
        ];
        for (source, raw_expected, transparent_expected) in cases {
            let allocator = Allocator::new();
            let (mut root, errors) = Parser::new(&allocator, source).parse();
            assert!(errors.is_empty(), "{source}: {errors:?}");
            let raw_original = original_block_placeholders(&root.children);
            assert_eq!(raw_original, raw_expected, "raw original: {source}");
            assert_eq!(
                block_placeholders(&root.children, false),
                raw_original,
                "raw: {source}"
            );
            make_templates_transparent(&mut root.children);
            let transparent_original = original_block_placeholders(&root.children);
            assert_eq!(
                transparent_original, transparent_expected,
                "transparent original: {source}"
            );
            assert_eq!(
                block_placeholders(&root.children, false),
                transparent_original,
                "transparent: {source}"
            );
        }
    }

    #[test]
    fn original_sibling_order_and_transparent_scopes_keep_exact_block_flags() {
        let cases: [(&str, bool, &[bool]); 8] = [
            ("<i>native</i><p>{{ value }}</p>", false, &[]),
            ("<Counter /><p>after</p><Counter />", false, &[true, false]),
            (
                "<template #default><Counter /></template><span>after</span>",
                false,
                &[true],
            ),
            (
                "<Counter /><template #default><p>after</p><Counter /></template>",
                false,
                &[true, false],
            ),
            (
                "<template #default><Counter /><template #named><Counter /><p>after</p></template></template><Counter />",
                false,
                &[true, true, false],
            ),
            (
                "<input :key=\"epoch\" /><i>after</i><!-- ignored --><input :key=\"end\" />",
                false,
                &[true, false],
            ),
            (
                "<input :key=\"epoch\" /><i>after</i><input :key=\"end\" />",
                true,
                &[],
            ),
            (
                "<Counter /><template #default v-once><input :key=\"held\" /></template><Counter />",
                false,
                &[true, false],
            ),
        ];
        for (source, non_reactive, expected) in cases {
            let allocator = Allocator::new();
            let (root, errors) = parse(&allocator, source);
            assert!(errors.is_empty(), "{errors:?}");
            assert_eq!(
                block_placeholders(&root.children, non_reactive),
                expected,
                "{source}"
            );
        }
    }
}
