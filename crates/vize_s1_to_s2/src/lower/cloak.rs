//! `v-cloak` lowering into the Vue dialect cloak marker.

use vize_s0::{Box, String};
use vize_s1::Attribute;
use vize_s2::op::{BindingOp, VueCloakOp};

use super::cx::{Cx, attr_slice, attr_span};
use super::directive::Directive;

pub(crate) fn lower_cloak<'a>(
    cx: &mut Cx<'a>,
    attr: &Attribute<'a>,
    _directive: &Directive<'a>,
) -> BindingOp<'a> {
    let span = attr_span(cx, attr);
    let node = cx.mint_op();
    cx.record(
        "lower.vue-cloak",
        node,
        attr_slice(cx, attr),
        String::from("vue.cloak"),
        span,
    );
    BindingOp::VueCloak(Box::new_in(VueCloakOp { span }, &cx.allocator))
}
