//! Emit-time CSS-variable metadata; authored template nodes remain untouched.

use vize_atelier_core::TemplateChildNode;

/// Root eligibility and the only transparent template carriers visited by Vue.
#[derive(Clone, Copy, Default)]
pub(crate) struct RootCssVars {
    pub(crate) enabled: bool,
    pub(crate) template_wrapper: bool,
}

pub(crate) fn root_is_fragment(children: &[TemplateChildNode<'_>]) -> bool {
    let mut rendered = 0;
    let mut non_text = false;
    for child in children {
        if matches!(child, TemplateChildNode::Comment(comment) if comment.directive.is_some()) {
            continue;
        }
        rendered += 1;
        non_text |= !matches!(child, TemplateChildNode::Text(_));
    }
    rendered > 1 && non_text
}
