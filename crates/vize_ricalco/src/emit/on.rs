//! Static-name `ui.on` (`@click` / `v-on:click`), including event / key /
//! option modifiers (`withModifiers` / `withKeys`, `onClickOnce`, …).
//! Object `v-on` lives in [`super::merge`].

use oxc_ast::ast::{ChainElement, Expression};
use vize_s0::{SmallVec, String, camelize, capitalize};
use vize_s2::expr::{ExprRef, JsExpr};
use vize_s2::op::{DynamicName, OnOp};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;

// The committed Vue corpus and emitter fixtures currently peak at two
// modifiers in each bucket (242 modifier-bearing `v-on` spellings sampled).
// Authored directives remain unbounded: `SmallVec` spills beyond these common
// two-entry shapes without changing order or output.
const OPTION_INLINE_CAP: usize = 2;
const EVENT_INLINE_CAP: usize = 2;
const KEY_INLINE_CAP: usize = 2;

type OptionModifiers<'a> = SmallVec<[&'a str; OPTION_INLINE_CAP]>;
type EventModifiers<'a> = SmallVec<[&'a str; EVENT_INLINE_CAP]>;
type KeyModifiers<'a> = SmallVec<[&'a str; KEY_INLINE_CAP]>;

struct Classified<'a> {
    options: OptionModifiers<'a>,
    event: EventModifiers<'a>,
    keys: KeyModifiers<'a>,
}

pub(super) fn admit_on(on: &OnOp<'_>) -> Result<(), EmitError> {
    if on.name.is_none() {
        return super::merge::admit_object_on(on);
    }
    static_on_name(on)?;
    classify(on)?;
    match on.handler {
        None | Some(ExprRef::Js(_)) => Ok(()),
        Some(_) => Err(EmitError::Unsupported),
    }
}

pub(super) fn static_on_name<'a>(on: &'a OnOp<'a>) -> Result<&'a str, EmitError> {
    match on.name {
        Some(DynamicName::Static(name)) => Ok(name),
        Some(DynamicName::Dynamic(_)) | None => Err(EmitError::Unsupported),
    }
}

/// Mirror `von_event_key_for`: camelize unless a user-authored event on a
/// plain element keeps uppercase (`on:customEvent`). `vue:` rewrites to
/// `vnode-` first so `@vue:mounted` becomes `onVnodeMounted`.
pub(super) fn event_key(raw: &str, is_plain_element: bool) -> String {
    let mut vnode_owned = String::default();
    let raw_name = if let Some(rest) = raw.strip_prefix("vue:") {
        vnode_owned.push_str("vnode-");
        vnode_owned.push_str(rest);
        vnode_owned.as_str()
    } else {
        raw
    };
    if !is_plain_element
        || raw_name.starts_with("vnode")
        || !raw_name.chars().any(|c| c.is_ascii_uppercase())
    {
        let camelized = camelize(raw_name);
        let mut key = String::with_capacity(camelized.len() + 2);
        key.push_str("on");
        key.push_str(capitalize(camelized.as_str()).as_str());
        key
    } else {
        let mut key = String::with_capacity(raw_name.len() + 3);
        key.push_str("on:");
        key.push_str(raw_name);
        key
    }
}

pub(super) fn event_key_for(on: &OnOp<'_>, is_plain_element: bool) -> Result<String, EmitError> {
    let classified = classify(on)?;
    let mut key = event_key(
        remapped_name(static_on_name(on)?, &classified.event),
        is_plain_element,
    );
    for option in &classified.options {
        key.push_str(capitalize(option).as_str());
    }
    Ok(key)
}

pub(super) fn needs_hydration(key: &str, on: &OnOp<'_>) -> bool {
    if key == "onUpdate:modelValue" || key.starts_with("onVnode") {
        return false;
    }
    if has_native_modifier(on) {
        return true;
    }
    key != "onClick" || classify(on).is_ok_and(|classified| !classified.keys.is_empty())
}

pub(super) fn forces_inline_on(on: &OnOp<'_>) -> bool {
    classify(on).is_ok_and(|classified| {
        has_native_modifier(on) || !classified.event.is_empty() || !classified.keys.is_empty()
    })
}

pub(super) fn is_inline_handler_source(source: &str) -> bool {
    source.contains('(')
        || source.contains('+')
        || source.contains('-')
        || source.contains('=')
        || source.contains(' ')
}

pub(super) fn emit_on_pair(
    cx: &mut EmitCx<'_>,
    on: &OnOp<'_>,
    is_plain_element: bool,
) -> Result<(), EmitError> {
    super::js::push_ident_key(cx, event_key_for(on, is_plain_element)?.as_str());
    cx.buf.push(": ");
    emit_on_value(cx, on)
}

pub(super) fn emit_on_value(cx: &mut EmitCx<'_>, on: &OnOp<'_>) -> Result<(), EmitError> {
    let classified = classify(on)?;
    emit_wrapped_handler(cx, on, &classified)
}

