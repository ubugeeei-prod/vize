//! Import usage check for SFC templates.
//!
//! This module checks if an identifier is used in the SFC's template.
//! - Used to determine the properties that should be included in the object returned from setup()
//!   when not using inline mode.
//! - Check whether the built-in properties such as $attrs, $slots, $emit are used in the template

use vize_atelier_core::{
    DirectiveNode, ElementNode, ExpressionNode, PropNode, RootNode, TemplateChildNode,
};
use vize_carton::{
    FxHashSet, String, ToCompactString, camelize, capitalize, is_builtin_directive, is_native_tag,
    is_simple_identifier,
};
use vize_croquis::builtins::is_builtin_component;

mod expressions;
use expressions::{
    extract_identifiers_from_compound, extract_identifiers_from_expression,
    extract_slot_pattern_identifiers,
};

/// Result of template analysis
#[derive(Debug, Clone, Default)]
pub struct TemplateUsedIdentifiers {
    /// All identifiers used in the template
    pub used_ids: FxHashSet<String>,
    /// Identifiers used in v-model expressions (simple identifiers only)
    pub v_model_ids: FxHashSet<String>,
}

#[derive(Debug, Clone, Default)]
struct TemplateIdentifierAnalysis {
    used_ids: FxHashSet<String>,
    read_ids: FxHashSet<String>,
    v_model_ids: FxHashSet<String>,
}

impl From<TemplateIdentifierAnalysis> for TemplateUsedIdentifiers {
    fn from(value: TemplateIdentifierAnalysis) -> Self {
        Self {
            used_ids: value.used_ids,
            v_model_ids: value.v_model_ids,
        }
    }
}

/// Check if an identifier is used in the SFC's template.
pub fn is_used_in_template(identifier: &str, root: &RootNode) -> bool {
    resolve_template_used_identifiers(root)
        .used_ids
        .contains(identifier)
}

/// Resolve all identifiers used in v-model expressions.
pub fn resolve_template_v_model_identifiers(root: &RootNode) -> FxHashSet<String> {
    resolve_template_analysis_result(root, false).v_model_ids
}

/// Resolve all identifiers used in the template.
pub fn resolve_template_used_identifiers(root: &RootNode) -> TemplateUsedIdentifiers {
    resolve_template_analysis_result(root, true).into()
}

/// Resolve identifiers that should count as TypeScript reads.
///
/// Static string refs are intentionally excluded: Vue observes `ref="name"` at
/// runtime, but vue-tsc does not count that attribute as a script read for
/// `noUnusedLocals`.
pub fn resolve_template_read_identifiers(root: &RootNode) -> FxHashSet<String> {
    resolve_template_analysis_result(root, true).read_ids
}

/// Analyze the template and extract identifiers.
///
/// When `collect_used_ids` is false, we skip the expensive identifier extraction
/// and only collect `v_model_ids`.
fn resolve_template_analysis_result(
    root: &RootNode,
    collect_used_ids: bool,
) -> TemplateIdentifierAnalysis {
    let mut result = TemplateIdentifierAnalysis::default();

    for child in root.children.iter() {
        walk_node(child, &mut result, collect_used_ids);
    }

    result
}

/// Walk a template child node and collect identifiers.
fn walk_node(
    node: &TemplateChildNode,
    result: &mut TemplateIdentifierAnalysis,
    collect_used_ids: bool,
) {
    match node {
        TemplateChildNode::Element(element) => {
            walk_element(element, result, collect_used_ids);
        }
        TemplateChildNode::Interpolation(interpolation) => {
            if collect_used_ids {
                extract_read_identifiers_from_expression(&interpolation.content, result);
            }
        }
        TemplateChildNode::If(if_node) => {
            for branch in if_node.branches.iter() {
                // Walk condition expression if present
                if collect_used_ids && let Some(ref condition) = branch.condition {
                    extract_read_identifiers_from_expression(condition, result);
                }
                // Walk children
                for child in branch.children.iter() {
                    walk_node(child, result, collect_used_ids);
                }
            }
        }
        TemplateChildNode::For(for_node) => {
            // Walk source expression
            if collect_used_ids {
                extract_read_identifiers_from_expression(&for_node.source, result);
            }
            // Walk children
            for child in for_node.children.iter() {
                walk_node(child, result, collect_used_ids);
            }
        }
        TemplateChildNode::TextCall(text_call) => {
            if collect_used_ids {
                match &text_call.content {
                    vize_atelier_core::TextCallContent::Interpolation(interp) => {
                        extract_read_identifiers_from_expression(&interp.content, result);
                    }
                    vize_atelier_core::TextCallContent::Compound(compound) => {
                        extract_read_identifiers_from_compound(compound, result);
                    }
                    _ => {}
                }
            }
        }
        TemplateChildNode::CompoundExpression(compound) if collect_used_ids => {
            extract_read_identifiers_from_compound(compound, result);
        }
        // Text, Comment, IfBranch, Hoisted don't need processing
        _ => {}
    }
}

