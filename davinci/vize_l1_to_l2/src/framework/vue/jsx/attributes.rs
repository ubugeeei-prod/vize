//! Attributes: `Attribute`, `ui.bind`, `ui.on` and the `v-*` ops of
//! [`super::directives`], in authored order.

use oxc_ast::ast::{JSXAttribute, JSXAttributeItem, JSXAttributeName, JSXAttributeValue};
use oxc_span::GetSpan;
use vize_l0::{Allocator, Box, Vec};
use vize_l2::artifact::OpFamily;
use vize_l2::expr::ExprRef;
use vize_l2::op::{Attribute, BindOp, BindingOp, DynamicName, OnOp};

use super::children::contains_jsx;
use super::directives::directive;
use super::slots::{SlotsAttribute, V_SLOTS, slots_attribute, slots_object};
use super::{Cx, JsxRefusal, span};

/// Whose attributes these are: a directive lowers by its owner's kind.
#[derive(Clone, Copy)]
pub(super) enum Owner<'a, 'r, 'x> {
    /// An intrinsic element; a native `v-model` reads its `type`.
    Element {
        tag: &'a str,
        items: &'r [JSXAttributeItem<'x>],
    },
    Component,
}

pub(super) struct Props<'a> {
    pub(super) attributes: Vec<'a, Attribute<'a>>,
    pub(super) bindings: Vec<'a, BindingOp<'a>>,
    /// The `v-slots` attribute, which lowers after the other props.
    slots: Option<SlotsAttribute>,
}

impl<'a> Props<'a> {
    pub(super) fn new(allocator: &'a Allocator) -> Self {
        Self {
            attributes: Vec::new_in(&allocator),
            bindings: Vec::new_in(&allocator),
            slots: None,
        }
    }

    /// Whether a `v-slots` object appends templates to the children.
    pub(super) const fn has_slot_templates(&self) -> bool {
        matches!(self.slots, Some(SlotsAttribute::Templates))
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
    owner: Owner<'a, '_, '_>,
    out: &mut Props<'a>,
) -> Result<(), JsxRefusal> {
    for item in items {
        match item {
            JSXAttributeItem::Attribute(attribute) => self::attribute(cx, attribute, owner, out)?,
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
    if let Some(SlotsAttribute::Forwarded(at)) = out.slots {
        cx.mint();
        out.bindings.push(slots_object(cx, at)?);
    }
    Ok(())
}

fn attribute<'a>(
    cx: &mut Cx<'a, '_>,
    attribute: &JSXAttribute<'_>,
    owner: Owner<'a, '_, '_>,
    out: &mut Props<'a>,
) -> Result<(), JsxRefusal> {
    let head = match &attribute.name {
        JSXAttributeName::Identifier(name) => cx.slice(name.span)?,
        JSXAttributeName::NamespacedName(name) => cx.slice(name.namespace.span)?,
    };
    if V_SLOTS.contains(&head) {
        let slots = slots_attribute(attribute, owner)?;
        return match out.slots.replace(slots) {
            None => Ok(()),
            Some(_) => Err(JsxRefusal::Attribute),
        };
    }
    if head.starts_with("v-") {
        return directive(cx, attribute, owner, out);
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
        cx.observe(OpFamily::RuntimeClass);
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
