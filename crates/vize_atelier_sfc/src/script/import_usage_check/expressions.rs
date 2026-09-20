//! Expression and binding-pattern reads shared by compiler import retention and Canon.

use oxc_allocator::Allocator;
use oxc_ast::ast as oxc_ast_types;
use oxc_ast_visit::{Visit, walk::walk_arrow_function_expression};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_core::{ExpressionNode, SimpleExpressionNode};
use vize_carton::{FxHashSet, String, ToCompactString, cstr, is_simple_identifier};
use vize_croquis::builtins::is_global_allowed;

pub(super) fn extract_slot_pattern_identifiers(node: &ExpressionNode, ids: &mut FxHashSet<String>) {
    if let ExpressionNode::Simple(simple) = node {
        // A binding pattern is not an object expression: defaults and computed
        // keys read outer bindings, while the declared slot locals do not.
        let expression = cstr!("({}) => {{}}", simple.content);
        extract_identifiers_from_js_expression(&expression, ids);
    } else {
        extract_identifiers_from_expression(node, ids);
    }
}

/// Extract identifiers from an expression node.
pub(super) fn extract_identifiers_from_expression(
    node: &ExpressionNode,
    ids: &mut FxHashSet<String>,
) {
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
pub(super) fn extract_identifiers_from_compound(
    node: &vize_atelier_core::CompoundExpressionNode,
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
            vize_atelier_core::CompoundExpressionChild::Simple(simple) => {
                extract_identifiers_from_simple_expression(simple, ids);
            }
            vize_atelier_core::CompoundExpressionChild::Compound(compound) => {
                extract_identifiers_from_compound(compound, ids);
            }
            vize_atelier_core::CompoundExpressionChild::Interpolation(interp) => {
                extract_identifiers_from_expression(&interp.content, ids);
            }
            // Text and Symbol don't contain identifiers
            _ => {}
        }
    }
}
