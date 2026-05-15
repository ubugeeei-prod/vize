//! Template string construction, escaping, and template-ref extraction.

use super::*;

/// Generate element template string (recursively includes static children)
pub(crate) fn generate_element_template(el: &ElementNode<'_>) -> String {
    let mut template = cstr!("<{}", el.tag);

    // Collect dynamic binding names to skip their static counterparts
    let dynamic_attrs: vize_carton::FxHashSet<&str> = el
        .props
        .iter()
        .filter_map(|p| {
            if let PropNode::Directive(dir) = p {
                if dir.name.as_str() == "bind" {
                    if let Some(ref arg) = dir.arg {
                        if let ExpressionNode::Simple(key) = arg {
                            return Some(key.content.as_str());
                        }
                    }
                }
            }
            None
        })
        .collect();

    // Add static attributes (skip those overridden by dynamic bindings)
    for prop in el.props.iter() {
        if let PropNode::Attribute(attr) = prop {
            if is_runtime_only_attr(attr.name.as_str()) {
                continue;
            }
            if dynamic_attrs.contains(attr.name.as_str()) {
                continue;
            }
            if let Some(ref value) = attr.value {
                append!(template, " {}=\"{}\"", attr.name, value.content);
            } else {
                append!(template, " {}", attr.name);
            }
        }
    }

    if is_void_element(&el.tag) {
        template.push('>');
    } else if el.is_self_closing {
        append!(template, "></{}>", el.tag);
    } else {
        template.push('>');

        // Check if there are any interpolations - if so, use a space placeholder
        let has_interpolation = el
            .children
            .iter()
            .any(|c| matches!(c, TemplateChildNode::Interpolation(_)));

        if has_interpolation {
            // Use single space as placeholder for interpolation text content
            template.push(' ');
        } else {
            // Recursively add static children (text and static elements)
            for child in el.children.iter() {
                match child {
                    TemplateChildNode::Text(text) => {
                        template.push_str(&escape_html_text(&text.content));
                    }
                    TemplateChildNode::Element(child_el) => {
                        if is_template_backed_element(child_el) {
                            template.push_str(&generate_element_template(child_el));
                        }
                    }
                    _ => {
                        // Other dynamic content is handled elsewhere
                    }
                }
            }
        }

        append!(template, "</{}>", el.tag);
    }

    template
}

/// Escape HTML special characters in text content (vuejs/core #14310)
pub(crate) fn escape_html_text(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            _ => result.push(c),
        }
    }
    result
}

/// Check if an element is static (no dynamic directives)
pub(crate) fn is_static_element(el: &ElementNode<'_>) -> bool {
    if !matches!(el.tag_type, ElementType::Element) {
        return false;
    }

    // Template refs require runtime child lookup even when the rest of the
    // subtree is static, so they must not be folded into a purely static path.
    for prop in el.props.iter() {
        match prop {
            PropNode::Directive(_) => return false,
            PropNode::Attribute(attr) if is_runtime_only_attr(attr.name.as_str()) => return false,
            _ => {}
        }
    }

    // Check if any child is dynamic
    for child in el.children.iter() {
        match child {
            TemplateChildNode::Interpolation(_) => return false,
            TemplateChildNode::Element(child_el) => {
                if !is_static_element(child_el) {
                    return false;
                }
            }
            TemplateChildNode::If(_) | TemplateChildNode::For(_) => return false,
            _ => {}
        }
    }

    true
}

pub(super) fn is_template_backed_element(el: &ElementNode<'_>) -> bool {
    matches!(el.tag_type, ElementType::Element)
}

pub(super) fn transform_template_ref<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    element_id: usize,
    block: &mut BlockIRNode<'a>,
) {
    let Some(value) = extract_template_ref_value(ctx, el) else {
        return;
    };

    block
        .operation
        .push(OperationNode::SetTemplateRef(SetTemplateRefIRNode {
            element: element_id,
            value,
            ref_for: has_static_ref_for(el),
        }));
}

fn extract_template_ref_value<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
) -> Option<Box<'a, SimpleExpressionNode<'a>>> {
    for prop in el.props.iter() {
        match prop {
            PropNode::Attribute(attr) if attr.name.as_str() == "ref" => {
                let value = attr.value.as_ref()?;
                let node =
                    SimpleExpressionNode::new(value.content.clone(), true, value.loc.clone());
                return Some(Box::new_in(node, ctx.allocator));
            }
            PropNode::Directive(dir) if dir.name.as_str() == "bind" => {
                let Some(ExpressionNode::Simple(arg)) = dir.arg.as_ref() else {
                    continue;
                };
                if arg.content.as_str() != "ref" {
                    continue;
                }

                let Some(ExpressionNode::Simple(exp)) = dir.exp.as_ref() else {
                    continue;
                };
                let node =
                    SimpleExpressionNode::new(exp.content.clone(), exp.is_static, exp.loc.clone());
                return Some(Box::new_in(node, ctx.allocator));
            }
            _ => {}
        }
    }

    None
}

fn has_static_ref_for(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|prop| {
        matches!(
            prop,
            PropNode::Attribute(attr) if attr.name.as_str() == "ref_for"
        )
    })
}

pub(super) fn is_runtime_only_attr(name: &str) -> bool {
    matches!(name, "ref" | "ref_for" | "ref_key")
}

/// Check if an element is a void (self-closing) HTML element
fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}
