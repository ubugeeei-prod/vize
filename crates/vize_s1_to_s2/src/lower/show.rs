//! `v-show` lowering into the Vue dialect display-toggle op.

use vize_s0::{Box, String};
use vize_s1::Element;
use vize_s2::op::{BindingOp, VueShowOp};

use super::binding::defer;
use super::cx::{Cx, attr_slice, attr_span};
use super::directive::Directive;
use super::element::attr_value_text;
use super::expr::expr_at;

pub(crate) fn lower_show<'a>(
    cx: &mut Cx<'a>,
    element: &Element<'a>,
    index: usize,
    directive: &Directive<'a>,
) -> Option<BindingOp<'a>> {
    let attr = &element.open.attrs[index];
    let span = attr_span(cx, attr);
    let text = attr_value_text(element, index)
        .map(str::trim)
        .filter(|text| !text.is_empty());
    if directive.arg.is_some() || !directive.modifiers.is_empty() || text.is_none() {
        defer(
            cx,
            "defer.v-show",
            span,
            attr_slice(cx, attr),
            String::from(
                "`v-show` is representable as `vue.show` only with a value expression and no argument or modifier",
            ),
        );
        return None;
    }
    let node = cx.mint_op();
    let value = expr_at(cx, text?);
    cx.record(
        "lower.vue-show",
        node,
        attr_slice(cx, attr),
        String::from("vue.show"),
        span,
    );
    Some(BindingOp::VueShow(Box::new_in(
        VueShowOp { value, span },
        &cx.allocator,
    )))
}
