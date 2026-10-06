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
    use super::block_placeholders;
    use vize_atelier_core::parser::parse;
    use vize_carton::Allocator;

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
