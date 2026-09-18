use vize_armature::patterns::MatchArm;
use vize_s0::{Allocator, Box, String, Vec, cstr};

use crate::{
    DirectiveNode, ElementNode, ElementType, ExpressionNode, PropNode, SimpleExpressionNode,
    SourceLocation, TemplateChildNode,
};

pub(super) fn install_match_scope<'a>(
    allocator: &'a Allocator,
    el: &mut ElementNode<'a>,
    scope: PropNode<'a>,
) {
    if el.tag == "template" && !el.props.iter().any(|prop| matches!(prop, PropNode::Directive(dir) if matches!(dir.name, "if" | "else-if" | "else" | "for" | "slot"))) {
        el.tag_type = ElementType::Template;
        el.props.push(scope);
        return;
    }
    // Only the branches enter the match scope; the authored host keeps its
    // element/component identity, props, directives, and enclosing bindings.
    let mut wrapper = ElementNode::new(allocator, "template", el.loc.clone());
    wrapper.tag_type = ElementType::Template;
    wrapper.ns = el.ns;
    wrapper.children = std::mem::replace(&mut el.children, Vec::new_in(&allocator));
    wrapper.props.push(scope);
    el.children
        .push(TemplateChildNode::Element(Box::new_in(wrapper, &allocator)));
}

pub(super) fn install_arm_scope<'a>(
    allocator: &'a Allocator,
    el: &mut ElementNode<'a>,
    arm: &MatchArm,
    local: &str,
    index: usize,
    loc: SourceLocation,
) {
    let mut branch = ElementNode::new(allocator, "template", el.loc.clone());
    branch.tag_type = ElementType::Template;
    branch.ns = el.ns;
    let name = if index == 0 { "if" } else { "else-if" };
    let raw = if index == 0 { "v-if" } else { "v-else-if" };
    branch.props.push(create_directive(
        allocator,
        name,
        raw,
        Some(cstr!("{local}[0] === {index}")),
        loc.clone(),
    ));
    let mut original = std::mem::replace(el, branch);
    let mut content = Vec::new_in(&allocator);
    let mut retained = Vec::new_in(&allocator);
    if original.tag == "template" {
        content = std::mem::replace(&mut original.children, Vec::new_in(&allocator));
        retained = std::mem::replace(&mut original.props, Vec::new_in(&allocator));
    } else {
        content.push(TemplateChildNode::Element(Box::new_in(
            original, &allocator,
        )));
    }
    if arm.bindings.is_empty() && retained.is_empty() {
        el.children = content;
        return;
    }
    let mut bindings = ElementNode::new(allocator, "template", loc.clone());
    bindings.tag_type = ElementType::Template;
    bindings.ns = el.ns;
    bindings.children = content;
    bindings.props = retained;
    let names = arm
        .bindings
        .iter()
        .map(|binding| binding.name.as_str())
        .collect::<std::vec::Vec<_>>()
        .join(", ");
    bindings.props.push(create_directive(
        allocator,
        "for",
        "v-for",
        Some(cstr!("[, {names}] in [{local}]")),
        loc,
    ));
    el.children.push(TemplateChildNode::Element(Box::new_in(
        bindings, &allocator,
    )));
}

pub(super) fn create_directive<'a>(
    allocator: &'a Allocator,
    name: &'static str,
    raw_name: &'static str,
    exp: Option<String>,
    loc: SourceLocation,
) -> PropNode<'a> {
    let mut dir = DirectiveNode::new(allocator, name, loc.clone());
    dir.raw_name = Some(raw_name);
    dir.exp = exp.map(|exp| {
        ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode::new(allocator.alloc_str(&exp), false, loc),
            &allocator,
        ))
    });
    PropNode::Directive(Box::new_in(dir, &allocator))
}
