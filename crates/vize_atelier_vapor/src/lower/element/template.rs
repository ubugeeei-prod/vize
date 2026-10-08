//! Template string construction, escaping, and template-ref extraction.

use super::{
    BlockIRNode, Box, ElementNode, ElementType, ExpressionNode, OperationNode, PropNode,
    SetTemplateRefIRNode, SimpleExpressionNode, String, TemplateChildNode, TransformContext,
};
use vize_atelier_core::codegen::document::EmitDocument;
use vize_carton::Span;
use vize_carton::ensure_sufficient_stack;

mod attributes;
pub(in crate::lower) use attributes::RootAttributes;
mod writer;
use writer::{LeadingNewlineWriter, TemplateWriter};

/// Generate an element template from its already-classified root key scope.
#[inline(always)]
pub(in crate::lower) fn generate_element_template(
    root: RootAttributes<'_, '_>,
    scope_id: Option<&str>,
    source: &str,
) -> String {
    let mut template = String::default();
    let el = root.owner();
    let non_reactive = root.non_reactive();
    write_element_template(
        &mut template,
        el,
        scope_id,
        source,
        non_reactive,
        Some(root),
    );
    template
}

/// [`generate_element_template`] linking the tag names, static attributes and
/// text it copies to their authored ranges (Davinci P3-9); identical bytes.
pub(in crate::lower) fn generate_element_template_spanned(
    root: RootAttributes<'_, '_>,
    scope_id: Option<&str>,
    source: &str,
) -> EmitDocument {
    let mut template = EmitDocument::default();
    let el = root.owner();
    let non_reactive = root.non_reactive();
    write_element_template(
        &mut template,
        el,
        scope_id,
        source,
        non_reactive,
        Some(root),
    );
    template
}

fn write_element_template(
    template: &mut impl TemplateWriter,
    el: &ElementNode<'_>,
    scope_id: Option<&str>,
    source: &str,
    non_reactive: bool,
    root: Option<RootAttributes<'_, '_>>,
) {
    // Root facts came from its actual lowering walk. Recursion still owns
    // each child's once/memo derivation and never broadens the lower's context.
    let mut non_reactive = non_reactive;
    template.push_str("<");
    let tag_start = el.loc.span.start + 1;
    template.push_linked(
        el.tag,
        Span::new(tag_start, tag_start + el.tag.len() as u32),
    );
    if let Some(scope_id) = scope_id {
        template.push_str(" ");
        template.push_str(scope_id);
    }

    if !el.props.is_empty() {
        attributes::write_attributes(template, el, source, &mut non_reactive, root);
    }

    if is_void_element(el.tag) {
        template.push_str(">");
    } else if el.is_self_closing {
        template.push_str("></");
        template.push_str(el.tag);
        template.push_str(">");
    } else {
        template.push_str(">");

        // Recursively add template-backed children. `<template>` is a
        // transparent wrapper in Vapor just as it is in the main element
        // dispatcher, so its children contribute directly to the enclosing
        // element's static template instead of producing a component lookup.
        let placeholders = super::insertion::block_placeholders(&el.children, non_reactive);
        let mut placeholders = placeholders.into_iter();
        if el.ns == vize_atelier_core::Namespace::Html && matches!(el.tag, "pre" | "textarea") {
            let mut children = LeadingNewlineWriter {
                sink: template,
                first: true,
            };
            append_child_templates(
                &mut children,
                &el.children,
                scope_id,
                source,
                non_reactive,
                &mut placeholders,
            );
        } else {
            append_child_templates(
                template,
                &el.children,
                scope_id,
                source,
                non_reactive,
                &mut placeholders,
            );
        }

        template.push_str("</");
        template.push_str(el.tag);
        template.push_str(">");
    }
}

fn append_child_templates(
    template: &mut impl TemplateWriter,
    children: &[TemplateChildNode<'_>],
    scope_id: Option<&str>,
    source: &str,
    non_reactive: bool,
    placeholders: &mut std::vec::IntoIter<bool>,
) {
    for child in children {
        match child {
            TemplateChildNode::Text(text) => {
                template.push_linked(&escape_html_text(text.content), text.loc.span);
            }
            TemplateChildNode::Interpolation(_) => {
                template.push_str(" ");
            }
            TemplateChildNode::Element(child_el) if child_el.tag_type == ElementType::Template => {
                ensure_sufficient_stack(|| {
                    append_child_templates(
                        template,
                        &child_el.children,
                        scope_id,
                        source,
                        super::super::key::is_non_reactive(child_el, non_reactive),
                        placeholders,
                    )
                });
            }
            TemplateChildNode::Element(child_el)
                if is_template_backed_element(child_el, non_reactive) =>
            {
                ensure_sufficient_stack(|| {
                    write_element_template(template, child_el, scope_id, source, non_reactive, None)
                });
            }
            // Only a block followed by template-rendered siblings keeps its
            // insertion placeholder (see `insertion::block_placeholders`).
            TemplateChildNode::Element(_)
            | TemplateChildNode::If(_)
            | TemplateChildNode::For(_)
                if placeholders.next().unwrap_or(true) =>
            {
                template.push_str("<!---->");
            }
            _ => {}
        }
    }
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
            PropNode::Attribute(attr) if is_runtime_only_attr(attr.name) => return false,
            _ => {}
        }
    }

    for child in el.children.iter() {
        match child {
            TemplateChildNode::Interpolation(_) => return false,
            TemplateChildNode::Element(child_el) => {
                if !ensure_sufficient_stack(|| is_static_element(child_el)) {
                    return false;
                }
            }
            TemplateChildNode::If(_) | TemplateChildNode::For(_) => return false,
            _ => {}
        }
    }

    true
}

pub(super) fn is_template_backed_element(el: &ElementNode<'_>, non_reactive: bool) -> bool {
    matches!(el.tag_type, ElementType::Element)
        && super::super::key::value(el, non_reactive).is_none()
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
            ref_for: ctx.for_depth > 0 || has_static_ref_for(el),
        }));
}

fn extract_template_ref_value<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
) -> Option<Box<'a, SimpleExpressionNode<'a>>> {
    for prop in el.props.iter() {
        match prop {
            PropNode::Attribute(attr) if attr.name == "ref" => {
                let value = attr.value.as_ref()?;
                let node = SimpleExpressionNode::new(value.content, true, value.loc.clone());
                return Some(Box::new_in(node, &ctx.allocator));
            }
            PropNode::Directive(dir) if dir.name == "bind" => {
                let Some(ExpressionNode::Simple(arg)) = dir.arg.as_ref() else {
                    continue;
                };
                if !arg.is_static || arg.content != "ref" {
                    continue;
                }

                let Some(ExpressionNode::Simple(exp)) = dir.exp.as_ref() else {
                    continue;
                };
                let node = SimpleExpressionNode::from_node(exp);
                return Some(Box::new_in(node, &ctx.allocator));
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
            PropNode::Attribute(attr) if attr.name == "ref_for"
        )
    })
}

pub(in crate::lower) fn is_runtime_only_attr(name: &str) -> bool {
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

#[cfg(test)]
mod facts_tests;