fn emit_wrapped_handler(
    cx: &mut EmitCx<'_>,
    on: &OnOp<'_>,
    classified: &Classified<'_>,
) -> Result<(), EmitError> {
    if !classified.keys.is_empty() {
        cx.buf.use_with_keys();
        cx.buf.push(Buf::with_keys_alias());
        cx.buf.push("(");
    }
    if !classified.event.is_empty() {
        cx.buf.use_with_modifiers();
        cx.buf.push(Buf::with_modifiers_alias());
        cx.buf.push("(");
    }
    match on.handler {
        Some(ExprRef::Js(js)) => emit_handler(cx, js),
        None => cx.buf.push("() => {}"),
        Some(_) => return Err(EmitError::Unsupported),
    }
    if !classified.event.is_empty() {
        cx.buf.push(", ");
        emit_mod_array(cx, &classified.event);
        cx.buf.push(")");
    }
    if !classified.keys.is_empty() {
        cx.buf.push(", ");
        emit_mod_array(cx, &classified.keys);
        cx.buf.push(")");
    }
    Ok(())
}

fn emit_mod_array(cx: &mut EmitCx<'_>, mods: &[&str]) {
    cx.buf.push("[");
    for (i, modifier) in mods.iter().enumerate() {
        if i > 0 {
            cx.buf.push(",");
        }
        cx.buf.push("\"");
        cx.buf.push(modifier);
        cx.buf.push("\"");
    }
    cx.buf.push("]");
}

fn emit_handler(cx: &mut EmitCx<'_>, js: &JsExpr<'_>) {
    if is_handler_reference(js.ast) || is_function(js.ast) {
        cx.buf.push(js.source);
        return;
    }
    if js.source.contains(';') {
        cx.buf.push("$event => {");
        cx.buf.push(js.source);
        cx.buf.push("}");
    } else {
        cx.buf.push("$event => (");
        cx.buf.push(js.source);
        cx.buf.push(")");
    }
}

fn classify<'a>(on: &'a OnOp<'a>) -> Result<Classified<'a>, EmitError> {
    let name = static_on_name(on)?;
    Ok(classify_modifiers(name, on.modifiers.iter().copied()))
}

fn classify_modifiers<'a>(
    name: &str,
    modifiers: impl IntoIterator<Item = &'a str>,
) -> Classified<'a> {
    let keyboard = matches!(name, "keydown" | "keyup" | "keypress");
    let mut options = OptionModifiers::new();
    let mut event = EventModifiers::new();
    let mut keys = KeyModifiers::new();
    for modifier in modifiers {
        match modifier {
            // Vue 2's `.native` event sugar is stripped by the shipped
            // lane before handler wrapping, and does not affect the event
            // key. Keep the authored modifier accepted but inert here.
            "native" => {}
            "capture" | "once" | "passive" => options.push(modifier),
            "left" | "right" if keyboard => keys.push(modifier),
            "stop" | "prevent" | "self" | "ctrl" | "shift" | "alt" | "meta" | "middle"
            | "exact" | "left" | "right" => event.push(modifier),
            _ => keys.push(modifier),
        }
    }
    Classified {
        options,
        event,
        keys,
    }
}

fn has_native_modifier(on: &OnOp<'_>) -> bool {
    on.modifiers.contains(&"native")
}

fn remapped_name<'a>(raw: &'a str, event: &[&str]) -> &'a str {
    if raw == "click" && event.contains(&"right") {
        "contextmenu"
    } else if raw == "click" && event.contains(&"middle") {
        "mouseup"
    } else {
        raw
    }
}

fn is_handler_reference(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::Identifier(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::PrivateFieldExpression(_) => true,
        Expression::ChainExpression(chain) => matches!(
            chain.expression,
            ChainElement::StaticMemberExpression(_) | ChainElement::ComputedMemberExpression(_)
        ),
        _ => false,
    }
}

fn is_function(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
    )
}

#[cfg(test)]
mod tests {
    use super::classify_modifiers;

    #[test]
    fn common_two_modifier_buckets_stay_inline() {
        let classified = classify_modifiers(
            "keyup",
            ["capture", "once", "stop", "prevent", "enter", "escape"],
        );

        assert!(!classified.options.spilled());
        assert!(!classified.event.spilled());
        assert!(!classified.keys.spilled());
    }

    #[test]
    fn authored_modifiers_spill_without_a_length_ceiling() {
        let classified = classify_modifiers(
            "keyup",
            [
                "capture", "once", "passive", "stop", "prevent", "self", "enter", "escape", "space",
            ],
        );

        assert!(classified.options.spilled());
        assert!(classified.event.spilled());
        assert!(classified.keys.spilled());
        assert_eq!(
            classified.options.as_slice(),
            ["capture", "once", "passive"]
        );
        assert_eq!(classified.event.as_slice(), ["stop", "prevent", "self"]);
        assert_eq!(classified.keys.as_slice(), ["enter", "escape", "space"]);
    }
}
