//! Tag and keyword classification helpers.
//!
//! Provides functions for determining:
//! - Whether a tag name refers to a component vs. native HTML element
//! - Whether a directive is built-in to Vue
//! - Whether a string is a JavaScript keyword

use crate::builtins::is_builtin_component;
use vize_carton::is_native_tag;

/// Check if a tag is a component or custom element.
#[inline]
pub fn is_component_tag(tag: &str) -> bool {
    if is_builtin_component(tag) {
        return false;
    }

    tag.contains('-')
        || tag.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        || !is_native_tag(tag)
}

/// Check if a directive is built-in
#[inline]
pub fn is_builtin_directive(name: &str) -> bool {
    matches!(
        name,
        "if" | "else"
            | "else-if"
            | "for"
            | "show"
            | "bind"
            | "on"
            | "model"
            | "slot"
            | "text"
            | "html"
            | "cloak"
            | "once"
            | "pre"
            | "memo"
    )
}

/// Check if a string is a JS keyword
#[inline]
pub fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "true"
            | "false"
            | "null"
            | "undefined"
            | "this"
            | "arguments"
            | "if"
            | "else"
            | "for"
            | "while"
            | "do"
            | "switch"
            | "case"
            | "break"
            | "continue"
            | "return"
            | "throw"
            | "try"
            | "catch"
            | "finally"
            | "new"
            | "delete"
            | "typeof"
            | "void"
            | "in"
            | "of"
            | "instanceof"
            | "function"
            | "class"
            | "const"
            | "let"
            | "var"
            | "async"
            | "await"
            | "yield"
            | "import"
            | "export"
            | "default"
            | "from"
            | "as"
    )
}

#[cfg(test)]
mod tests {
    use super::{is_builtin_directive, is_component_tag};

    #[test]
    fn test_is_component_tag() {
        assert!(is_component_tag("MyComponent"));
        assert!(is_component_tag("my-component"));
        assert!(is_component_tag("basic"));
        assert!(!is_component_tag("component"));
        assert!(!is_component_tag("slot"));
        assert!(!is_component_tag("transition-group"));
        assert!(!is_component_tag("div"));
        assert!(!is_component_tag("span"));
    }

    #[test]
    fn test_is_builtin_directive() {
        assert!(is_builtin_directive("if"));
        assert!(is_builtin_directive("for"));
        assert!(is_builtin_directive("model"));
        assert!(!is_builtin_directive("custom"));
    }
}
