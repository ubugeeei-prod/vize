//! L2 ops as TypeScript constructs, branching on the op and its facts only.
//!
//! A re-render root is one expression, `([items] as unknown as VNode)`: an
//! array keeps every item in the enclosing scope, so narrowing still applies,
//! and its elements need no comma operator. Items, in authored order:
//!
//! - `ui.bind name` on a native element: `__vizeNativeElementProp<"tag",
//!   "name">((value))`, one cached instantiation per tag and name.
//! - `ui.bind` spread: `({ ...(value) })`, the object spread rule.
//! - `ui.on`: the native element's `onName` prop, checked like a binding.
//! - `ui.jsx-child`: `(value)`.
//! - `ui.if` / `ui.for` authored as script: see [`super::control`].
//!
//! `ui.text` and static attributes write nothing, as today.

use vize_l2::expr::ExprRef;
use vize_l2::module::ComponentDef;
use vize_l2::op::{BindOp, BindingOp, DynamicName, ElementOp, OnHandlerRef, OnOp, Op, Region};

use super::control;
use super::helpers::{NATIVE_ELEMENT_PROP, RE_RENDER_ROOT};
use super::module::{Cx, JsxModuleRefusal};
use super::output::LinkRole;

/// Listener modifiers that leave the handler's type unchanged.
const LISTENER_OPTIONS: [&str; 3] = ["capture", "once", "passive"];

/// Write one root in expression position, anchored to the whole root.
pub(super) fn root(cx: &mut Cx<'_>, root: &ComponentDef<'_>) -> Result<(), JsxModuleRefusal> {
    cx.anchor(root.span, LinkRole::Expression, |cx| {
        cx.out.push("([");
        let mut items = Items::new();
        region(cx, &root.body.root, &mut items)?;
        cx.out.push("] as unknown as ");
        cx.out.push(RE_RENDER_ROOT);
        cx.out.push(")");
        Ok(())
    })
}

/// Separates the items of one array.
pub(super) struct Items {
    first: bool,
}

impl Items {
    pub(super) const fn new() -> Self {
        Self { first: true }
    }

    pub(super) fn next(&mut self, cx: &mut Cx<'_>) {
        if !self.first {
            cx.out.push(", ");
        }
        self.first = false;
    }
}

fn region(cx: &mut Cx<'_>, region: &Region<'_>, items: &mut Items) -> Result<(), JsxModuleRefusal> {
    for op in &region.ops {
        self::op(cx, op, items)?;
    }
    Ok(())
}

fn op(cx: &mut Cx<'_>, op: &Op<'_>, items: &mut Items) -> Result<(), JsxModuleRefusal> {
    match op {
        Op::Text(_) => Ok(()),
        Op::Element(element) => self::element(cx, element, items),
        Op::JsxChildValue(child) => {
            items.next(cx);
            cx.out.push("(");
            expression(cx, &child.expression)?;
            cx.out.push(")");
            Ok(())
        }
        // The owner-object helper for components is not published yet.
        Op::Component(_) => Err(JsxModuleRefusal::UnpublishedHelper(op.mnemonic())),
        Op::If(_) | Op::For(_) => {
            items.next(cx);
            control::child(cx, op)
        }
        Op::Interpolation(_) | Op::Comment(_) | Op::OriginalFor(_) | Op::Slot(_) | Op::And(_) => {
            Err(JsxModuleRefusal::Unsupported(op.mnemonic()))
        }
    }
}

/// A native element's checks and then its children's items.
pub(super) fn element(
    cx: &mut Cx<'_>,
    element: &ElementOp<'_>,
    items: &mut Items,
) -> Result<(), JsxModuleRefusal> {
    for binding in &element.bindings {
        native_binding(cx, element.tag, binding, items)?;
    }
    region(cx, &element.children, items)
}

