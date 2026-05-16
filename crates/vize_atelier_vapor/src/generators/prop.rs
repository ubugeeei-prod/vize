//! Prop code generation for Vapor mode.

use super::block::GenerateContext;
use crate::ir::{SetDynamicPropsIRNode, SetPropIRNode};
use vize_carton::{cstr, String, ToCompactString};

/// Generate SetProp code
pub fn generate_set_prop(ctx: &mut GenerateContext, set_prop: &SetPropIRNode<'_>) {
    let element = cstr!("_n{}", set_prop.element);
    let key = &set_prop.prop.key.content;

    let value: String = if let Some(first) = set_prop.prop.values.first() {
        if first.is_static {
            cstr!("\"{}\"", first.content)
        } else {
            first.content.to_compact_string()
        }
    } else {
        String::from("undefined")
    };

    // Determine how to set the prop
    if is_dom_prop(key) {
        // DOM property
        ctx.push_line_fmt(format_args!("{element}.{key} = {value}"));
    } else if key.starts_with("on") {
        // Event handler as prop (component)
        ctx.push_line_fmt(format_args!("_setEventProp({element}, \"{key}\", {value})"));
    } else {
        // Attribute
        ctx.push_line_fmt(format_args!("_setAttribute({element}, \"{key}\", {value})"));
    }
}

/// Generate SetDynamicProps code
pub fn generate_set_dynamic_props(
    ctx: &mut GenerateContext,
    set_props: &SetDynamicPropsIRNode<'_>,
) {
    let element = cstr!("_n{}", set_props.element);

    for prop in set_props.props.iter() {
        let expr: String = if prop.is_static {
            cstr!("\"{}\"", prop.content)
        } else {
            prop.content.to_compact_string()
        };
        ctx.push_line_fmt(format_args!("_setDynamicProps({element}, {expr})"));
    }
}

/// Check if key is a DOM property (vs attribute)
fn is_dom_prop(key: &str) -> bool {
    matches!(
        key,
        "innerHTML"
            | "textContent"
            | "value"
            | "checked"
            | "selected"
            | "disabled"
            | "readOnly"
            | "multiple"
            | "indeterminate"
    )
}

/// Generate class binding
pub fn generate_class_binding(element_var: &str, value: &str, is_static: bool) -> String {
    if is_static {
        cstr!("{element_var}.className = \"{value}\"")
    } else {
        cstr!("_setClass({element_var}, {value})")
    }
}

/// Generate style binding
pub fn generate_style_binding(element_var: &str, value: &str, is_static: bool) -> String {
    if is_static {
        cstr!("{element_var}.style.cssText = \"{value}\"")
    } else {
        cstr!("_setStyle({element_var}, {value})")
    }
}

/// Generate attribute binding
pub fn generate_attribute(element_var: &str, name: &str, value: &str) -> String {
    cstr!("{element_var}.setAttribute(\"{name}\", {value})")
}

/// Generate prop binding for component
pub fn generate_component_prop(component_var: &str, key: &str, value: &str) -> String {
    cstr!("{component_var}.$props.{key} = {value}")
}

/// Normalize prop key for components
pub fn normalize_prop_key(key: &str) -> String {
    // Convert kebab-case to camelCase
    let mut result = String::default();
    let mut capitalize_next = false;

    for c in key.chars() {
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

#[cfg(test)]
mod tests {
    use super::{generate_class_binding, is_dom_prop, normalize_prop_key};

    #[test]
    fn test_is_dom_prop() {
        assert!(is_dom_prop("value"));
        assert!(is_dom_prop("innerHTML"));
        assert!(!is_dom_prop("class"));
        assert!(!is_dom_prop("id"));
    }

    #[test]
    fn test_normalize_prop_key() {
        assert_eq!(normalize_prop_key("foo-bar"), "fooBar");
        assert_eq!(normalize_prop_key("foo-bar-baz"), "fooBarBaz");
        assert_eq!(normalize_prop_key("foo"), "foo");
    }

    #[test]
    fn test_generate_class_binding_static() {
        let result = generate_class_binding("_n1", "active", true);
        assert_eq!(result, "_n1.className = \"active\"");
    }
}
