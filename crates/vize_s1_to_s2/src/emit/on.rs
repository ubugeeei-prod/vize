//! Static/dynamic `ui.on` emission (`@click`, `@[event]`) with event,
//! key, and option modifiers (`withModifiers`, `withKeys`, `onClickOnce`, …).
//! Object `v-on` lives in [`super::merge`].

mod wrapped;

#[cfg(test)]
mod tests;

use vize_s0::{SmallVec, String, camelize, capitalize};
use vize_s2::expr::{ExprRef, OpaqueReason};
use vize_s2::op::{DynamicName, OnOp};
pub(super) use wrapped::emit_wrapped_handler;
pub(super) use wrapped::needs_handler_cache as caches_handler;

use super::{EmitCx, EmitError, UnsupportedReason as Reason};

// The checked modifier inventory in `tests/tooling/davinci-v-on-storage.test.ts`
// selects two inline entries per classifier bucket. Authored directives remain
// unbounded and spill without changing order or output; the allocation budget in
// `davinci-road/plan/budgets.toml` owns the measured proof.
const OPTION_INLINE_CAP: usize = 2;
const EVENT_INLINE_CAP: usize = 2;
const KEY_INLINE_CAP: usize = 2;

type OptionModifiers<'a> = SmallVec<[&'a str; OPTION_INLINE_CAP]>;
type EventModifiers<'a> = SmallVec<[&'a str; EVENT_INLINE_CAP]>;
type KeyModifiers<'a> = SmallVec<[&'a str; KEY_INLINE_CAP]>;

pub(super) struct Classified<'a> {
    options: OptionModifiers<'a>,
    event: EventModifiers<'a>,
    keys: KeyModifiers<'a>,
}

pub(super) fn admit_on(on: &OnOp<'_>) -> Result<(), EmitError> {
    if on.name.is_none() {
        return super::merge::admit_object_on(on);
    }
    if super::on_dynamic::is_dynamic_on_name(on) {
        return super::on_dynamic::admit(on);
    }
    static_on_name(on)?;
    classify(on)?;
    match on.handler {
        None | Some(ExprRef::Js(_)) => Ok(()),
        Some(ExprRef::Opaque(opaque)) if opaque.reason == OpaqueReason::MultiStatement => Ok(()),
        Some(expr) => Err(EmitError::unsupported_at(
            Reason::OnHandlerNotJs,
            expr.span(),
        )),
    }
}

pub(super) fn static_on_name<'a>(on: &'a OnOp<'a>) -> Result<&'a str, EmitError> {
    match on.name {
        Some(DynamicName::Static(name)) => Ok(name),
        Some(DynamicName::Dynamic(_)) | None => {
            Err(EmitError::unsupported_at(Reason::OnNameNotStatic, on.span))
        }
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
    if super::on_dynamic::is_dynamic_on_name(on) {
        return false;
    }
    if key == "onUpdate:modelValue" || key.starts_with("onVnode") {
        return false;
    }
    if has_native_modifier(on) {
        return true;
    }
    key != "onClick" || classify(on).is_ok_and(|classified| !classified.keys.is_empty())
}

pub(super) fn forces_inline_on(on: &OnOp<'_>) -> bool {
    if super::on_dynamic::is_dynamic_on_name(on) {
        return super::on_dynamic::forces_inline(on);
    }
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
    if super::on_dynamic::is_dynamic_on_name(on) {
        return super::on_dynamic::emit_pair(cx, on, is_plain_element);
    }
    super::js::push_ident_key(cx, event_key_for(on, is_plain_element)?.as_str());
    cx.buf.push(": ");
    emit_on_value(cx, on, is_plain_element)
}

pub(super) fn emit_on_value(
    cx: &mut EmitCx<'_>,
    on: &OnOp<'_>,
    is_plain_element: bool,
) -> Result<(), EmitError> {
    if super::on_dynamic::is_dynamic_on_name(on) {
        return super::on_dynamic::emit_value(cx, on, is_plain_element);
    }
    let classified = classify(on)?;
    emit_wrapped_handler(cx, on, &classified, is_plain_element)
}

pub(super) fn reserve_skipped_once_helpers(
    cx: &mut EmitCx<'_>,
    on: &OnOp<'_>,
) -> Result<(), EmitError> {
    if super::on_dynamic::is_dynamic_on_name(on) {
        return Ok(());
    }
    let classified = classify(on)?;
    if !classified.keys.is_empty() {
        cx.buf.use_with_keys();
    }
    if !classified.event.is_empty() {
        cx.buf.use_with_modifiers();
    }
    Ok(())
}

fn classify<'a>(on: &'a OnOp<'a>) -> Result<Classified<'a>, EmitError> {
    let name = static_on_name(on)?;
    Ok(classify_modifiers(name, on.modifiers.iter().copied()))
}

pub(super) fn classify_dynamic_modifiers<'a>(
    modifiers: impl IntoIterator<Item = &'a str>,
) -> Classified<'a> {
    classify_modifier_buckets(false, false, modifiers)
}

fn classify_modifiers<'a>(
    name: &str,
    modifiers: impl IntoIterator<Item = &'a str>,
) -> Classified<'a> {
    classify_modifier_buckets(
        matches!(name, "keydown" | "keyup" | "keypress"),
        true,
        modifiers,
    )
}

fn classify_modifier_buckets<'a>(
    keyboard: bool,
    keep_options: bool,
    modifiers: impl IntoIterator<Item = &'a str>,
) -> Classified<'a> {
    let mut options = OptionModifiers::new();
    let mut event = EventModifiers::new();
    let mut keys = KeyModifiers::new();
    for modifier in modifiers {
        match modifier {
            // Vue 2's `.native` sugar is stripped before handler wrapping.
            "native" => {}
            "capture" | "once" | "passive" if keep_options => options.push(modifier),
            "capture" | "once" | "passive" => {}
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
