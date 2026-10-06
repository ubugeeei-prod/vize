//! One existing property walk retains directive facts and the original key.

use vize_atelier_core::{
    ElementNode, ElementType, PropNode, SimpleExpressionNode, TemplateChildNode,
};

use super::super::element::template::{RootAttributes, is_static_element};

mod scope;
pub(in crate::lower) use scope::ScopeDirectives;

pub(in crate::lower) struct DirectiveAnalysis<'a, 'b> {
    pub(in crate::lower) should_lower_as_once: bool,
    pub(in crate::lower) memo_error: Option<&'static str>,
    pub(in crate::lower) key: Option<&'b SimpleExpressionNode<'a>>,
    pub(in crate::lower) has_control_flow_children: bool,
    pub(in crate::lower) has_dynamic_element_children: bool,
    pub(in crate::lower) template_attributes: RootAttributes<'a, 'b>,
}

impl<'a, 'b> DirectiveAnalysis<'a, 'b> {
    pub(in crate::lower) fn into_template(self) -> RootAttributes<'a, 'b> {
        self.template_attributes
    }
}

pub(in crate::lower) fn classify<'a, 'b>(
    el: &'b ElementNode<'a>,
    inherited: bool,
    own_key: bool,
) -> DirectiveAnalysis<'a, 'b> {
    if own_key && !inherited && matches!(el.tag_type, ElementType::Element | ElementType::Component)
    {
        classify_role::<true>(el, inherited)
    } else {
        classify_role::<false>(el, inherited)
    }
}

// The original key must exist before its complete target policy is needed.
fn classify_role<'a, 'b, const READ_KEY: bool>(
    el: &'b ElementNode<'a>,
    inherited: bool,
) -> DirectiveAnalysis<'a, 'b> {
    let mut key = None;
    let mut has_for = false;
    let mut scope = ScopeDirectives::new(inherited);
    let has_control_flow_children = el.tag_type == ElementType::Element
        && el
            .children
            .iter()
            .any(|c| matches!(c, TemplateChildNode::If(_) | TemplateChildNode::For(_)));
    let has_dynamic_element_children = el.tag_type == ElementType::Element
        && !has_control_flow_children
        && el.children.iter().any(
            |c| matches!(c, TemplateChildNode::Element(child_el) if !is_static_element(child_el)),
        );

    let needs_writer = el.tag_type == ElementType::Element
        && (has_control_flow_children || has_dynamic_element_children || el.tag != "component");
    let mut template_attributes = RootAttributes::new(el, inherited);
    for prop in el.props.iter() {
        match prop {
            PropNode::Attribute(_) => {
                if needs_writer {
                    template_attributes.observe_attribute();
                }
            }
            PropNode::Directive(dir) => match dir.name {
                "bind" => {
                    if needs_writer {
                        template_attributes.observe_binding(dir);
                    }
                    if READ_KEY && key.is_none() {
                        key = super::binding_value(dir);
                    }
                }
                "once" | "memo" => scope.observe(dir),
                "for" if READ_KEY => has_for = true,
                _ => {}
            },
        }
    }
    let (should_lower_as_once, memo_error, key_non_reactive) = scope.finish();
    template_attributes.set_non_reactive(key_non_reactive);
    DirectiveAnalysis {
        should_lower_as_once,
        memo_error,
        key: key.filter(|value| {
            !key_non_reactive
                && !has_for
                && super::eligible_target(el)
                && super::eligible_value(value)
        }),
        has_control_flow_children,
        has_dynamic_element_children,
        template_attributes,
    }
}

#[cfg(test)]
mod tests;
