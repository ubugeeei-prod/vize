//! `v-bind` / `v-on` into the normalized one-way ops (P2-9 series 5).
//!
//! The ops keep the authored surface; the two normalizations here are
//! the shipped **parser's**, mirrored byte-for-byte so the two lanes
//! never differ on a spelling
//! (`crates/vize_armature/src/parser/attribute.rs:267-340`):
//!
//! - **same-name shorthand** (Vue 3.4): a `v-bind` with a static
//!   argument and no authored value reads as binding the camelized
//!   argument name (`:foo-bar` ⇒ value `fooBar`), recorded under
//!   [`RULE_SAME_NAME`];
//! - **the `.` dot shorthand** synthesizes a leading `prop` modifier
//!   when the authored modifiers do not already carry one.
//!
//! The legacy lane emits **no** grammar diagnostic for a valueless
//! `v-bind`/`v-on` (relief's `VBindNoExpression`/`VOnNoExpression` have
//! zero live call sites - measured for this installment), so the
//! lowering emits none either: a bare `@click` is a bare listener, a
//! bare `v-bind` an empty spread, both kept as authored.

use vize_carton::{Box, String, Vec, cstr};
use vize_sinopia::Element;

use vize_disegno::expr::ExprRef;
use vize_disegno::op::{BindOp, BindingOp, DynamicName, OnOp};

use super::cx::{Cx, attr_slice, attr_span};
use super::directive::{Arg, Directive};
use super::element::attr_value_text;
use super::expr::{desc, expr_at};

/// The provenance rule of the same-name expansion.
pub(crate) const RULE_SAME_NAME: &str = "normalize.bind.same-name";

/// The op's name position from the directive's argument.
fn lower_name<'a>(cx: &mut Cx<'a>, arg: Option<Arg<'a>>) -> Option<DynamicName<'a>> {
    arg.map(|arg| match arg {
        Arg::Static(text) => DynamicName::Static(text),
        Arg::Dynamic(inner) => DynamicName::Dynamic(expr_at(cx, inner)),
    })
}

/// The record spelling of an optional name position.
fn name_desc(head: &str, name: &Option<DynamicName<'_>>) -> String {
    match name {
        None => String::from(head),
        Some(DynamicName::Static(text)) => cstr!("{head} \"{text}\""),
        Some(DynamicName::Dynamic(expr)) => cstr!("{head} {}", desc(expr)),
    }
}

/// Admit one `v-bind` value: under a legacy filter dialect a top-level
/// filter chain becomes the pessimal escape with its split recorded
/// (P2-9 series 7); everything else — and every default-dialect value —
/// takes the ordinary total rule.
fn bind_value_at<'a>(
    cx: &mut Cx<'a>,
    node: Option<vize_davinci::id::NodeId>,
    text: &'a str,
) -> vize_disegno::expr::ExprRef<'a> {
    #[cfg(feature = "_legacy")]
    if let Some(expr) = super::legacy::filter_value(cx, node, text) {
        return expr;
    }
    #[cfg(not(feature = "_legacy"))]
    let _ = node;
    expr_at(cx, text)
}

/// Lower one `v-bind` spelling into `ui.bind`.
pub(crate) fn lower_bind<'a>(
    cx: &mut Cx<'a>,
    element: &Element<'a>,
    index: usize,
    directive: &Directive<'a>,
) -> BindingOp<'a> {
    let attr = &element.open.attrs[index];
    let span = attr_span(cx, attr);
    let node = cx.mint_op();
    let name = lower_name(cx, directive.arg);

    // The `.foo` dot shorthand: a leading synthesized `prop` modifier,
    // exactly as the shipped parser injects it.
    let dot_shorthand = attr.name.text.starts_with('.') && !directive.modifiers.contains(&"prop");
    let mut modifiers: Vec<'a, &'a str> = Vec::new_in(&cx.allocator);
    if dot_shorthand {
        modifiers.push("prop");
    }
    for modifier in &directive.modifiers {
        modifiers.push(modifier);
    }

    let value = match attr_value_text(element, index) {
        // An authored-blank value (`:x=""`) materializes no expression
        // in the shipped parser (and blocks the same-name expansion,
        // which requires no value position at all) — mirrored exactly,
        // caught by the corpus lane on the first run (directus
        // `@contextmenu.stop=""`).
        Some(text) if text.trim().is_empty() => None,
        Some(text) => Some(bind_value_at(cx, node, text)),
        None => match directive.arg {
            // Same-name shorthand: static argument, no authored value.
            // The synthesized text is not a source slice, so the
            // expression is admitted at the argument's own span.
            Some(Arg::Static(arg_text)) => {
                let camel = vize_carton::camelize(arg_text);
                let text: &'a str = if camel.as_str() == arg_text {
                    arg_text
                } else {
                    cx.allocator.alloc_str(camel.as_str())
                };
                let expr = ExprRef::parse_js_in(cx.allocator, text, cx.span_of(arg_text));
                cx.record(
                    RULE_SAME_NAME,
                    node,
                    attr_slice(cx, attr),
                    cstr!("{text}"),
                    span,
                );
                Some(expr)
            }
            _ => None,
        },
    };

    cx.record(
        "lower.bind",
        node,
        attr_slice(cx, attr),
        name_desc("ui.bind", &name),
        span,
    );
    BindingOp::Bind(Box::new_in(
        BindOp {
            name,
            modifiers,
            value,
            span,
        },
        &cx.allocator,
    ))
}

/// Lower one `v-on` spelling into `ui.on`.
pub(crate) fn lower_on<'a>(
    cx: &mut Cx<'a>,
    element: &Element<'a>,
    index: usize,
    directive: &Directive<'a>,
) -> BindingOp<'a> {
    let attr = &element.open.attrs[index];
    let span = attr_span(cx, attr);
    let node = cx.mint_op();
    let name = lower_name(cx, directive.arg);
    let mut modifiers: Vec<'a, &'a str> = Vec::new_in(&cx.allocator);
    for modifier in &directive.modifiers {
        modifiers.push(modifier);
    }
    // Vue 2 v-on event sugar (`.native`, numeric keycodes), mirrored
    // from the shipped per-directive desugar (P2-9 series 7).
    #[cfg(feature = "_legacy")]
    super::legacy::rewrite_on_modifiers(cx, element, index, &mut modifiers);
    let handler = attr_value_text(element, index)
        .filter(|text| !text.trim().is_empty())
        .map(|text| expr_at(cx, text));
    cx.record(
        "lower.on",
        node,
        attr_slice(cx, attr),
        name_desc("ui.on", &name),
        span,
    );
    BindingOp::On(Box::new_in(
        OnOp {
            name,
            modifiers,
            handler,
            span,
        },
        &cx.allocator,
    ))
}
