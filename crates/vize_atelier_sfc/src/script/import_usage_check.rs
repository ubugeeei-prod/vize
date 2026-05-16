//! Import usage check for SFC templates.
//!
//! This module checks if an identifier is used in the SFC's template.
//! - Used to determine the properties that should be included in the object returned from setup()
//!   when not using inline mode.
//! - Check whether the built-in properties such as $attrs, $slots, $emit are used in the template

use oxc_allocator::Allocator;
use oxc_ast::ast as oxc_ast_types;
use oxc_ast_visit::{walk::walk_arrow_function_expression, Visit};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_core::ast::{
    DirectiveNode, ElementNode, ExpressionNode, PropNode, RootNode, SimpleExpressionNode,
    TemplateChildNode,
};
use vize_carton::{
    camelize, capitalize, is_builtin_directive, is_native_tag, is_simple_identifier, FxHashSet,
    String, ToCompactString,
};
use vize_croquis::builtins::{is_builtin_component, is_global_allowed};

/// Result of template analysis
#[derive(Debug, Clone, Default)]
pub struct TemplateUsedIdentifiers {
    /// All identifiers used in the template
    pub used_ids: FxHashSet<String>,
    /// Identifiers used in v-model expressions (simple identifiers only)
    pub v_model_ids: FxHashSet<String>,
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
    resolve_template_analysis_result(root, true)
}

/// Analyze the template and extract identifiers.
///
/// When `collect_used_ids` is false, we skip the expensive identifier extraction
/// and only collect `v_model_ids`.
fn resolve_template_analysis_result(
    root: &RootNode,
    collect_used_ids: bool,
) -> TemplateUsedIdentifiers {
    let mut result = TemplateUsedIdentifiers::default();

    for child in root.children.iter() {
        walk_node(child, &mut result, collect_used_ids);
    }

    result
}

