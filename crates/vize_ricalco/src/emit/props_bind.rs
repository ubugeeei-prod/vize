//! Static-name `ui.bind` accessors shared by props admission and object emit.

use vize_carton::{String, camelize};
use vize_disegno::expr::{ExprRef, JsExpr};
use vize_disegno::op::{BindOp, DynamicName};

use super::EmitError;

/// Whether static bind keys should use their ordinary casing or the
/// `<slot>` outlet casing rule.
#[derive(Clone, Copy)]
pub(super) enum StaticBindKeyCasing {
    Preserve,
    Camelize,
}

/// The DOM prop key after static `v-bind` modifiers have been realized.
pub(super) enum StaticBindKey<'a> {
    Borrowed(&'a str),
    Owned(String),
}

impl StaticBindKey<'_> {
    pub(super) fn as_str(&self) -> &str {
        match self {
            Self::Borrowed(text) => text,
            Self::Owned(text) => text.as_str(),
        }
    }
}

pub(super) fn static_bind_name<'a>(bind: &'a BindOp<'a>) -> Result<&'a str, EmitError> {
    match bind.name {
        Some(DynamicName::Static(name)) => Ok(name),
        Some(DynamicName::Dynamic(_)) | None => Err(EmitError::Unsupported),
    }
}

pub(super) fn static_bind_key<'a>(
    bind: &'a BindOp<'a>,
    casing: StaticBindKeyCasing,
) -> Result<StaticBindKey<'a>, EmitError> {
    let raw = static_bind_name(bind)?;
    let mods = StaticBindModifiers::of(bind);
    let mut key = if mods.camel || matches!(casing, StaticBindKeyCasing::Camelize) {
        StaticBindKey::Owned(camelize(raw))
    } else {
        StaticBindKey::Borrowed(raw)
    };
    if mods.prop {
        key = prefixed('.', key);
    } else if mods.attr {
        key = prefixed('^', key);
    }
    Ok(key)
}

pub(super) fn has_prop_modifier(bind: &BindOp<'_>) -> bool {
    StaticBindModifiers::of(bind).prop
}

pub(super) fn js_value<'a>(bind: &'a BindOp<'a>) -> Result<&'a JsExpr<'a>, EmitError> {
    match bind.value {
        Some(ExprRef::Js(js)) => Ok(js),
        _ => Err(EmitError::Unsupported),
    }
}

struct StaticBindModifiers {
    camel: bool,
    prop: bool,
    attr: bool,
}

impl StaticBindModifiers {
    fn of(bind: &BindOp<'_>) -> Self {
        let mut out = Self {
            camel: false,
            prop: false,
            attr: false,
        };
        for modifier in bind.modifiers.iter() {
            match *modifier {
                "camel" => out.camel = true,
                "prop" => out.prop = true,
                "attr" => out.attr = true,
                _ => {}
            }
        }
        out
    }
}

fn prefixed(prefix: char, key: StaticBindKey<'_>) -> StaticBindKey<'_> {
    let key = key.as_str();
    let mut text = String::with_capacity(prefix.len_utf8() + key.len());
    text.push(prefix);
    text.push_str(key);
    StaticBindKey::Owned(text)
}
