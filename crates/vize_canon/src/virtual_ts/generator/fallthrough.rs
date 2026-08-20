use vize_carton::{String, is_native_tag};
use vize_croquis::Croquis;
use vize_relief::{ElementNode, ExpressionNode, IfNode, PropNode, RootNode, TemplateChildNode};

enum FallthroughRootTarget {
    Native(String),
    Component,
}

pub(super) fn fallthrough_props_type_ref(
    summary: &Croquis,
    template_ast: Option<&RootNode<'_>>,
    legacy_vue2: bool,
) -> Option<String> {
    if legacy_vue2 || summary.template_info.inherit_attrs_disabled {
        return None;
    }
    let targets = possible_single_root_targets(template_ast?)?;
    if targets
        .iter()
        .any(|target| matches!(target, FallthroughRootTarget::Component))
    {
        return Some(String::from("Record<string, unknown>"));
    }
    let mut ty = String::default();
    for (index, target) in targets.iter().enumerate() {
        let FallthroughRootTarget::Native(tag) = target else {
            unreachable!("component roots returned open fallthrough props above");
        };
        if index > 0 {
            ty.push_str(" & ");
        }
        ty.push_str("Partial<__VizeNativeElement<");
        push_ts_string_literal(&mut ty, tag.as_str());
        ty.push_str(">>");
    }
    Some(ty)
}

fn possible_single_root_targets(root: &RootNode<'_>) -> Option<Vec<FallthroughRootTarget>> {
    possible_single_root_targets_from_children(root.children.as_slice())
}

fn possible_single_root_targets_from_children(
    children: &[TemplateChildNode<'_>],
) -> Option<Vec<FallthroughRootTarget>> {
    let roots = children
        .iter()
        .filter(|child| !is_ignorable_root_child(child))
        .collect::<Vec<_>>();
    if let [root] = roots.as_slice() {
        return possible_single_root_targets_from_child(root);
    }
    possible_raw_if_chain_targets(roots.as_slice())
}

fn possible_single_root_targets_from_child(
    child: &TemplateChildNode<'_>,
) -> Option<Vec<FallthroughRootTarget>> {
    match child {
        TemplateChildNode::Element(element)
            if element.tag.as_str() == "template" && !has_for_directive(element) =>
        {
            possible_single_root_targets_from_children(element.children.as_slice())
        }
        TemplateChildNode::Element(element) if !has_for_directive(element) => {
            if is_native_tag(element.tag.as_str()) {
                Some(vec![FallthroughRootTarget::Native(String::from(
                    element.tag.as_str(),
                ))])
            } else {
                Some(vec![FallthroughRootTarget::Component])
            }
        }
        TemplateChildNode::If(node) => possible_if_root_targets(node),
        _ => None,
    }
}

fn possible_raw_if_chain_targets(
    children: &[&TemplateChildNode<'_>],
) -> Option<Vec<FallthroughRootTarget>> {
    let (first, rest) = children.split_first()?;
    let first_element = element_child(first)?;
    let ElementBranchKind::If(first_condition) = element_branch_kind(first_element)? else {
        return None;
    };
    if condition_is_literal_true(first_condition) && rest_is_only_if_chain_branches(rest) {
        return possible_element_branch_targets(first_element);
    }

    let mut targets = possible_element_branch_targets(first_element)?;
    for child in rest {
        let element = element_child(child)?;
        match element_branch_kind(element)? {
            ElementBranchKind::ElseIf(condition) => {
                targets.extend(possible_element_branch_targets(element)?);
                if condition_is_literal_true(condition) {
                    return Some(targets);
                }
            }
            ElementBranchKind::Else => {
                targets.extend(possible_element_branch_targets(element)?);
                return Some(targets);
            }
            _ => return None,
        }
    }
    None
}

fn rest_is_only_if_chain_branches(children: &[&TemplateChildNode<'_>]) -> bool {
    let mut has_final_else = false;
    for child in children {
        let Some(element) = element_child(child) else {
            return false;
        };
        match element_branch_kind(element) {
            Some(ElementBranchKind::ElseIf(_)) if !has_final_else => {}
            Some(ElementBranchKind::Else) if !has_final_else => has_final_else = true,
            _ => return false,
        }
    }
    true
}

#[cfg(test)]
fn possible_raw_if_chain_tags(children: &[&TemplateChildNode<'_>]) -> Option<Vec<String>> {
    possible_raw_if_chain_targets(children).and_then(native_target_tags)
}

#[cfg(test)]
fn native_target_tags(targets: Vec<FallthroughRootTarget>) -> Option<Vec<String>> {
    let mut tags = Vec::new();
    for target in targets {
        let FallthroughRootTarget::Native(tag) = target else {
            return None;
        };
        tags.push(tag);
    }
    Some(tags)
}

fn element_child<'a>(child: &'a TemplateChildNode<'a>) -> Option<&'a ElementNode<'a>> {
    match child {
        TemplateChildNode::Element(element) => Some(element),
        _ => None,
    }
}

fn possible_element_branch_targets(
    element: &ElementNode<'_>,
) -> Option<Vec<FallthroughRootTarget>> {
    if has_for_directive(element) {
        return None;
    }
    if element.tag.as_str() == "template" {
        return possible_single_root_targets_from_children(element.children.as_slice());
    }
    if is_native_tag(element.tag.as_str()) {
        Some(vec![FallthroughRootTarget::Native(String::from(
            element.tag.as_str(),
        ))])
    } else {
        Some(vec![FallthroughRootTarget::Component])
    }
}

enum ElementBranchKind<'a> {
    If(&'a ExpressionNode<'a>),
    ElseIf(&'a ExpressionNode<'a>),
    Else,
}

fn element_branch_kind<'a>(element: &'a ElementNode<'a>) -> Option<ElementBranchKind<'a>> {
    for prop in &element.props {
        let PropNode::Directive(directive) = prop else {
            continue;
        };
        match directive.name.as_str() {
            "if" => return directive.exp.as_ref().map(ElementBranchKind::If),
            "else-if" => {
                return directive.exp.as_ref().map(ElementBranchKind::ElseIf);
            }
            "else" => return Some(ElementBranchKind::Else),
            _ => {}
        }
    }
    None
}

fn has_for_directive(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| {
        matches!(
            prop,
            PropNode::Directive(directive) if directive.name.as_str() == "for"
        )
    })
}

fn possible_if_root_targets(node: &IfNode<'_>) -> Option<Vec<FallthroughRootTarget>> {
    let first_branch = node.branches.first()?;
    if first_branch
        .condition
        .as_ref()
        .is_some_and(condition_is_literal_true)
    {
        return possible_single_root_targets_from_children(first_branch.children.as_slice());
    }
    let mut targets = Vec::new();
    for branch in &node.branches {
        targets.extend(possible_single_root_targets_from_children(
            branch.children.as_slice(),
        )?);
        if branch.condition.is_none()
            || branch
                .condition
                .as_ref()
                .is_some_and(condition_is_literal_true)
        {
            return Some(targets);
        }
    }
    None
}

fn is_ignorable_root_child(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Text(text) => text.content.trim().is_empty(),
        TemplateChildNode::Comment(_) => true,
        _ => false,
    }
}

fn condition_is_literal_true(condition: &ExpressionNode<'_>) -> bool {
    matches!(condition, ExpressionNode::Simple(simple) if simple.content.trim() == "true")
}

fn push_ts_string_literal(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            _ => output.push(character),
        }
    }
    output.push('"');
}

#[cfg(test)]
#[path = "fallthrough_tests.rs"]
mod fallthrough_tests;
