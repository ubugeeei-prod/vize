//! JavaScript-value preserving config normalization for the public NAPI API.
//!
//! Configs loaded from `.ts`, `.js`, and `.mjs` may contain values that cannot
//! round-trip through JSON, most notably `RegExp` instances used by Vite-style
//! include/exclude filters. The normalization below therefore copies only the
//! public config container objects and arrays while preserving every nested
//! non-plain JavaScript value by reference.

mod raw;

use napi::{
    JsValue,
    bindgen_prelude::{Result, Unknown, ValueType, sys},
};

use raw::*;

const ENTRY_METADATA_KEYS: &[&str] = &["name", "basePath", "files", "ignores", "extends"];

/// Normalize a raw JavaScript config export into the resolved public shape.
pub fn normalize_vize_config(value: Unknown<'_>) -> Result<Unknown<'_>> {
    match strip_nullish(value)? {
        Some(value) if is_array(value)? => normalize_config_entries(value),
        Some(value) if is_plain_object(value)? => normalize_config_object(value),
        Some(value) => empty_config(value.value().env),
        None => empty_config(value.value().env),
    }
}

fn normalize_config_object(config: Unknown<'_>) -> Result<Unknown<'_>> {
    let env = config.value().env;
    let config = normalize_config_aliases(config)?;
    let raw_entries = get_own_named_property(config, "entries")?;
    let root_entry = copy_without_keys(config, &["entries"])?;
    let entries = create_array(env, 0)?;
    let mut entry_count = 0;

    if !is_empty_object(root_entry)? {
        set_element(entries, entry_count, root_entry)?;
        entry_count += 1;
    }

    if let Some(raw_entries) = raw_entries {
        if !is_array(raw_entries)? {
            return Err(invalid_arg("config.entries must be an array when provided"));
        }

        for index in 0..array_len(raw_entries)? {
            let entry = normalize_entry(get_element(raw_entries, index)?)?;
            set_element(entries, entry_count, entry)?;
            entry_count += 1;
        }
    }

    let resolved = clone_object(root_entry)?;
    set_named_property(resolved, "entries", entries)?;
    Ok(resolved)
}

fn normalize_config_entries(raw_entries: Unknown<'_>) -> Result<Unknown<'_>> {
    let env = raw_entries.value().env;
    let global_config = create_object(env)?;
    let entries = create_array(env, 0)?;
    let mut entry_count = 0;

    for index in 0..array_len(raw_entries)? {
        let entry = normalize_entry(get_element(raw_entries, index)?)?;
        if is_global_config_entry(entry)? {
            deep_merge(global_config, strip_entry_metadata(entry)?)?;
        }
        set_element(entries, entry_count, entry)?;
        entry_count += 1;
    }

    set_named_property(global_config, "entries", entries)?;
    Ok(global_config)
}

fn normalize_entry(entry: Unknown<'_>) -> Result<Unknown<'_>> {
    match strip_nullish(entry)? {
        Some(entry) if is_plain_object(entry)? => normalize_config_aliases(entry),
        Some(_) => Err(invalid_arg("config entries must be objects")),
        None => create_object(entry.value().env),
    }
}

fn normalize_config_aliases(config: Unknown<'_>) -> Result<Unknown<'_>> {
    let env = config.value().env;
    let output = create_object(env)?;
    let mut lsp = None;
    let mut has_language_server = false;

    for (key, key_value) in enumerable_keys(config)? {
        let value = get_property(config, key_value)?;
        match key.as_str() {
            "lsp" => lsp = Some(value),
            "languageServer" => {
                has_language_server = true;
                set_property(output, key_value, value)?;
            }
            _ => set_property(output, key_value, value)?,
        }
    }

    if let Some(lsp) = lsp
        && !has_language_server
    {
        set_named_property(output, "languageServer", lsp)?;
    }

    Ok(output)
}

fn strip_nullish(value: Unknown<'_>) -> Result<Option<Unknown<'_>>> {
    match value.get_type()? {
        ValueType::Null | ValueType::Undefined => Ok(None),
        ValueType::Object if is_array(value)? => strip_array(value).map(Some),
        ValueType::Object if is_plain_object(value)? => strip_object(value).map(Some),
        _ => Ok(Some(value)),
    }
}

fn strip_array(value: Unknown<'_>) -> Result<Unknown<'_>> {
    let env = value.value().env;
    let output = create_array(env, 0)?;
    let mut next_index = 0;

    for index in 0..array_len(value)? {
        if let Some(entry) = strip_nullish(get_element(value, index)?)? {
            set_element(output, next_index, entry)?;
            next_index += 1;
        }
    }

    Ok(output)
}

fn strip_object(value: Unknown<'_>) -> Result<Unknown<'_>> {
    let output = create_object(value.value().env)?;

    for (_, key_value) in enumerable_keys(value)? {
        if let Some(entry) = strip_nullish(get_property(value, key_value)?)? {
            set_property(output, key_value, entry)?;
        }
    }

    Ok(output)
}

fn deep_merge(target: Unknown<'_>, source: Unknown<'_>) -> Result<()> {
    for (_, key_value) in enumerable_keys(source)? {
        let value = get_property(source, key_value)?;
        let current = get_property(target, key_value)?;
        if is_plain_object(current)? && is_plain_object(value)? {
            deep_merge(current, value)?;
        } else {
            set_property(target, key_value, value)?;
        }
    }
    Ok(())
}

fn strip_entry_metadata(entry: Unknown<'_>) -> Result<Unknown<'_>> {
    copy_without_keys(entry, ENTRY_METADATA_KEYS)
}

fn copy_without_keys<'js>(value: Unknown<'js>, ignored: &[&str]) -> Result<Unknown<'js>> {
    let output = create_object(value.value().env)?;
    for (key, key_value) in enumerable_keys(value)? {
        if !ignored.contains(&key.as_str()) {
            set_property(output, key_value, get_property(value, key_value)?)?;
        }
    }
    Ok(output)
}

fn clone_object(value: Unknown<'_>) -> Result<Unknown<'_>> {
    copy_without_keys(value, &[])
}

fn is_global_config_entry(entry: Unknown<'_>) -> Result<bool> {
    Ok(!has_own_property(entry, "basePath")?
        && !has_own_property(entry, "files")?
        && !has_own_property(entry, "ignores")?)
}

fn is_empty_object(value: Unknown<'_>) -> Result<bool> {
    Ok(enumerable_keys(value)?.is_empty())
}

fn is_plain_object(value: Unknown<'_>) -> Result<bool> {
    if value.get_type()? != ValueType::Object || is_array(value)? {
        return Ok(false);
    }

    let prototype = get_prototype(value)?;
    if prototype.get_type()? == ValueType::Null {
        return Ok(true);
    }

    strict_equals(prototype, object_prototype(value.value().env)?)
}

fn empty_config(env: sys::napi_env) -> Result<Unknown<'static>> {
    let output = create_object(env)?;
    set_named_property(output, "entries", create_array(env, 0)?)?;
    Ok(output)
}
