//! `v-text` lowering into the Vue dialect text-content op.

use vize_s0::{Box, String};
use vize_s1::Attribute;
use vize_s2::op::{BindingOp, VueTextOp};

use super::binding::defer;
use super::cx::{Cx, attr_slice, attr_span};
use super::directive::Directive;
use super::element::attr_text;
use super::expr::expr_at;

pub(crate) fn lower_text<'a>(
    cx: &mut Cx<'a>,
    attr: &Attribute<'a>,
    directive: &Directive<'a>,
) -> Option<BindingOp<'a>> {
    let span = attr_span(cx, attr);
    let text = attr_text(attr)
        .map(str::trim)
        .filter(|text| !text.is_empty());
    if directive.arg.is_some() || !directive.modifiers.is_empty() {
        defer(
            cx,
            "defer.v-text",
            span,
            attr_slice(cx, attr),
            String::from(
                "`v-text` is representable as `vue.text` only with no argument or modifier",
            ),
        );
        return None;
    }
    let node = cx.mint_op();
    let value = text.map(|text| expr_at(cx, text));
    cx.record(
        "lower.vue-text",
        node,
        attr_slice(cx, attr),
        String::from("vue.text"),
        span,
    );
    Some(BindingOp::VueText(Box::new_in(
        VueTextOp { value, span },
        &cx.allocator,
    )))
}