fn native_binding(
    cx: &mut Cx<'_>,
    tag: &str,
    binding: &BindingOp<'_>,
    items: &mut Items,
) -> Result<(), JsxModuleRefusal> {
    let unsupported = JsxModuleRefusal::Unsupported(binding.mnemonic());
    match binding {
        BindingOp::Bind(bind) => {
            let BindOp {
                name,
                modifiers,
                value: Some(value),
                span,
            } = &**bind
            else {
                return Err(unsupported);
            };
            if !modifiers.is_empty() {
                return Err(unsupported);
            }
            items.next(cx);
            match name {
                None => cx.anchor(*span, LinkRole::Attribute, |cx| {
                    cx.out.push("({ ...(");
                    expression(cx, value)?;
                    cx.out.push(") })");
                    Ok(())
                }),
                Some(DynamicName::Static(name)) => cx.anchor(*span, LinkRole::Attribute, |cx| {
                    native_prop(cx, tag, PropKey::Bound(name), value)
                }),
                Some(DynamicName::Dynamic(_)) => Err(unsupported),
            }
        }
        BindingOp::On(on) => {
            let OnOp {
                name: Some(DynamicName::Static(event)),
                modifiers,
                handler: Some(handler),
                span,
            } = &**on
            else {
                return Err(unsupported);
            };
            if !modifiers
                .iter()
                .all(|modifier| LISTENER_OPTIONS.contains(modifier))
            {
                return Err(unsupported);
            }
            let handler = match handler {
                OnHandlerRef::Js(js) => ExprRef::Js(js),
                OnHandlerRef::Opaque(opaque) => ExprRef::Opaque(opaque),
                OnHandlerRef::Foreign(_) | OnHandlerRef::Filter(_) | OnHandlerRef::Body(_) => {
                    return Err(unsupported);
                }
            };
            items.next(cx);
            cx.anchor(*span, LinkRole::Event, |cx| {
                native_prop(cx, tag, PropKey::Listener(event), &handler)
            })
        }
        BindingOp::Model(_)
        | BindingOp::SlotContent(_)
        | BindingOp::SlotsObject(_)
        | BindingOp::VueDirective(_)
        | BindingOp::VueCssBind(_)
        | BindingOp::VueSync(_)
        | BindingOp::VueSlotScope(_)
        | BindingOp::VueOnce(_)
        | BindingOp::VueMemo(_)
        | BindingOp::VueShow(_)
        | BindingOp::VueHtml(_)
        | BindingOp::VueText(_)
        | BindingOp::VueCloak(_) => Err(unsupported),
    }
}

/// The native element prop a binding sets.
#[derive(Clone, Copy)]
enum PropKey<'n> {
    Bound(&'n str),
    /// The event's handler key, `on` plus the capitalized event name.
    Listener(&'n str),
}

fn native_prop(
    cx: &mut Cx<'_>,
    tag: &str,
    key: PropKey<'_>,
    value: &ExprRef<'_>,
) -> Result<(), JsxModuleRefusal> {
    cx.out.push(NATIVE_ELEMENT_PROP);
    cx.out.push("<");
    quoted(cx, "", tag)?;
    cx.out.push(", ");
    match key {
        PropKey::Bound(name) => quoted(cx, "", name)?,
        PropKey::Listener(event) => {
            let mut chars = event.chars();
            let first = chars.next().ok_or(JsxModuleRefusal::Name)?;
            let mut capitalized = vize_l0::String::default();
            capitalized.extend(first.to_uppercase());
            capitalized.push_str(chars.as_str());
            quoted(cx, "on", &capitalized)?;
        }
    }
    cx.out.push(">((");
    expression(cx, value)?;
    cx.out.push("))");
    Ok(())
}

/// A string literal of `prefix` and `name`, which need no escape.
fn quoted(cx: &mut Cx<'_>, prefix: &str, name: &str) -> Result<(), JsxModuleRefusal> {
    if name.is_empty()
        || name
            .chars()
            .any(|ch| matches!(ch, '"' | '\\' | '\u{2028}' | '\u{2029}') || ch.is_control())
    {
        return Err(JsxModuleRefusal::Name);
    }
    cx.out.push_char('"');
    cx.out.push(prefix);
    cx.out.push(name);
    cx.out.push_char('"');
    Ok(())
}

/// An authored JavaScript expression, verbatim at its own span.
pub(super) fn expression(
    cx: &mut Cx<'_>,
    expression: &ExprRef<'_>,
) -> Result<(), JsxModuleRefusal> {
    let (text, span) = match expression {
        ExprRef::Js(js) => (js.source, js.span),
        ExprRef::Opaque(opaque) => (opaque.source, opaque.span),
        ExprRef::Foreign(_) | ExprRef::Filter(_) => return Err(JsxModuleRefusal::Expression),
    };
    if cx.source.get(span.start as usize..span.end as usize) != Some(text) {
        return Err(JsxModuleRefusal::Expression);
    }
    cx.verbatim(span, LinkRole::Expression)
}
