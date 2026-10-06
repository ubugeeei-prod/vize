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
    pub(in crate::lower) template_attributes: Option<RootAttributes<'a, 'b>>,
}

impl<'a, 'b> DirectiveAnalysis<'a, 'b> {
    pub(in crate::lower) fn into_template(self) -> RootAttributes<'a, 'b> {
        self.template_attributes
            .expect("the actual element writer route owns its root facts")
    }
}

pub(in crate::lower) fn classify<'a, 'b>(
    el: &'b ElementNode<'a>,
    inherited: bool,
    own_key: bool,
) -> DirectiveAnalysis<'a, 'b> {
    if own_key && !inherited && super::eligible_target(el) {
        classify_role::<true>(el, inherited)
    } else {
        classify_role::<false>(el, inherited)
    }
}

// Hoist the actual target/scope role once; both paths retain the same property walk.
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

    let needs_html = el.tag_type == ElementType::Element
        && (has_control_flow_children || has_dynamic_element_children || el.tag != "component");
    let mut template_attributes = RootAttributes::new(el, inherited, needs_html);
    for prop in el.props.iter() {
        if let Some(attrs) = &mut template_attributes {
            attrs.observe(prop);
        }
        let PropNode::Directive(dir) = prop else {
            continue;
        };
        scope.observe(dir);
        if READ_KEY {
            match dir.name {
                "for" => has_for = true,
                "bind" if key.is_none() => key = super::binding_value(dir),
                _ => {}
            }
        }
    }
    let (should_lower_as_once, memo_error, key_non_reactive) = scope.finish();
    if let Some(attrs) = &mut template_attributes {
        attrs.set_non_reactive(key_non_reactive);
    }
    DirectiveAnalysis {
        should_lower_as_once,
        memo_error,
        key: key.filter(|value| !key_non_reactive && !has_for && super::eligible_value(value)),
        has_control_flow_children,
        has_dynamic_element_children,
        template_attributes,
    }
}

#[cfg(test)]
mod tests;
