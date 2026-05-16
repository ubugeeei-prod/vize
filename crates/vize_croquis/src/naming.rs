//! Naming convention utilities for Vue components and properties.
//!
//! This module provides centralized utilities for:
//! - Case conversion (camelCase, PascalCase, kebab-case)
//! - Case validation
//! - Re-exports from vize_carton for convenience

// Re-export core utilities from vize_carton
pub use vize_carton::{camelize, capitalize, hyphenate, is_simple_identifier, String};

use once_cell::sync::Lazy;
use rustc_hash::FxHashMap;
use std::sync::RwLock;
use vize_carton::CompactString;

// =============================================================================
// Cached Conversions
// =============================================================================

/// Cache for to_pascal_case conversions
static PASCAL_CASE_CACHE: Lazy<RwLock<FxHashMap<CompactString, CompactString>>> =
    Lazy::new(|| RwLock::new(FxHashMap::default()));

/// Convert kebab-case or camelCase to PascalCase
///
/// # Examples
/// ```
/// use vize_croquis::naming::to_pascal_case;
///
/// assert_eq!(to_pascal_case("my-component"), "MyComponent");
/// assert_eq!(to_pascal_case("myComponent"), "MyComponent");
/// assert_eq!(to_pascal_case("MyComponent"), "MyComponent");
/// ```
pub fn to_pascal_case(s: &str) -> CompactString {
    if s.is_empty() {
        return CompactString::default();
    }

    // Check cache first
    {
        let cache = PASCAL_CASE_CACHE.read().unwrap();
        if let Some(cached) = cache.get(s) {
            return cached.clone();
        }
    }

    let result = to_pascal_case_uncached(s);

    // Store in cache
    {
        let mut cache = PASCAL_CASE_CACHE.write().unwrap();
        cache.insert(CompactString::new(s), result.clone());
    }

    result
}

fn to_pascal_case_uncached(s: &str) -> CompactString {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '-' || c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    CompactString::new(&result)
}

// =============================================================================
// Validation Functions
// =============================================================================

/// Check if a string is in camelCase format
///
/// Returns true if the string starts with a lowercase letter and contains
/// at least one uppercase letter (excluding the first character).
///
/// # Examples
/// ```
/// use vize_croquis::naming::is_camel_case;
///
/// assert!(is_camel_case("myComponent"));
/// assert!(is_camel_case("fooBar"));
/// assert!(!is_camel_case("MyComponent")); // PascalCase
/// assert!(!is_camel_case("my-component")); // kebab-case
/// assert!(!is_camel_case("foo")); // no uppercase
/// ```
#[inline]
pub fn is_camel_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    // Must start with lowercase
    if !first.is_ascii_lowercase() {
        return false;
    }

    // Must contain at least one uppercase letter (true camelCase)
    // and no dashes or underscores
    let mut has_uppercase = false;
    for c in chars {
        if c == '-' || c == '_' {
            return false;
        }
        if c.is_ascii_uppercase() {
            has_uppercase = true;
        }
    }

    has_uppercase
}

/// Check if a string is in strict camelCase format (allows single lowercase words)
///
/// Similar to `is_camel_case` but also returns true for single lowercase words.
///
/// # Examples
/// ```
/// use vize_croquis::naming::is_camel_case_loose;
///
/// assert!(is_camel_case_loose("myComponent"));
/// assert!(is_camel_case_loose("foo")); // single word is OK
/// assert!(!is_camel_case_loose("MyComponent"));
/// assert!(!is_camel_case_loose("my-component"));
/// ```
#[inline]
pub fn is_camel_case_loose(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let first = s.chars().next().unwrap();

    // Must start with lowercase
    if !first.is_ascii_lowercase() {
        return false;
    }

    // No dashes or underscores
    !s.contains('-') && !s.contains('_')
}

/// Check if a string is in PascalCase format
///
/// Returns true if the string starts with an uppercase letter and contains
/// no dashes or underscores.
///
/// # Examples
/// ```
/// use vize_croquis::naming::is_pascal_case;
///
/// assert!(is_pascal_case("MyComponent"));
/// assert!(is_pascal_case("FooBar"));
/// assert!(!is_pascal_case("myComponent")); // camelCase
/// assert!(!is_pascal_case("my-component")); // kebab-case
/// ```
#[inline]
pub fn is_pascal_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let first = s.chars().next().unwrap();

    // Must start with uppercase
    if !first.is_ascii_uppercase() {
        return false;
    }

    // No dashes or underscores
    !s.contains('-') && !s.contains('_')
}

