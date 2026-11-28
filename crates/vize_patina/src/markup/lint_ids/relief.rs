//! Exact raw-Relief projection, including standalone IfBranch nodes.

use super::LintStaticId;
use crate::ir::ByteRange;
use vize_l0::ToCompactString;
use vize_relief::{ElementNode, PropNode, TemplateChildNode};

pub(super) fn collect<'a>(
    children: &[TemplateChildNode<'a>],
    branches: &mut Vec<(u32, usize)>,
    ids: &mut Vec<LintStaticId>,
) {
    let mut index = 0;
    while let Some(child) = children.get(index) {
        if let TemplateChildNode::Element(el) = child
            && element_has_directive(el, "if")
        {
            index = collect_conditional_element_chain(children, index, branches, ids);
            continue;
        }

        match child {
            TemplateChildNode::Element(el) => {
                collect_element_id(el, branches, ids);
                collect(&el.children, branches, ids);
            }
            TemplateChildNode::If(if_node) => {
                for (branch_index, branch) in if_node.branches.iter().enumerate() {
                    branches.push((if_node.loc.span.start, branch_index));
                    collect(&branch.children, branches, ids);
                    branches.pop();
                }
            }
            TemplateChildNode::IfBranch(branch) => collect(&branch.children, branches, ids),
            TemplateChildNode::For(for_node) => {
                collect(&for_node.children, branches, ids);
            }
            _ => {}
        }
        index += 1;
    }
}

fn collect_conditional_element_chain<'a>(
    children: &[TemplateChildNode<'a>],
    mut index: usize,
    branches: &mut Vec<(u32, usize)>,
    ids: &mut Vec<LintStaticId>,
) -> usize {
    let Some(TemplateChildNode::Element(first)) = children.get(index) else {
        return index + 1;
    };
    let if_start = first.loc.span.start;
    let mut branch_index = 0;

    collect_element_branch(first, if_start, branch_index, branches, ids);
    index += 1;

    while let Some(next_index) = next_branch_candidate(children, index) {
        let Some(TemplateChildNode::Element(branch)) = children.get(next_index) else {
            break;
        };
        let is_else = element_has_directive(branch, "else");
        if !is_else && !element_has_directive(branch, "else-if") {
            break;
        }

        branch_index += 1;
        collect_element_branch(branch, if_start, branch_index, branches, ids);
        index = next_index + 1;

        if is_else {
            break;
        }
    }

    index
}

fn collect_element_branch(
    element: &ElementNode,
    if_start: u32,
    branch_index: usize,
    branches: &mut Vec<(u32, usize)>,
    ids: &mut Vec<LintStaticId>,
) {
    branches.push((if_start, branch_index));
    collect_element_id(element, branches, ids);
    collect(&element.children, branches, ids);
    branches.pop();
}

fn next_branch_candidate(children: &[TemplateChildNode<'_>], index: usize) -> Option<usize> {
    children
        .iter()
        .enumerate()
        .skip(index)
        .find(|(_, child)| !is_ignorable_between_branches(child))
        .map(|(index, _)| index)
}

fn is_ignorable_between_branches(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Text(text) => text.content.trim().is_empty(),
        TemplateChildNode::Comment(_) => true,
        _ => false,
    }
}

fn collect_element_id(
    element: &ElementNode,
    branches: &[(u32, usize)],
    ids: &mut Vec<LintStaticId>,
) {
    for prop in &element.props {
        if let PropNode::Attribute(attr) = prop
            && attr.name == "id"
            && let Some(value) = &attr.value
        {
            ids.push(LintStaticId {
                value: value.content.to_compact_string(),
                range: ByteRange::new(attr.loc.span.start, attr.loc.span.end),
                branches: branches.to_vec(),
            });
        }
    }
}

fn element_has_directive(element: &ElementNode, name: &str) -> bool {
    element
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(directive) if directive.name == name))
}