/// Walk an element node and collect identifiers.
fn walk_element(
    element: &ElementNode,
    result: &mut TemplateIdentifierAnalysis,
    collect_used_ids: bool,
) {
    // Process tag name - check if it's a component
    let mut tag = element.tag;

    // Handle member expression tags like Foo.Bar
    if let Some((head, _)) = tag.split_once('.') {
        tag = head;
    }

    // If not a native tag or built-in component, add to identifiers
    if !is_native_tag(tag) && !is_builtin_component(tag) && collect_used_ids {
        // Add both camelCase and PascalCase versions
        let camelized = camelize(tag);
        let capitalized = capitalize(&camelized);
        insert_read_identifier(result, camelized.to_compact_string());
        insert_read_identifier(result, capitalized.to_compact_string());
    }

    // Process props
    for prop in element.props.iter() {
        match prop {
            PropNode::Directive(directive) => {
                process_directive(directive, result, collect_used_ids);
            }
            PropNode::Attribute(attr) => {
                // ref attribute value is an identifier
                if collect_used_ids
                    && attr.name == "ref"
                    && let Some(ref value) = attr.value
                    && !value.content.is_empty()
                {
                    result.used_ids.insert(value.content.to_compact_string());
                }
            }
        }
    }

    // Walk children
    for child in element.children.iter() {
        walk_node(child, result, collect_used_ids);
    }
}

/// Process a directive and collect identifiers.
fn process_directive(
    directive: &DirectiveNode,
    result: &mut TemplateIdentifierAnalysis,
    collect_used_ids: bool,
) {
    let name = directive.name;

    // Add custom directive to identifiers
    if collect_used_ids && !is_builtin_directive(name) {
        let camel = camelize(name);
        let cap = capitalize(&camel);
        let mut directive_name = String::with_capacity(1 + cap.len());
        directive_name.push('v');
        directive_name.push_str(&cap);
        insert_read_identifier(result, directive_name);
    }

    // Collect v-model target identifiers (simple identifiers only)
    if name == "model"
        && let Some(ref exp) = directive.exp
        && let ExpressionNode::Simple(simple_exp) = exp
    {
        let exp_string = simple_exp.content.trim();
        if is_simple_identifier(exp_string) && exp_string != "undefined" {
            result.v_model_ids.insert(exp_string.to_compact_string());
        }
    }

    // Process dynamic directive arguments
    if collect_used_ids
        && let Some(ref arg) = directive.arg
        && let ExpressionNode::Simple(simple_arg) = arg
        && !simple_arg.is_static
    {
        extract_read_identifiers_from_expression(arg, result);
    }

    // Process directive expression
    if collect_used_ids {
        if name == "for" {
            // For v-for, use the parsed source expression if available
            if let Some(ref for_result) = directive.for_parse_result {
                extract_read_identifiers_from_expression(&for_result.source, result);
            } else if let Some(ref exp) = directive.exp {
                // Before transform, v-for expression is in exp (e.g., "item in items")
                // We need to extract the source part after "in" or "of"
                extract_v_for_source_read_identifiers(exp, result);
            }
        } else if let Some(ref exp) = directive.exp {
            if name == "slot" {
                let mut ids = FxHashSet::default();
                extract_slot_pattern_identifiers(exp, &mut ids);
                insert_read_identifiers(result, ids);
            } else {
                extract_read_identifiers_from_expression(exp, result);
            }
        } else if name == "bind" {
            // v-bind shorthand name as identifier
            if let Some(ref arg) = directive.arg
                && let ExpressionNode::Simple(simple_arg) = arg
                && simple_arg.is_static
            {
                let identifier = camelize(simple_arg.content);
                insert_read_identifier(result, identifier.to_compact_string());
            }
        }
    }
}

fn insert_read_identifier(result: &mut TemplateIdentifierAnalysis, identifier: String) {
    result.used_ids.insert(identifier.clone());
    result.read_ids.insert(identifier);
}

fn insert_read_identifiers(
    result: &mut TemplateIdentifierAnalysis,
    identifiers: FxHashSet<String>,
) {
    for identifier in identifiers {
        insert_read_identifier(result, identifier);
    }
}

fn extract_read_identifiers_from_expression(
    node: &ExpressionNode,
    result: &mut TemplateIdentifierAnalysis,
) {
    let mut ids = FxHashSet::default();
    extract_identifiers_from_expression(node, &mut ids);
    insert_read_identifiers(result, ids);
}

fn extract_read_identifiers_from_compound(
    node: &vize_atelier_core::CompoundExpressionNode,
    result: &mut TemplateIdentifierAnalysis,
) {
    let mut ids = FxHashSet::default();
    extract_identifiers_from_compound(node, &mut ids);
    insert_read_identifiers(result, ids);
}

/// Extract source identifiers from v-for expression.
/// Handles expressions like "item in items", "(item, index) in items", "item of items"
fn extract_v_for_source_read_identifiers(
    exp: &ExpressionNode,
    result: &mut TemplateIdentifierAnalysis,
) {
    let mut ids = FxHashSet::default();
    extract_v_for_source_identifiers(exp, &mut ids);
    insert_read_identifiers(result, ids);
}

fn extract_v_for_source_identifiers(exp: &ExpressionNode, ids: &mut FxHashSet<String>) {
    if let ExpressionNode::Simple(simple) = exp {
        let content = simple.content;

        // Find " in " or " of " to split the expression
        let source_part = if let Some((_, source)) = content.split_once(" in ") {
            source
        } else if let Some((_, source)) = content.split_once(" of ") {
            source
        } else {
            // No "in" or "of" found, use the whole expression
            content
        };

        let source_trimmed = source_part.trim();
        if !source_trimmed.is_empty() && is_simple_identifier(source_trimmed) {
            ids.insert(source_trimmed.to_compact_string());
        }
    }
}

#[cfg(test)]
mod tests;
