//! Project the canonical parser's existing zero-copy attributes, without rescanning.

use std::borrow::Cow;

use napi::bindgen_prelude::{Error, Result};
use serde_json::{Value, json};
use vize_atelier_sfc::SfcDescriptor;
use vize_l0::FxHashMap;

use super::SfcParsedAttributeNapi;

fn invalid_geometry() -> Error<&'static str> {
    Error::new(
        "InvalidSfcAttributes",
        "Native attribute geometry is unavailable",
    )
}

fn attach(
    value: &mut Value,
    source: &str,
    attrs: &FxHashMap<Cow<'_, str>, Cow<'_, str>>,
) -> Result<(), &'static str> {
    let single = attrs.len() < 2;
    let mut ordered = attrs
        .iter()
        .map(|(name, value)| {
            // Names borrow the original opening tag. Relative addresses retain its
            // authored order even though lookup uses a hash table. Duplicate names
            // keep the parser's retained name position and final effective value.
            // The existing template-only fast path can intern its sole `lang`
            // name. A zero/one-entry map already has an unambiguous order.
            let offset = if single {
                0
            } else {
                let offset = name
                    .as_ptr()
                    .addr()
                    .checked_sub(source.as_ptr().addr())
                    .ok_or_else(invalid_geometry)?;
                let end = offset
                    .checked_add(name.len())
                    .ok_or_else(invalid_geometry)?;
                if source.get(offset..end) != Some(name.as_ref()) {
                    return Err(invalid_geometry());
                }
                offset
            };
            Ok((
                offset,
                SfcParsedAttributeNapi {
                    name: name.to_string(),
                    value: value.to_string(),
                },
            ))
        })
        .collect::<Result<Vec<_>, &'static str>>()?;
    ordered.sort_unstable_by_key(|(offset, _)| *offset);
    value.as_object_mut().ok_or_else(invalid_geometry)?.insert(
        "attrs".into(),
        json!(
            ordered
                .into_iter()
                .map(|(_, attribute)| attribute)
                .collect::<Vec<_>>()
        ),
    );
    Ok(())
}

pub(super) fn include(
    result: &mut Value,
    descriptor: &SfcDescriptor<'_>,
    source: &str,
) -> Result<(), &'static str> {
    for (key, attrs) in [
        (
            "template",
            descriptor.template.as_ref().map(|block| &block.attrs),
        ),
        (
            "script",
            descriptor.script.as_ref().map(|block| &block.attrs),
        ),
        (
            "scriptSetup",
            descriptor.script_setup.as_ref().map(|block| &block.attrs),
        ),
    ] {
        if let Some(attrs) = attrs {
            attach(
                result.get_mut(key).ok_or_else(invalid_geometry)?,
                source,
                attrs,
            )?;
        }
    }
    let styles = result
        .get_mut("styles")
        .and_then(Value::as_array_mut)
        .ok_or_else(invalid_geometry)?;
    for (value, block) in styles.iter_mut().zip(&descriptor.styles) {
        attach(value, source, &block.attrs)?;
    }
    let custom = result
        .get_mut("customBlocks")
        .and_then(Value::as_array_mut)
        .ok_or_else(invalid_geometry)?;
    for (value, block) in custom.iter_mut().zip(&descriptor.custom_blocks) {
        attach(value, source, &block.attrs)?;
    }
    Ok(())
}
