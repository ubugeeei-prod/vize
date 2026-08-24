//! `v-once` / `v-memo` → [`BindingOp::VueOnce`] / [`BindingOp::VueMemo`].
//!
//! Vue-specific (no JSX twin — P2-16), codegen-only in the shipped
//! lane (`has_v_once`, `get_memo_exp`). This increment admits
//! well-formed spellings; DOM realization stays later (P2-11 emit).

use vize_carton::{Box, String, cstr};
use vize_sinopia::Element;

use vize_disegno::op::{BindingOp, VueMemoOp, VueOnceOp};

use super::binding::defer;
use super::cx::{Cx, attr_slice, attr_span};
use super::directive::Directive;
use super::element::attr_value_text;
use super::expr::{desc, expr_at};

/// Bare `v-once` (no argument, modifier, or value) → `vue.once`.
pub(crate) fn lower_once<'a>(
    cx: &mut Cx<'a>,
    element: &Element<'a>,
    index: usize,
    directive: &Directive<'a>,
) -> Option<BindingOp<'a>> {
    let attr = &element.open.attrs[index];
    let span = attr_span(cx, attr);
    if !is_bare(directive, element, index) {
        defer(
            cx,
            "defer.v-once",
            span,
            attr_slice(cx, attr),
            String::from("`v-once` is representable as `vue.once` only as the bare directive"),
        );
        return None;
    }
    let node = cx.mint_op();
    cx.record(
        "lower.vue-once",
        node,
        attr_slice(cx, attr),
        String::from("vue.once"),
        span,
    );
    Some(BindingOp::VueOnce(Box::new_in(
        VueOnceOp { span },
        &cx.allocator,
    )))
}

/// `v-memo="…"` with a value and no argument or modifier → `vue.memo`.
pub(crate) fn lower_memo<'a>(
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
            "defer.v-memo",
            span,
            attr_slice(cx, attr),
            String::from(
                "`v-memo` is representable as `vue.memo` only with a value expression and no argument or modifier",
            ),
        );
        return None;
    }
    let text = text?;
    let node = cx.mint_op();
    let value = expr_at(cx, text);
    cx.record(
        "lower.vue-memo",
        node,
        attr_slice(cx, attr),
        cstr!("vue.memo {}", desc(&value)),
        span,
    );
    Some(BindingOp::VueMemo(Box::new_in(
        VueMemoOp { value, span },
        &cx.allocator,
    )))
}

fn is_bare(directive: &Directive<'_>, element: &Element<'_>, index: usize) -> bool {
    directive.arg.is_none()
        && directive.modifiers.is_empty()
        && attr_value_text(element, index).is_none()
}
