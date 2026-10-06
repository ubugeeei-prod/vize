//! Only the actual returned keyed root owns component fallthrough attributes.

use crate::ir::{BlockIRNode, IfIRNode, NegativeBranch, OperationNode};
use vize_carton::{FxHashMap, FxHashSet, ensure_sufficient_stack};

pub(super) fn collect_root_key_templates(
    block: &BlockIRNode<'_>,
    templates: &FxHashMap<usize, usize>,
    roots: &mut FxHashSet<usize>,
) {
    let [returned] = block.returns.as_slice() else {
        return;
    };
    // An existing native return already owns its root flag. Unique IR ids
    // cannot simultaneously identify the returned Key or conditional owner.
    if templates.contains_key(returned) {
        return;
    }
    ensure_sufficient_stack(|| {
        for op in &block.operation {
            match op {
                OperationNode::Key(node) if node.parent.is_none() && node.id == *returned => {
                    if let [native] = node.render.returns.as_slice()
                        && let Some(&template) = templates.get(native)
                    {
                        roots.insert(template);
                    }
                }
                OperationNode::If(node) if node.parent.is_none() && node.id == *returned => {
                    collect_conditional_key_roots(node, templates, roots);
                }
                _ => {}
            }
        }
    });
}

fn collect_conditional_key_roots(
    node: &IfIRNode<'_>,
    templates: &FxHashMap<usize, usize>,
    roots: &mut FxHashSet<usize>,
) {
    collect_root_key_templates(&node.positive, templates, roots);
    match node.negative.as_ref() {
        Some(NegativeBranch::Block(block)) => collect_root_key_templates(block, templates, roots),
        Some(NegativeBranch::If(branch)) if branch.parent.is_none() => {
            ensure_sufficient_stack(|| collect_conditional_key_roots(branch, templates, roots));
        }
        _ => {}
    }
}
