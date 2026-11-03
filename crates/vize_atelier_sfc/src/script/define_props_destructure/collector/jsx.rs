//! JSX in `lang="tsx"` / `"jsx"` setup scripts. Like Vue, expressions in
//! attributes and children are rewritten; tag names are not.

use oxc_ast::ast::{JSXAttributeItem, JSXAttributeValue, JSXChild, JSXElement, JSXFragment};
use vize_carton::{FxHashMap, String};

use super::collect_from_expression;

pub(super) fn collect_from_element(
    element: &JSXElement<'_>,
    source: &str,
    local_to_key: &FxHashMap<&str, &str>,
    local_bindings: &FxHashMap<String, bool>,
    rewrites: &mut Vec<(usize, usize, String)>,
) {
    for attribute in &element.opening_element.attributes {
        match attribute {
            JSXAttributeItem::Attribute(attribute) => match &attribute.value {
                Some(JSXAttributeValue::ExpressionContainer(container)) => {
                    if let Some(expression) = container.expression.as_expression() {
                        collect_from_expression(
                            expression,
                            source,
                            local_to_key,
                            local_bindings,
                            rewrites,
                        );
                    }
                }
                Some(JSXAttributeValue::Element(element)) => {
                    collect_from_element(element, source, local_to_key, local_bindings, rewrites);
                }
                Some(JSXAttributeValue::Fragment(fragment)) => {
                    collect_from_fragment(fragment, source, local_to_key, local_bindings, rewrites);
                }
                Some(JSXAttributeValue::StringLiteral(_)) | None => {}
            },
            JSXAttributeItem::SpreadAttribute(spread) => collect_from_expression(
                &spread.argument,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            ),
        }
    }
    collect_from_children(&element.children, source, local_to_key, local_bindings, rewrites);
}

pub(super) fn collect_from_fragment(
    fragment: &JSXFragment<'_>,
    source: &str,
    local_to_key: &FxHashMap<&str, &str>,
    local_bindings: &FxHashMap<String, bool>,
    rewrites: &mut Vec<(usize, usize, String)>,
) {
    collect_from_children(&fragment.children, source, local_to_key, local_bindings, rewrites);
}

fn collect_from_children(
    children: &[JSXChild<'_>],
    source: &str,
    local_to_key: &FxHashMap<&str, &str>,
    local_bindings: &FxHashMap<String, bool>,
    rewrites: &mut Vec<(usize, usize, String)>,
) {
    for child in children {
        match child {
            JSXChild::ExpressionContainer(container) => {
                if let Some(expression) = container.expression.as_expression() {
                    collect_from_expression(
                        expression,
                        source,
                        local_to_key,
                        local_bindings,
                        rewrites,
                    );
                }
            }
            JSXChild::Spread(spread) => collect_from_expression(
                &spread.expression,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            ),
            JSXChild::Element(element) => {
                collect_from_element(element, source, local_to_key, local_bindings, rewrites);
            }
            JSXChild::Fragment(fragment) => {
                collect_from_fragment(fragment, source, local_to_key, local_bindings, rewrites);
            }
            JSXChild::Text(_) => {}
        }
    }
}
