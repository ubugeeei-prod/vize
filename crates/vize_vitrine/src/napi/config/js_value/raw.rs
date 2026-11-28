//! Small NAPI helpers used by the config normalizer.
//!
//! Keeping the N-API calls here lets the normalizer read like config logic while
//! still avoiding a JSON conversion boundary. All helpers return `Unknown`
//! handles that point at values owned by the active JavaScript environment.
//!
//! Every call goes through napi's safe `Env`, `JsValue` and `JsObjectValue`
//! methods, which issue the same N-API calls the helpers used to make by hand.
//! N-API checks each handle's type itself and reports `napi_object_expected` or
//! `napi_array_expected` for a value of the wrong kind, so viewing a handle
//! through [`object`] never reads memory as the wrong type.

use napi::{
    Env, JsString, JsStringUtf8, JsValue,
    bindgen_prelude::{Error, JsObjectValue, Object, Result, Unknown, sys},
};

/// View a handle as an object so napi's object methods can take it.
fn object<'js>(value: Unknown<'js>) -> Object<'js> {
    Object::from_raw(value.value().env, value.raw())
}

/// One enumerable string key. Its name is decoded only when a caller asks.
#[derive(Clone, Copy)]
pub(super) struct PropertyKey<'js>(JsString<'js>);

impl<'js> PropertyKey<'js> {
    /// The key as a handle for property reads and writes.
    pub(super) fn handle(self) -> Unknown<'js> {
        self.0.to_unknown()
    }

    /// The key's UTF-8 name; `as_str` borrows it.
    pub(super) fn name(self) -> Result<JsStringUtf8<'js>> {
        self.0.into_utf8()
    }
}

pub(super) fn enumerable_keys(value: Unknown<'_>) -> Result<Vec<PropertyKey<'_>>> {
    let keys = object(value).get_property_names()?;
    let mut result = Vec::with_capacity(keys.get_array_length_unchecked()? as usize);

    for index in 0..keys.get_array_length_unchecked()? {
        // The property-name array N-API returned holds only strings.
        result.push(PropertyKey(keys.get_element(index)?));
    }

    Ok(result)
}

pub(super) fn create_object(env: sys::napi_env) -> Result<Unknown<'static>> {
    Ok(Object::new(&Env::from_raw(env))?.to_unknown())
}

pub(super) fn create_array(env: sys::napi_env, len: u32) -> Result<Unknown<'static>> {
    let array = Env::from_raw(env).create_array(len)?.raw();
    Ok(Object::from_raw(env, array).to_unknown())
}

pub(super) fn array_len(value: Unknown<'_>) -> Result<u32> {
    object(value).get_array_length_unchecked()
}

pub(super) fn is_array(value: Unknown<'_>) -> Result<bool> {
    value.is_array()
}

pub(super) fn get_element(value: Unknown<'_>, index: u32) -> Result<Unknown<'_>> {
    object(value).get_element(index)
}

pub(super) fn set_element(array: Unknown<'_>, index: u32, value: Unknown<'_>) -> Result<()> {
    object(array).set_element(index, value)
}

pub(super) fn get_property<'js>(
    object_value: Unknown<'js>,
    key: Unknown<'_>,
) -> Result<Unknown<'js>> {
    object(object_value).get_property_unchecked(key)
}

pub(super) fn set_property(
    object_value: Unknown<'_>,
    key: Unknown<'_>,
    value: Unknown<'_>,
) -> Result<()> {
    object(object_value).set_property(key, value)
}

pub(super) fn get_own_named_property<'js>(
    object: Unknown<'js>,
    name: &str,
) -> Result<Option<Unknown<'js>>> {
    if !has_own_property(object, name)? {
        return Ok(None);
    }
    Ok(Some(get_named_property(object, name)?))
}

pub(super) fn get_named_property<'js>(
    object_value: Unknown<'js>,
    name: &str,
) -> Result<Unknown<'js>> {
    object(object_value).get_named_property_unchecked(name)
}

pub(super) fn set_named_property(
    object_value: Unknown<'_>,
    name: &str,
    value: Unknown<'_>,
) -> Result<()> {
    object(object_value).set_named_property(name, value)
}

pub(super) fn has_own_property(object_value: Unknown<'_>, name: &str) -> Result<bool> {
    object(object_value).has_own_property(name)
}

pub(super) fn get_prototype(value: Unknown<'_>) -> Result<Unknown<'_>> {
    object(value).get_prototype()
}

pub(super) fn object_prototype(env: sys::napi_env) -> Result<Unknown<'static>> {
    let global = Env::from_raw(env).get_global()?;
    let constructor = global.get_named_property_unchecked::<Unknown<'static>>("Object")?;
    get_named_property(constructor, "prototype")
}

pub(super) fn strict_equals(left: Unknown<'_>, right: Unknown<'_>) -> Result<bool> {
    Env::from_raw(left.value().env).strict_equals(left, right)
}

pub(super) fn invalid_arg(message: &str) -> Error {
    Error::new(napi::Status::InvalidArg, message)
}
