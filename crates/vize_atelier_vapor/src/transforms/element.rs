//! Element transform for Vapor mode.
//!
//! Transforms element nodes into template strings and operations.

use vize_carton::append;
use vize_carton::cstr;
use vize_carton::String;

use vize_atelier_core::{ElementNode, ElementType, PropNode, TemplateChildNode};

/// Generate static template string for an element
pub fn generate_element_template(el: &ElementNode<'_>) -> String {
    let mut template = cstr!("<{}", el.tag);

    // Add static attributes
    for prop in el.props.iter() {
        if let PropNode::Attribute(attr) = prop {
            if let Some(ref value) = attr.value {
                append!(
                    template,
                    " {}=\"{}\"",
                    attr.name,
                    escape_attr(&value.content)
                );
            } else {
                append!(template, " {}", attr.name);
            }
        }
    }

    if el.is_self_closing {
        template.push_str(" />");
    } else {
        template.push('>');

        // Add static text content
        for child in el.children.iter() {
            match child {
                TemplateChildNode::Text(text) => {
                    template.push_str(&escape_html(&text.content));
                }
                TemplateChildNode::Element(child_el) => {
                    // Recursively generate child element template
                    template.push_str(&generate_element_template(child_el));
                }
                _ => {}
            }
        }

        append!(template, "</{}>", el.tag);
    }

    template
}

/// Check if element is static (no dynamic bindings or children)
pub fn is_static_element(el: &ElementNode<'_>) -> bool {
    // Has no directives
    let has_directives = el.props.iter().any(|p| matches!(p, PropNode::Directive(_)));
    if has_directives {
        return false;
    }

    // All children are static
    el.children.iter().all(|child| match child {
        TemplateChildNode::Text(_) => true,
        TemplateChildNode::Element(child_el) => is_static_element(child_el),
        _ => false,
    })
}

/// Check if element has any event listeners
pub fn has_event_listeners(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|prop| {
        if let PropNode::Directive(dir) = prop {
            dir.name == "on"
        } else {
            false
        }
    })
}

/// Check if element has any dynamic bindings
pub fn has_dynamic_bindings(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|prop| {
        if let PropNode::Directive(dir) = prop {
            dir.name == "bind"
        } else {
            false
        }
    })
}

/// Check if element needs slot handling
pub fn is_slot_outlet(el: &ElementNode<'_>) -> bool {
    el.tag == "slot"
}

/// Check if element is a component
pub fn is_component(el: &ElementNode<'_>) -> bool {
    el.tag_type == ElementType::Component
}

/// Check if element is a template wrapper
pub fn is_template_wrapper(el: &ElementNode<'_>) -> bool {
    el.tag_type == ElementType::Template
}

/// Get element tag name
pub fn get_tag_name(el: &ElementNode<'_>) -> String {
    el.tag.clone()
}

/// Escape HTML special characters
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .into()
}

/// Escape attribute value
fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .into()
}

#[cfg(test)]
mod tests {
    use super::{escape_attr, escape_html};

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<div>"), "&lt;div&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
    }

    #[test]
    fn test_escape_attr() {
        assert_eq!(escape_attr("hello \"world\""), "hello &quot;world&quot;");
    }
}
