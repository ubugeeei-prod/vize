//! Attributes: `Attribute`, `ui.bind` and `ui.on`, in authored order.

use oxc_ast::ast::{JSXAttribute, JSXAttributeItem, JSXAttributeName, JSXAttributeValue};
use oxc_span::GetSpan;
use vize_l0::{Allocator, Box, Vec};
use vize_l2::artifact::OpFamily;
use vize_l2::expr::ExprRef;
use vize_l2::op::{Attribute, BindOp, BindingOp, DynamicName, OnOp};

use super::children::contains_jsx;
use super::{Cx, JsxRefusal, span};

/// The camelCase spelling of `v-slots`, which the JSX dialects accept too.
const V_SLOTS_ALIAS: &str = "vSlots";

pub(super) struct Props<'a> {
    pub(super) attributes: Vec<'a, Attribute<'a>>,
    pub(super) bindings: Vec<'a, BindingOp<'a>>,
}

impl<'a> Props<'a> {
    pub(super) fn new(allocator: &'a Allocator) -> Self {
        Self {
            attributes: Vec::new_in(&allocator),
            bindings: Vec::new_in(&allocator),
        }
    }

    /// Whether the props name a dynamic component's `is`.
    pub(super) fn has_is(&self) -> bool {
        self.attributes
            .iter()
            .any(|attribute| attribute.name == "is")
            || self.bindings.iter().any(|binding| {
                matches!(binding, BindingOp::Bind(bind)
                    if matches!(bind.name, Some(DynamicName::Static("is"))))
            })
    }
}

pub(super) fn props<'a>(
    cx: &mut Cx<'a, '_>,
    items: &[JSXAttributeItem<'_>],
    out: &mut Props<'a>,
) -> Result<(), JsxRefusal> {
    for item in items {
        match item {
            JSXAttributeItem::Attribute(attribute) => self::attribute(cx, attribute, out)?,
            // `{...e}`: an ordered dynamic props source (`mergeProps`).
            JSXAttributeItem::SpreadAttribute(spread) => {
                if contains_jsx(&spread.argument) {
                    return Err(JsxRefusal::Attribute);
                }
                cx.mint();
                let value = value(cx, spread.argument.span())?;
                out.bindings.push(bind(cx, None, value, spread.span));
            }
        }
    }
    Ok(())
}

fn attribute<'a>(
    cx: &mut Cx<'a, '_>,
    attribute: &JSXAttribute<'_>,
    out: &mut Props<'a>,
) -> Result<(), JsxRefusal> {
    let directive = match &attribute.name {
        JSXAttributeName::Identifier(name) => is_directive(cx.slice(name.span)?),
        JSXAttributeName::NamespacedName(name) => is_directive(cx.slice(name.namespace.span)?),
    };
    if directive {
        return Err(JsxRefusal::Attribute);
    }
    let name = cx.slice(attribute.name.span())?;
    let expression = match &attribute.value {
        None => None,
        Some(JSXAttributeValue::StringLiteral(string)) => {
            out.attributes.push(Attribute {
                name,
                value: Some(cx.allocator.alloc_str(string.value.as_str())),
                span: span(attribute.span),
            });
            return Ok(());
        }
        Some(JSXAttributeValue::ExpressionContainer(container)) => {
            container.expression.as_expression()
        }
        Some(JSXAttributeValue::Element(_) | JSXAttributeValue::Fragment(_)) => {
            return Err(JsxRefusal::Attribute);
        }
    };
    // `a` and `a={}` are boolean attributes.
    let Some(expression) = expression else {
        out.attributes.push(Attribute {
            name,
            value: None,
            span: span(attribute.span),
        });
        return Ok(());
    };
    if contains_jsx(expression) {
        return Err(JsxRefusal::Attribute);
    }
    if cx.dialect.features.event_underscore_modifiers && is_on(name) && name.contains('_') {
        return Err(JsxRefusal::Attribute);
    }
    cx.mint();
    let value = value(cx, expression.span())?;
    if cx.dialect.features.on_option_suffixes
        && let Some((event, modifiers)) = split_option_suffixes(cx.allocator, name)
    {
        out.bindings.push(BindingOp::On(Box::new_in(
            OnOp {
                name: Some(DynamicName::Static(event)),
                modifiers,
                handler: Some(value.into()),
                span: span(attribute.span),
            },
            &cx.allocator,
        )));
        return Ok(());
    }
    if name == "class" {
        cx.observe(OpFamily::JsxClass);
    }
    out.bindings.push(bind(
        cx,
        Some(DynamicName::Static(name)),
        value,
        attribute.span,
    ));
    Ok(())
}

fn value<'a>(cx: &Cx<'a, '_>, at: oxc_span::Span) -> Result<ExprRef<'a>, JsxRefusal> {
    Ok(ExprRef::parse_js_in(cx.allocator, cx.slice(at)?, span(at)))
}

fn bind<'a>(
    cx: &Cx<'a, '_>,
    name: Option<DynamicName<'a>>,
    value: ExprRef<'a>,
    at: oxc_span::Span,
) -> BindingOp<'a> {
    BindingOp::Bind(Box::new_in(
        BindOp {
            name,
            modifiers: Vec::new_in(&cx.allocator),
            value: Some(value),
            span: span(at),
        },
        &cx.allocator,
    ))
}

/// A `v-*` directive or the `vSlots` alias, alone or as a namespace (`vSlots:default`).
fn is_directive(name: &str) -> bool {
    name.starts_with("v-") || name == V_SLOTS_ALIAS
}

/// `on` followed by an uppercase event character.
fn is_on(name: &str) -> bool {
    name.strip_prefix("on")
        .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_uppercase()))
}

/// `onClickCapture` → `click` with `[capture]`; `onInputPassiveCapture` →
/// `input` with `[passive, capture]`. A name without a suffix, or made only
/// of suffixes (`onCapture`), is an ordinary prop.
fn split_option_suffixes<'a>(
    allocator: &'a Allocator,
    name: &str,
) -> Option<(&'a str, Vec<'a, &'a str>)> {
    if !is_on(name) {
        return None;
    }
    let mut event = name.get(2..)?;
    let mut reversed = Vec::new_in(&allocator);
    loop {
        let (head, modifier) = if let Some(head) = event.strip_suffix("Capture") {
            (head, "capture")
        } else if let Some(head) = event.strip_suffix("Once") {
            (head, "once")
        } else if let Some(head) = event.strip_suffix("Passive") {
            (head, "passive")
        } else {
            break;
        };
        event = head;
        reversed.push(modifier);
    }
    if reversed.is_empty() || event.is_empty() {
        return None;
    }
    let mut modifiers = Vec::new_in(&allocator);
    for modifier in reversed.iter().rev() {
        modifiers.push(*modifier);
    }
    let mut chars = event.chars();
    let first = chars.next()?;
    let mut lowered = vize_l0::String::default();
    lowered.push(first.to_ascii_lowercase());
    lowered.push_str(chars.as_str());
    Some((allocator.alloc_str(&lowered), modifiers))
}
