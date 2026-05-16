//! General utility functions shared across the compiler.

use crate::String;
use once_cell::sync::Lazy;
use phf::phf_set;
use rustc_hash::FxHashMap;
use std::sync::RwLock;

/// Reserved props that should not be passed to components
pub static RESERVED_PROPS: phf::Set<&'static str> = phf_set! {
    "", "key", "ref", "ref_for", "ref_key",
    "onVnodeBeforeMount", "onVnodeMounted",
    "onVnodeBeforeUpdate", "onVnodeUpdated",
    "onVnodeBeforeUnmount", "onVnodeUnmounted"
};

/// Built-in tags
pub static BUILTIN_TAGS: phf::Set<&'static str> = phf_set! {
    "slot", "component"
};

/// Built-in directives
pub static BUILTIN_DIRECTIVES: phf::Set<&'static str> = phf_set! {
    "bind", "cloak", "else-if", "else", "for", "html", "if",
    "model", "on", "once", "pre", "show", "slot", "text", "memo"
};

/// Check if a property name is reserved
#[inline]
pub fn is_reserved_prop(key: &str) -> bool {
    RESERVED_PROPS.contains(key)
}

/// Check if a tag is a built-in tag
#[inline]
pub fn is_builtin_tag(tag: &str) -> bool {
    BUILTIN_TAGS.contains(tag)
}

/// Check if a directive is a built-in directive
#[inline]
pub fn is_builtin_directive(name: &str) -> bool {
    BUILTIN_DIRECTIVES.contains(name)
}

/// Check if a key is an event handler (starts with "on" + uppercase letter)
#[inline]
pub fn is_on(key: &str) -> bool {
    let bytes = key.as_bytes();
    bytes.len() > 2 && bytes[0] == b'o' && bytes[1] == b'n' && (bytes[2] > 122 || bytes[2] < 97)
    // uppercase letter
}

/// Check if a key is a native event handler (starts with "on" + lowercase letter)
#[inline]
pub fn is_native_on(key: &str) -> bool {
    let bytes = key.as_bytes();
    bytes.len() > 2 && bytes[0] == b'o' && bytes[1] == b'n' && bytes[2] > 96 && bytes[2] < 123
    // lowercase letter
}

/// Check if a key is a model listener (starts with "onUpdate:")
#[inline]
pub fn is_model_listener(key: &str) -> bool {
    key.starts_with("onUpdate:")
}

// String transformation caches
static CAMELIZE_CACHE: Lazy<RwLock<FxHashMap<String, String>>> =
    Lazy::new(|| RwLock::new(FxHashMap::default()));
static HYPHENATE_CACHE: Lazy<RwLock<FxHashMap<String, String>>> =
    Lazy::new(|| RwLock::new(FxHashMap::default()));
static CAPITALIZE_CACHE: Lazy<RwLock<FxHashMap<String, String>>> =
    Lazy::new(|| RwLock::new(FxHashMap::default()));

/// Convert kebab-case to camelCase
/// Example: "foo-bar" -> "fooBar"
pub fn camelize(s: &str) -> String {
    // Check cache first
    {
        let cache = CAMELIZE_CACHE.read().unwrap();
        if let Some(cached) = cache.get(s) {
            return cached.clone();
        }
    }

    let result = camelize_uncached(s);

    // Store in cache
    {
        let mut cache = CAMELIZE_CACHE.write().unwrap();
        cache.insert(String::from(s), result.clone());
    }

    result
}

fn camelize_uncached(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Convert camelCase to kebab-case
/// Example: "fooBar" -> "foo-bar"
pub fn hyphenate(s: &str) -> String {
    // Check cache first
    {
        let cache = HYPHENATE_CACHE.read().unwrap();
        if let Some(cached) = cache.get(s) {
            return cached.clone();
        }
    }

    let result = hyphenate_uncached(s);

    // Store in cache
    {
        let mut cache = HYPHENATE_CACHE.write().unwrap();
        cache.insert(String::from(s), result.clone());
    }

    result
}

fn hyphenate_uncached(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);

    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            result.push('-');
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}

