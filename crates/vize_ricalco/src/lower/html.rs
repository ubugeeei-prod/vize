//! `v-html` lowering into the Vue dialect raw-HTML op.

use vize_s0::{Box, String};
use vize_s1::Element;
use vize_s2::op::{BindingOp, VueHtmlOp};

use super::binding::defer;
use super::cx::{Cx, attr_slice, attr_span};
use super::directive::Directive;
use super::element::attr_value_text;
use super::expr::expr_at;

pub(crate) fn lower_html<'a>(
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
    if directive.arg.is_some() || !directive.modifiers.is_empty() {
        defer(
            cx,
            "defer.v-html",
            span,
            attr_slice(cx, attr),
            String::from(
                "`v-html` is representable as `vue.html` only with no argument or modifier",
            ),
        );
        return None;
    }
    let node = cx.mint_op();
    let value = text.map(|text| expr_at(cx, text));
    cx.record(
        "lower.vue-html",
        node,
        attr_slice(cx, attr),
        String::from("vue.html"),
        span,
    );
    Some(BindingOp::VueHtml(Box::new_in(
        VueHtmlOp { value, span },
        &cx.allocator,
    )))
}
