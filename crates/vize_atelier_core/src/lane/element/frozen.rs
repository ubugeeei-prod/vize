//! Gate only element promotion and Vue-is consumption on validated authored custody.

use super::{ElementNode, TransformContext, maybe_promote_element_to_component, vue_is};
use vize_l0::Box;

pub(super) fn resolve_identity<'a>(
    ctx: &mut TransformContext<'a>,
    el: &mut Box<'a, ElementNode<'a>>,
) {
    if !ctx.element_is_frozen(el.loc.span) {
        maybe_promote_element_to_component(ctx, el);
        vue_is::resolve(el);
    }
}