/// Check if a string is in kebab-case format
///
/// Returns true if the string contains dashes and all letters are lowercase.
///
/// # Examples
/// ```
/// use vize_croquis::naming::is_kebab_case;
///
/// assert!(is_kebab_case("my-component"));
/// assert!(is_kebab_case("foo-bar-baz"));
/// assert!(!is_kebab_case("myComponent")); // camelCase
/// assert!(!is_kebab_case("MyComponent")); // PascalCase
/// assert!(!is_kebab_case("foo")); // no dash
/// ```
#[inline]
pub fn is_kebab_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Must contain at least one dash
    if !s.contains('-') {
        return false;
    }

    // All letters must be lowercase
    s.chars().all(|c| !c.is_ascii_uppercase())
}

/// Check if a string is in strict kebab-case format (allows single lowercase words)
///
/// Similar to `is_kebab_case` but also returns true for single lowercase words.
///
/// # Examples
/// ```
/// use vize_croquis::naming::is_kebab_case_loose;
///
/// assert!(is_kebab_case_loose("my-component"));
/// assert!(is_kebab_case_loose("foo")); // single word is OK
/// assert!(!is_kebab_case_loose("myComponent"));
/// assert!(!is_kebab_case_loose("MyComponent"));
/// ```
#[inline]
pub fn is_kebab_case_loose(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // All letters must be lowercase
    s.chars().all(|c| !c.is_ascii_uppercase())
}

/// Check if two names match when normalized (kebab vs camel comparison)
///
/// # Examples
/// ```
/// use vize_croquis::naming::names_match;
///
/// assert!(names_match("my-prop", "myProp"));
/// assert!(names_match("myProp", "my-prop"));
/// assert!(names_match("foo", "foo"));
/// assert!(!names_match("myProp", "otherProp"));
/// ```
#[inline]
pub fn names_match(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }

    // Compare camelized versions
    camelize(a) == camelize(b)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::{
        is_camel_case, is_camel_case_loose, is_kebab_case, is_kebab_case_loose, is_pascal_case,
        names_match, to_pascal_case,
    };

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("my-component").as_str(), "MyComponent");
        assert_eq!(to_pascal_case("myComponent").as_str(), "MyComponent");
        assert_eq!(to_pascal_case("MyComponent").as_str(), "MyComponent");
        assert_eq!(to_pascal_case("foo-bar-baz").as_str(), "FooBarBaz");
        assert_eq!(to_pascal_case("foo_bar").as_str(), "FooBar");
        assert_eq!(to_pascal_case("").as_str(), "");
    }

    #[test]
    fn test_is_camel_case() {
        assert!(is_camel_case("myComponent"));
        assert!(is_camel_case("fooBar"));
        assert!(is_camel_case("fooBarBaz"));
        assert!(!is_camel_case("MyComponent"));
        assert!(!is_camel_case("my-component"));
        assert!(!is_camel_case("foo"));
        assert!(!is_camel_case(""));
    }

    #[test]
    fn test_is_camel_case_loose() {
        assert!(is_camel_case_loose("myComponent"));
        assert!(is_camel_case_loose("foo"));
        assert!(!is_camel_case_loose("MyComponent"));
        assert!(!is_camel_case_loose("my-component"));
    }

    #[test]
    fn test_is_pascal_case() {
        assert!(is_pascal_case("MyComponent"));
        assert!(is_pascal_case("FooBar"));
        assert!(is_pascal_case("Foo"));
        assert!(!is_pascal_case("myComponent"));
        assert!(!is_pascal_case("my-component"));
        assert!(!is_pascal_case(""));
    }

    #[test]
    fn test_is_kebab_case() {
        assert!(is_kebab_case("my-component"));
        assert!(is_kebab_case("foo-bar-baz"));
        assert!(!is_kebab_case("myComponent"));
        assert!(!is_kebab_case("MyComponent"));
        assert!(!is_kebab_case("foo"));
        assert!(!is_kebab_case(""));
    }

    #[test]
    fn test_is_kebab_case_loose() {
        assert!(is_kebab_case_loose("my-component"));
        assert!(is_kebab_case_loose("foo"));
        assert!(!is_kebab_case_loose("myComponent"));
        assert!(!is_kebab_case_loose("MyComponent"));
    }

    #[test]
    fn test_names_match() {
        assert!(names_match("my-prop", "myProp"));
        assert!(names_match("myProp", "my-prop"));
        assert!(names_match("foo", "foo"));
        assert!(names_match("foo-bar", "fooBar"));
        assert!(!names_match("myProp", "otherProp"));
    }
}