/// Walk a template child node and collect identifiers.
fn walk_node(
    node: &TemplateChildNode,
    result: &mut TemplateUsedIdentifiers,
    collect_used_ids: bool,
) {
    match node {
        TemplateChildNode::Element(element) => {
            walk_element(element, result, collect_used_ids);
        }
        TemplateChildNode::Interpolation(interpolation) => {
            if collect_used_ids {
                extract_identifiers_from_expression(&interpolation.content, &mut result.used_ids);
            }
        }
        TemplateChildNode::If(if_node) => {
            for branch in if_node.branches.iter() {
                // Walk condition expression if present
                if collect_used_ids {
                    if let Some(ref condition) = branch.condition {
                        extract_identifiers_from_expression(condition, &mut result.used_ids);
                    }
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
                extract_identifiers_from_expression(&for_node.source, &mut result.used_ids);
            }
            // Walk children
            for child in for_node.children.iter() {
                walk_node(child, result, collect_used_ids);
            }
        }
        TemplateChildNode::TextCall(text_call) => {
            if collect_used_ids {
                match &text_call.content {
                    vize_atelier_core::ast::TextCallContent::Interpolation(interp) => {
                        extract_identifiers_from_expression(&interp.content, &mut result.used_ids);
                    }
                    vize_atelier_core::ast::TextCallContent::Compound(compound) => {
                        extract_identifiers_from_compound(compound, &mut result.used_ids);
                    }
                    _ => {}
                }
            }
        }
        TemplateChildNode::CompoundExpression(compound) => {
            if collect_used_ids {
                extract_identifiers_from_compound(compound, &mut result.used_ids);
            }
        }
        // Text, Comment, IfBranch, Hoisted don't need processing
        _ => {}
    }
}

/// Walk an element node and collect identifiers.
fn walk_element(
    element: &ElementNode,
    result: &mut TemplateUsedIdentifiers,
    collect_used_ids: bool,
) {
    // Process tag name - check if it's a component
    let mut tag = element.tag.as_str();

    // Handle member expression tags like Foo.Bar
    if let Some(dot_pos) = tag.find('.') {
        tag = &tag[..dot_pos];
    }

    // If not a native tag or built-in component, add to identifiers
    if !is_native_tag(tag) && !is_builtin_component(tag) && collect_used_ids {
        // Add both camelCase and PascalCase versions
        let camelized = camelize(tag);
        let capitalized = capitalize(&camelized);
        result.used_ids.insert(camelized.to_compact_string());
        result.used_ids.insert(capitalized.to_compact_string());
    }

    // Process props
    for prop in element.props.iter() {
        match prop {
            PropNode::Directive(directive) => {
                process_directive(directive, result, collect_used_ids);
            }
            PropNode::Attribute(attr) => {
                // ref attribute value is an identifier
                if collect_used_ids && attr.name.as_str() == "ref" {
                    if let Some(ref value) = attr.value {
                        if !value.content.is_empty() {
                            result.used_ids.insert(value.content.to_compact_string());
                        }
                    }
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
    result: &mut TemplateUsedIdentifiers,
    collect_used_ids: bool,
) {
    let name = directive.name.as_str();

    // Add custom directive to identifiers
    if collect_used_ids && !is_builtin_directive(name) {
        let camel = camelize(name);
        let cap = capitalize(&camel);
        let mut directive_name = String::with_capacity(1 + cap.len());
        directive_name.push('v');
        directive_name.push_str(&cap);
        result.used_ids.insert(directive_name);
    }

    // Collect v-model target identifiers (simple identifiers only)
    if name == "model" {
        if let Some(ref exp) = directive.exp {
            if let ExpressionNode::Simple(simple_exp) = exp {
                let exp_string = simple_exp.content.trim();
                if is_simple_identifier(exp_string) && exp_string != "undefined" {
                    result.v_model_ids.insert(exp_string.to_compact_string());
                }
            }
        }
    }

    // Process dynamic directive arguments
    if collect_used_ids {
        if let Some(ref arg) = directive.arg {
            if let ExpressionNode::Simple(simple_arg) = arg {
                if !simple_arg.is_static {
                    extract_identifiers_from_expression(arg, &mut result.used_ids);
                }
            }
        }
    }

    // Process directive expression
    if collect_used_ids {
        if name == "for" {
            // For v-for, use the parsed source expression if available
            if let Some(ref for_result) = directive.for_parse_result {
                extract_identifiers_from_expression(&for_result.source, &mut result.used_ids);
            } else if let Some(ref exp) = directive.exp {
                // Before transform, v-for expression is in exp (e.g., "item in items")
                // We need to extract the source part after "in" or "of"
                extract_v_for_source_identifiers(exp, &mut result.used_ids);
            }
        } else if let Some(ref exp) = directive.exp {
            extract_identifiers_from_expression(exp, &mut result.used_ids);
        } else if name == "bind" {
            // v-bind shorthand name as identifier
            if let Some(ref arg) = directive.arg {
                if let ExpressionNode::Simple(simple_arg) = arg {
                    if simple_arg.is_static {
                        let identifier = camelize(simple_arg.content.as_str());
                        result.used_ids.insert(identifier.to_compact_string());
                    }
                }
            }
        }
    }
}

/// Extract source identifiers from v-for expression.
/// Handles expressions like "item in items", "(item, index) in items", "item of items"
fn extract_v_for_source_identifiers(exp: &ExpressionNode, ids: &mut FxHashSet<String>) {
    if let ExpressionNode::Simple(simple) = exp {
        let content = simple.content.as_str();

        // Find " in " or " of " to split the expression
        let source_part = if let Some(pos) = content.find(" in ") {
            &content[pos + 4..]
        } else if let Some(pos) = content.find(" of ") {
            &content[pos + 4..]
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

/// Extract identifiers from an expression node.
fn extract_identifiers_from_expression(node: &ExpressionNode, ids: &mut FxHashSet<String>) {
    match node {
        ExpressionNode::Simple(simple) => {
            extract_identifiers_from_simple_expression(simple, ids);
        }
        ExpressionNode::Compound(compound) => {
            extract_identifiers_from_compound(compound, ids);
        }
    }
}

/// Extract identifiers from a simple expression node.
fn extract_identifiers_from_simple_expression(
    node: &SimpleExpressionNode,
    ids: &mut FxHashSet<String>,
) {
    // If the node has pre-parsed identifiers, use them
    if let Some(ref identifiers) = node.identifiers {
        for ident in identifiers.iter() {
            ids.insert(ident.to_compact_string());
        }
        return;
    }

    // If it's a static expression, no identifiers to extract
    if node.is_static {
        return;
    }

    // For simple expressions without parsed AST, treat the whole content as an identifier
    // This matches the TypeScript behavior where node.ast === null means simple identifier
    let content = node.content.trim();
    if !content.is_empty() && is_simple_identifier(content) {
        ids.insert(content.to_compact_string());
    } else if !content.is_empty() {
        extract_identifiers_from_js_expression(content, ids);
    }
}

fn extract_identifiers_from_js_expression(content: &str, ids: &mut FxHashSet<String>) {
    let allocator = Allocator::default();
    let source_type = SourceType::default()
        .with_module(true)
        .with_typescript(true);

    let mut wrapped = String::with_capacity(content.len() + 2);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push(')');

    let parser = Parser::new(&allocator, &wrapped, source_type);
    let Ok(expr) = parser.parse_expression() else {
        return;
    };

    #[derive(Default)]
    struct TemplateIdentifierVisitor {
        ids: FxHashSet<String>,
        local_scope: FxHashSet<String>,
    }

    impl TemplateIdentifierVisitor {
        fn collect_binding_pattern(&mut self, pattern: &oxc_ast_types::BindingPattern<'_>) {
            match pattern {
                oxc_ast_types::BindingPattern::BindingIdentifier(id) => {
                    self.local_scope
                        .insert(id.name.as_str().to_compact_string());
                }
                oxc_ast_types::BindingPattern::ObjectPattern(obj) => {
                    for prop in obj.properties.iter() {
                        self.collect_binding_pattern(&prop.value);
                    }
                    if let Some(rest) = &obj.rest {
                        self.collect_binding_pattern(&rest.argument);
                    }
                }
                oxc_ast_types::BindingPattern::ArrayPattern(arr) => {
                    for elem in arr.elements.iter().flatten() {
                        self.collect_binding_pattern(elem);
                    }
                    if let Some(rest) = &arr.rest {
                        self.collect_binding_pattern(&rest.argument);
                    }
                }
                oxc_ast_types::BindingPattern::AssignmentPattern(assign) => {
                    self.collect_binding_pattern(&assign.left);
                }
            }
        }
    }

    impl<'a> Visit<'a> for TemplateIdentifierVisitor {
        fn visit_identifier_reference(&mut self, ident: &oxc_ast_types::IdentifierReference<'a>) {
            let name = ident.name.as_str();
            if !self.local_scope.contains(name) && !is_global_allowed(name) {
                self.ids.insert(name.to_compact_string());
            }
        }

        fn visit_arrow_function_expression(
            &mut self,
            arrow: &oxc_ast_types::ArrowFunctionExpression<'a>,
        ) {
            let previous = self.local_scope.clone();
            for param in &arrow.params.items {
                self.collect_binding_pattern(&param.pattern);
            }
            if let Some(rest) = &arrow.params.rest {
                self.collect_binding_pattern(&rest.rest.argument);
            }
            walk_arrow_function_expression(self, arrow);
            self.local_scope = previous;
        }
    }

    let mut visitor = TemplateIdentifierVisitor::default();
    visitor.visit_expression(&expr);
    ids.extend(visitor.ids);
}

/// Extract identifiers from a compound expression node.
fn extract_identifiers_from_compound(
    node: &vize_atelier_core::ast::CompoundExpressionNode,
    ids: &mut FxHashSet<String>,
) {
    // Use pre-parsed identifiers if available
    if let Some(ref identifiers) = node.identifiers {
        for ident in identifiers.iter() {
            ids.insert(ident.to_compact_string());
        }
        return;
    }

    // Otherwise, walk children
    for child in node.children.iter() {
        match child {
            vize_atelier_core::ast::CompoundExpressionChild::Simple(simple) => {
                extract_identifiers_from_simple_expression(simple, ids);
            }
            vize_atelier_core::ast::CompoundExpressionChild::Compound(compound) => {
                extract_identifiers_from_compound(compound, ids);
            }
            vize_atelier_core::ast::CompoundExpressionChild::Interpolation(interp) => {
                extract_identifiers_from_expression(&interp.content, ids);
            }
            // Text and Symbol don't contain identifiers
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_used_in_template, resolve_template_used_identifiers, TemplateUsedIdentifiers};
    use vize_atelier_core::parser::parse;
    use vize_carton::Bump;

    fn analyze_template(source: &str) -> TemplateUsedIdentifiers {
        let allocator = Bump::new();
        let (root, _) = parse(&allocator, source);
        resolve_template_used_identifiers(&root)
    }

    fn snapshot_identifiers(result: &TemplateUsedIdentifiers) -> (Vec<&str>, Vec<&str>) {
        let mut used_ids: Vec<_> = result.used_ids.iter().map(|id| id.as_str()).collect();
        used_ids.sort_unstable();

        let mut v_model_ids: Vec<_> = result.v_model_ids.iter().map(|id| id.as_str()).collect();
        v_model_ids.sort_unstable();

        (used_ids, v_model_ids)
    }

    #[test]
    fn test_component_usage() {
        let result = analyze_template("<MyComponent />");
        insta::assert_debug_snapshot!(snapshot_identifiers(&result));
    }

    #[test]
    fn test_component_usage_kebab() {
        let result = analyze_template("<my-component />");
        insta::assert_debug_snapshot!(snapshot_identifiers(&result));
    }

    #[test]
    fn test_component_with_dot() {
        let result = analyze_template("<Foo.Bar />");
        insta::assert_debug_snapshot!(snapshot_identifiers(&result));
    }

    #[test]
    fn test_interpolation() {
        let result = analyze_template("<div>{{ msg }}</div>");
        insta::assert_debug_snapshot!(snapshot_identifiers(&result));
    }

    #[test]
    fn test_v_bind() {
        let result = analyze_template("<div :class=\"classes\"></div>");
        insta::assert_debug_snapshot!(snapshot_identifiers(&result));
    }

    #[test]
    fn test_v_on() {
        let result = analyze_template("<div @click=\"handleClick\"></div>");
        insta::assert_debug_snapshot!(snapshot_identifiers(&result));
    }

    #[test]
    fn test_v_model() {
        let result = analyze_template("<input v-model=\"value\" />");
        insta::assert_debug_snapshot!(snapshot_identifiers(&result));
    }

    #[test]
    fn test_v_model_complex() {
        // Complex expressions should not be added to v_model_ids
        let result = analyze_template("<input v-model=\"obj.value\" />");
        insta::assert_debug_snapshot!(snapshot_identifiers(&result));
    }

    #[test]
    fn test_v_for() {
        let result = analyze_template("<div v-for=\"item in items\">{{ item }}</div>");
        insta::assert_debug_snapshot!(snapshot_identifiers(&result));
    }

    #[test]
    fn test_v_if() {
        let result = analyze_template("<div v-if=\"show\">content</div>");
        insta::assert_debug_snapshot!(snapshot_identifiers(&result));
    }

    #[test]
    fn test_custom_directive() {
        let result = analyze_template("<div v-focus></div>");
        insta::assert_debug_snapshot!(snapshot_identifiers(&result));
    }

    #[test]
    fn test_ref_attribute() {
        let result = analyze_template("<div ref=\"myRef\"></div>");
        insta::assert_debug_snapshot!(snapshot_identifiers(&result));
    }

    #[test]
    fn test_native_tag_not_added() {
        let result = analyze_template("<div></div>");
        insta::assert_debug_snapshot!(snapshot_identifiers(&result));
    }

    #[test]
    fn test_builtin_directive_not_added() {
        let result = analyze_template("<div v-if=\"show\" v-show=\"visible\"></div>");
        insta::assert_debug_snapshot!(snapshot_identifiers(&result));
    }

    #[test]
    fn test_is_used_in_template() {
        let allocator = Bump::new();
        let (root, _) = parse(&allocator, "<div>{{ msg }}</div>");
        assert!(is_used_in_template("msg", &root));
        assert!(!is_used_in_template("other", &root));
    }
}
