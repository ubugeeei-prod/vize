use vize_carton::{String, camelize, capitalize, cstr, is_native_tag};
use vize_relief::{ElementNode, ExpressionNode, IfNode, PropNode, RootNode, TemplateChildNode};

enum FallthroughRootTarget {
    Native(FallthroughRoot),
    /// A component root, by its authored tag. Whether that tag resolves to a
    /// setup binding (and so to a typed fallthrough surface) is decided when
    /// the type is rendered, not while walking the template.
    Component(FallthroughRoot),
}

/// The element the fallthrough attributes land on.
struct FallthroughRoot {
    tag: String,
    /// Template-relative start of the element, the identity Croquis gives
    /// its component usage.
    start: u32,
    /// Prop names the element binds itself (camelized, listeners as `onX`).
    /// Under `checkRequiredFallthroughAttributes` these are the root's
    /// required props the parent is not asked to provide.
    authored_keys: Vec<String>,
}

mod forwarded;
mod types;
pub(super) use forwarded::ForwardedRoots;
pub(crate) use types::fallthrough_component_root_starts;
pub(super) use types::{
    FallthroughComponentScope, fallthrough_attrs_type_ref, fallthrough_props_type_ref,
};

fn explicit_attrs_targets(root: &RootNode<'_>) -> Option<Vec<FallthroughRootTarget>> {
    let mut targets = Vec::new();
    collect_explicit_attrs_targets_from_children(
        root.children.as_slice(),
        root.source,
        &mut targets,
    );
    (!targets.is_empty()).then_some(targets)
}

fn collect_explicit_attrs_targets_from_children(
    children: &[TemplateChildNode<'_>],
    source: &str,
    targets: &mut Vec<FallthroughRootTarget>,
) {
    for child in children {
        collect_explicit_attrs_targets_from_child(child, source, targets);
    }
}

fn collect_explicit_attrs_targets_from_child(
    child: &TemplateChildNode<'_>,
    source: &str,
    targets: &mut Vec<FallthroughRootTarget>,
) {
    match child {
        TemplateChildNode::Element(element) => {
            if element_binds_attrs_explicitly(element, source) {
                targets.push(element_fallthrough_target(element));
            }
            collect_explicit_attrs_targets_from_children(
                element.children.as_slice(),
                source,
                targets,
            );
        }
        TemplateChildNode::If(node) => {
            for branch in &node.branches {
                collect_explicit_attrs_targets_from_children(
                    branch.children.as_slice(),
                    source,
                    targets,
                );
            }
        }
        _ => {}
    }
}

fn element_fallthrough_target(element: &ElementNode<'_>) -> FallthroughRootTarget {
    let root = FallthroughRoot {
        tag: String::from(element.tag),
        start: element.loc.span.start,
        authored_keys: authored_prop_keys(element),
    };
    if is_native_tag(element.tag) {
        FallthroughRootTarget::Native(root)
    } else {
        FallthroughRootTarget::Component(root)
    }
}

/// The statically named props and listeners an element binds, as the keys
/// the rendered component's props type uses for them.
fn authored_prop_keys(element: &ElementNode<'_>) -> Vec<String> {
    let mut keys = Vec::new();
    for prop in element.props.iter() {
        let key = match prop {
            PropNode::Attribute(attribute) => String::from(camelize(attribute.name).as_str()),
            PropNode::Directive(directive) => {
                let static_arg = directive.arg.as_ref().and_then(|arg| match arg {
                    ExpressionNode::Simple(simple) if simple.is_static => Some(simple.content),
                    _ => None,
                });
                match (directive.name, static_arg) {
                    ("bind", Some(arg)) => String::from(camelize(arg).as_str()),
                    ("on", Some(arg)) => {
                        String::from(cstr!("on{}", capitalize(&camelize(arg))).as_str())
                    }
                    ("model", arg) => String::from(camelize(arg.unwrap_or("modelValue")).as_str()),
                    _ => continue,
                }
            }
        };
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    keys
}

fn element_binds_attrs_explicitly(element: &ElementNode<'_>, source: &str) -> bool {
    element.props.iter().any(|prop| {
        let PropNode::Directive(directive) = prop else {
            return false;
        };
        if directive.name != "bind" || directive.arg.is_some() {
            return false;
        }
        directive
            .exp
            .as_ref()
            .is_some_and(|exp| expression_spreads_attrs(expression_source(exp, source).trim()))
    })
}

#[path = "fallthrough/attrs_expr.rs"]
mod attrs_expr;
use attrs_expr::{expression_source, expression_spreads_attrs};

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
            if element.tag == "template" && !has_for_directive(element) =>
        {
            possible_single_root_targets_from_children(element.children.as_slice())
        }
        TemplateChildNode::Element(element) if !has_for_directive(element) => {
            Some(vec![element_fallthrough_target(element)])
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
        let FallthroughRootTarget::Native(root) = target else {
            return None;
        };
        tags.push(root.tag);
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
    if element.tag == "template" {
        return possible_single_root_targets_from_children(element.children.as_slice());
    }
    Some(vec![element_fallthrough_target(element)])
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
        match directive.name {
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
            PropNode::Directive(directive) if directive.name == "for"
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
