//! Small NAPI helpers used by the config normalizer.
//!
//! Keeping the raw calls here lets the normalizer read like config logic while
//! still avoiding a JSON conversion boundary. All helpers return `Unknown`
//! handles that point at values owned by the active JavaScript environment.

use std::ffi::CString;

use napi::{
    JsValue,
    bindgen_prelude::{Error, FromNapiValue, Result, Unknown, check_status, sys},
};

pub(super) fn enumerable_keys(value: Unknown<'_>) -> Result<Vec<(String, sys::napi_value)>> {
    let env = value.value().env;
    let mut keys = std::ptr::null_mut();
    check_status!(unsafe { sys::napi_get_property_names(env, value.raw(), &mut keys) })?;
    let keys = unsafe { Unknown::from_raw_unchecked(env, keys) };
    let mut result = Vec::with_capacity(array_len(keys)? as usize);

    for index in 0..array_len(keys)? {
        let key_value = get_element(keys, index)?.raw();
        let key = unsafe { String::from_napi_value(env, key_value)? };
        result.push((key, key_value));
    }

    Ok(result)
}

pub(super) fn create_object(env: sys::napi_env) -> Result<Unknown<'static>> {
    let mut value = std::ptr::null_mut();
    check_status!(unsafe { sys::napi_create_object(env, &mut value) })?;
    Ok(unsafe { Unknown::from_raw_unchecked(env, value) })
}

pub(super) fn create_array(env: sys::napi_env, len: u32) -> Result<Unknown<'static>> {
    let mut value = std::ptr::null_mut();
    check_status!(unsafe { sys::napi_create_array_with_length(env, len as usize, &mut value) })?;
    Ok(unsafe { Unknown::from_raw_unchecked(env, value) })
}

pub(super) fn array_len(value: Unknown<'_>) -> Result<u32> {
    let mut len = 0;
    check_status!(unsafe { sys::napi_get_array_length(value.value().env, value.raw(), &mut len) })?;
    Ok(len)
}

pub(super) fn is_array(value: Unknown<'_>) -> Result<bool> {
    let mut result = false;
    check_status!(unsafe { sys::napi_is_array(value.value().env, value.raw(), &mut result) })?;
    Ok(result)
}

pub(super) fn get_element(value: Unknown<'_>, index: u32) -> Result<Unknown<'_>> {
    let env = value.value().env;
    let mut result = std::ptr::null_mut();
    check_status!(unsafe { sys::napi_get_element(env, value.raw(), index, &mut result) })?;
    Ok(unsafe { Unknown::from_raw_unchecked(env, result) })
}

pub(super) fn set_element(array: Unknown<'_>, index: u32, value: Unknown<'_>) -> Result<()> {
    check_status!(unsafe {
        sys::napi_set_element(array.value().env, array.raw(), index, value.raw())
    })
}

pub(super) fn get_property<'js>(
    object: Unknown<'js>,
    key: sys::napi_value,
) -> Result<Unknown<'js>> {
    let env = object.value().env;
    let mut result = std::ptr::null_mut();
    check_status!(unsafe { sys::napi_get_property(env, object.raw(), key, &mut result) })?;
    Ok(unsafe { Unknown::from_raw_unchecked(env, result) })
}

pub(super) fn set_property(
    object: Unknown<'_>,
    key: sys::napi_value,
    value: Unknown<'_>,
) -> Result<()> {
    check_status!(unsafe {
        sys::napi_set_property(object.value().env, object.raw(), key, value.raw())
    })
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

pub(super) fn get_named_property<'js>(object: Unknown<'js>, name: &str) -> Result<Unknown<'js>> {
    let env = object.value().env;
    let name = CString::new(name)?;
    let mut result = std::ptr::null_mut();
    check_status!(unsafe {
        sys::napi_get_named_property(env, object.raw(), name.as_ptr(), &mut result)
    })?;
    Ok(unsafe { Unknown::from_raw_unchecked(env, result) })
}

pub(super) fn set_named_property(
    object: Unknown<'_>,
    name: &str,
    value: Unknown<'_>,
) -> Result<()> {
    let name = CString::new(name)?;
    check_status!(unsafe {
        sys::napi_set_named_property(object.value().env, object.raw(), name.as_ptr(), value.raw())
    })
}

pub(super) fn has_own_property(object: Unknown<'_>, name: &str) -> Result<bool> {
    let env = object.value().env;
    let mut result = false;
    let mut key = std::ptr::null_mut();
    check_status!(unsafe {
        sys::napi_create_string_utf8(env, name.as_ptr().cast(), name.len() as isize, &mut key)
    })?;
    check_status!(unsafe { sys::napi_has_own_property(env, object.raw(), key, &mut result) })?;
    Ok(result)
}

pub(super) fn get_prototype(value: Unknown<'_>) -> Result<Unknown<'_>> {
    let env = value.value().env;
    let mut result = std::ptr::null_mut();
    check_status!(unsafe { sys::napi_get_prototype(env, value.raw(), &mut result) })?;
    Ok(unsafe { Unknown::from_raw_unchecked(env, result) })
}

pub(super) fn object_prototype(env: sys::napi_env) -> Result<Unknown<'static>> {
    let mut global = std::ptr::null_mut();
    check_status!(unsafe { sys::napi_get_global(env, &mut global) })?;
    let global = unsafe { Unknown::from_raw_unchecked(env, global) };
    get_named_property(get_named_property(global, "Object")?, "prototype")
}

pub(super) fn strict_equals(left: Unknown<'_>, right: Unknown<'_>) -> Result<bool> {
    let mut result = false;
    check_status!(unsafe {
        sys::napi_strict_equals(left.value().env, left.raw(), right.raw(), &mut result)
    })?;
    Ok(result)
}

pub(super) fn invalid_arg(message: &str) -> Error {
    Error::new(napi::Status::InvalidArg, message)
}
