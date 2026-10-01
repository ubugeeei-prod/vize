use vize_relief::{ElementNode, PropNode, TemplateChildNode};

#[derive(Clone, Copy)]
pub(super) struct BranchChoice {
    if_start: u32,
    index: usize,
}

pub(super) fn can_coexist(left: &[BranchChoice], right: &[BranchChoice]) -> bool {
    left.iter().all(|choice| {
        right
            .iter()
            .all(|other| choice.if_start != other.if_start || choice.index == other.index)
    })
}

fn has_directive(element: &ElementNode<'_>, name: &str) -> bool {
    element
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(directive) if directive.name == name))
}

pub(super) fn walk_elements<'a>(
    children: &[TemplateChildNode<'a>],
    branches: &mut Vec<BranchChoice>,
    visitor: &mut impl FnMut(&ElementNode<'a>, &[BranchChoice]),
) {
    let mut index = 0;
    while let Some(child) = children.get(index) {
        match child {
            TemplateChildNode::Element(element) if has_directive(element, "if") => {
                let if_start = element.loc.span.start;
                let mut branch_index = 0;
                loop {
                    let Some(TemplateChildNode::Element(branch)) = children.get(index) else {
                        break;
                    };
                    branches.push(BranchChoice {
                        if_start,
                        index: branch_index,
                    });
                    visitor(branch, branches);
                    walk_elements(&branch.children, branches, visitor);
                    branches.pop();
                    index += 1;
                    if has_directive(branch, "else") {
                        break;
                    }
                    let next = children.iter().enumerate().skip(index).find(|(_, child)| {
                        !matches!(child, TemplateChildNode::Comment(_))
                            && !matches!(child, TemplateChildNode::Text(text) if text.content.trim().is_empty())
                    });
                    let Some((next_index, TemplateChildNode::Element(next))) = next else {
                        break;
                    };
                    if !has_directive(next, "else-if") && !has_directive(next, "else") {
                        break;
                    }
                    index = next_index;
                    branch_index += 1;
                }
                continue;
            }
            TemplateChildNode::Element(element) => {
                visitor(element, branches);
                walk_elements(&element.children, branches, visitor);
            }
            TemplateChildNode::If(node) => {
                for (index, branch) in node.branches.iter().enumerate() {
                    branches.push(BranchChoice {
                        if_start: node.loc.span.start,
                        index,
                    });
                    walk_elements(&branch.children, branches, visitor);
                    branches.pop();
                }
            }
            TemplateChildNode::IfBranch(branch) => {
                walk_elements(&branch.children, branches, visitor)
            }
            TemplateChildNode::For(node) => walk_elements(&node.children, branches, visitor),
            _ => {}
        }
        index += 1;
    }
}