/// Capitalize the first letter
/// Example: "foo" -> "Foo"
pub fn capitalize(s: &str) -> String {
    if s.is_empty() {
        return String::new("");
    }

    // Check cache first
    {
        let cache = CAPITALIZE_CACHE.read().unwrap();
        if let Some(cached) = cache.get(s) {
            return cached.clone();
        }
    }

    let result = capitalize_uncached(s);

    // Store in cache
    {
        let mut cache = CAPITALIZE_CACHE.write().unwrap();
        cache.insert(String::from(s), result.clone());
    }

    result
}

fn capitalize_uncached(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(""),
        Some(first) => {
            let mut result = String::with_capacity(s.len());
            result.push(first.to_ascii_uppercase());
            result.extend(chars);
            result
        }
    }
}

/// Convert a string to an event handler key
/// Example: "click" -> "onClick"
pub fn to_handler_key(s: &str) -> String {
    if s.is_empty() {
        return String::new("");
    }

    let mut result = String::with_capacity(s.len() + 2);
    result.push_str("on");
    result.push_str(&capitalize(s));
    result
}

/// Get the modifiers prop name for v-model
pub fn get_modifier_prop_name(name: &str) -> String {
    let base = if name == "modelValue" || name == "model-value" {
        "model"
    } else {
        name
    };

    let suffix = if name == "model" { "$" } else { "" };

    crate::cstr!("{base}Modifiers{suffix}")
}

/// Check if a string is a valid JavaScript identifier
pub fn is_simple_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();

    // First character must be letter, underscore, or $
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }

    // Rest must be alphanumeric, underscore, or $
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

/// Generate a props access expression
pub fn gen_props_access_exp(name: &str) -> String {
    if is_simple_identifier(name) {
        crate::cstr!("__props.{name}")
    } else {
        let key = serde_json::to_string(name).unwrap();
        crate::cstr!("__props[{key}]")
    }
}

/// Check if a tag can have its value set directly
pub fn can_set_value_directly(tag_name: &str) -> bool {
    tag_name != "PROGRESS" && !tag_name.contains('-')
}

#[cfg(test)]
mod tests {
    use super::{camelize, capitalize, hyphenate, is_on, is_simple_identifier, to_handler_key};

    #[test]
    fn test_is_on() {
        assert!(is_on("onClick"));
        assert!(is_on("onUpdate"));
        assert!(!is_on("onclick"));
        assert!(!is_on("on"));
    }

    #[test]
    fn test_camelize() {
        assert_eq!(camelize("foo-bar"), "fooBar");
        assert_eq!(camelize("foo-bar-baz"), "fooBarBaz");
        assert_eq!(camelize("foo"), "foo");
    }

    #[test]
    fn test_hyphenate() {
        assert_eq!(hyphenate("fooBar"), "foo-bar");
        assert_eq!(hyphenate("fooBarBaz"), "foo-bar-baz");
        assert_eq!(hyphenate("foo"), "foo");
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("foo"), "Foo");
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("Foo"), "Foo");
    }

    #[test]
    fn test_to_handler_key() {
        assert_eq!(to_handler_key("click"), "onClick");
        assert_eq!(to_handler_key("update"), "onUpdate");
        assert_eq!(to_handler_key(""), "");
    }

    #[test]
    fn test_is_simple_identifier() {
        assert!(is_simple_identifier("foo"));
        assert!(is_simple_identifier("_foo"));
        assert!(is_simple_identifier("$foo"));
        assert!(is_simple_identifier("foo123"));
        assert!(!is_simple_identifier("123foo"));
        assert!(!is_simple_identifier("foo-bar"));
        assert!(!is_simple_identifier(""));
    }
}
