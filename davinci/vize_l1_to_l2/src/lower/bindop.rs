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

use vize_l0::{Allocator, Box, String, Vec, cstr};
use vize_l1::markup::Attribute;

use vize_l2::expr::OpaqueReason;
use vize_l2::op::{BindOp, BindingOp, DynamicName, OnOp, VueSyncOp};
use vize_l2::provenance::ProvenanceSink;

use super::cx::{Cx, attr_slice, attr_span};
use super::directive::{Arg, Directive};
use super::element::attr_text;
use super::expr::{desc, expr_at, filter_expr_at, handler_expr_at, opaque_at};

/// The provenance rule of the same-name expansion.
pub(crate) const RULE_SAME_NAME: &str = "normalize.bind.same-name";

/// What a `v-bind` value position reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindValue<'a> {
    /// No expression: an authored-blank value, or no value and no static
    /// argument to expand.
    None,
    /// The authored value text.
    Authored(&'a str),
    /// The same-name shorthand: the camelized static `arg` as `text`.
    SameName { text: &'a str, arg: &'a str },
}

/// The value a `v-bind` binds, from its authored value text and its static
/// argument. An authored-blank value (`:x=""`) materializes no expression
/// in the shipped parser (and blocks the same-name expansion, which
/// requires no value position at all) — mirrored exactly, caught by the
/// corpus lane on the first run (directus `@contextmenu.stop=""`).
pub fn bind_value<'a>(
    allocator: &'a Allocator,
    value: Option<&'a str>,
    static_arg: Option<&'a str>,
) -> BindValue<'a> {
    match (value, static_arg) {
        (Some(text), _) if text.trim().is_empty() => BindValue::None,
        (Some(text), _) => BindValue::Authored(text),
        (None, Some(arg)) => {
            let camel = vize_l0::camelize(arg);
            let text: &'a str = if camel.as_str() == arg {
                arg
            } else {
                allocator.alloc_str(camel.as_str())
            };
            BindValue::SameName { text, arg }
        }
        (None, None) => BindValue::None,
    }
}

/// The op's name position from the directive's argument.
fn lower_name<'a>(
    cx: &mut Cx<'a, impl ProvenanceSink>,
    arg: Option<Arg<'a>>,
) -> Option<DynamicName<'a>> {
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

/// Lower one `v-bind` spelling into `ui.bind`.
pub(crate) fn lower_bind<'a>(
    cx: &mut Cx<'a, impl ProvenanceSink>,
    attr: &Attribute<'a>,
    directive: &Directive<'a>,
) -> BindingOp<'a> {
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

    let static_arg = match directive.arg {
        Some(Arg::Static(text)) => Some(text),
        _ => None,
    };
    let value = match bind_value(cx.allocator, attr_text(attr), static_arg) {
        BindValue::None => None,
        BindValue::Authored(text) => Some(filter_expr_at(cx, text)),
        // Same-name shorthand: static argument, no authored value.
        // The synthesized text is not a source slice, so the
        // expression is admitted at the argument's own span.
        BindValue::SameName { text, arg } => {
            let expr = super::expr::expression_in(cx, text, cx.span_of(arg));
            cx.record(
                RULE_SAME_NAME,
                node,
                attr_slice(cx, attr),
                || cstr!("{text}"),
                span,
            );
            Some(expr)
        }
    };

    // Vue 2's bounded `.sync` subset: static name, a value, the
    // `sync` modifier. Dynamic `:[foo].sync` stays `ui.bind`, matching
    // the shipped pre-transform's skip. `ExprRef`/`DynamicName` are
    // `Copy`, so the Bind arm still sees the original positions.
    if cx.caps.scoped_slot_attrs
        && let Some(DynamicName::Static(name)) = name
        && let Some(value) = value
        && modifiers.contains(&"sync")
    {
        let mut rest: Vec<'a, &'a str> = Vec::new_in(&cx.allocator);
        let mut stripped = false;
        for modifier in &modifiers {
            if *modifier == "sync" && !stripped {
                stripped = true;
                continue;
            }
            rest.push(*modifier);
        }
        cx.record(
            "lower.sync",
            node,
            attr_slice(cx, attr),
            || name_desc("vue.sync", &Some(DynamicName::Static(name))),
            span,
        );
        return BindingOp::VueSync(Box::new_in(
            VueSyncOp {
                name,
                modifiers: rest,
                value,
                span,
            },
            &cx.allocator,
        ));
    }

    cx.record(
        "lower.bind",
        node,
        attr_slice(cx, attr),
        || name_desc("ui.bind", &name),
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
    cx: &mut Cx<'a, impl ProvenanceSink>,
    attr: &Attribute<'a>,
    directive: &Directive<'a>,
) -> BindingOp<'a> {
    let span = attr_span(cx, attr);
    let node = cx.mint_op();
    let name = lower_name(cx, directive.arg);
    let mut modifiers: Vec<'a, &'a str> = Vec::new_in(&cx.allocator);
    for modifier in &directive.modifiers {
        modifiers.push(modifier);
    }
    let handler = attr_text(attr)
        .filter(|text| !text.trim().is_empty())
        .map(|text| {
            let expression = handler_expr_at(cx, text);
            if name.is_none() && super::directive_values_are_clauses(cx) {
                opaque_at(
                    cx,
                    OpaqueReason::DirectiveClause,
                    expression.source(),
                    expression.span(),
                )
            } else {
                expression
            }
        });
    cx.record(
        "lower.on",
        node,
        attr_slice(cx, attr),
        || name_desc("ui.on", &name),
        span,
    );
    BindingOp::On(Box::new_in(
        OnOp {
            name,
            modifiers,
            handler: handler.map(Into::into),
            span,
        },
        &cx.allocator,
    ))
}
