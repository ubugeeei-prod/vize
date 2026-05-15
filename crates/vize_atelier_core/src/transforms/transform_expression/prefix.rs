//! Identifier prefix logic for expression transforms.
//!
//! Determines what prefix (if any) an identifier needs based on binding
//! metadata and context (e.g., `_ctx.`, `$setup.`, `__props.`).

use oxc_allocator::Allocator as OxcAllocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{FxHashSet, String};

use vize_croquis::builtins::is_global_allowed;

use crate::transform::TransformContext;

/// Determine what prefix (if any) an identifier needs.
///
/// Returns: `None` = no prefix, `Some("_ctx.")` = context prefix,
/// `Some("__props.")` = props prefix, `Some("$setup.")` = setup context prefix
/// (for function mode with binding metadata).
pub(crate) fn get_identifier_prefix(
    name: &str,
    ctx: &TransformContext<'_>,
) -> Option<&'static str> {
    // Don't prefix globals
    if is_global_allowed(name) {
        return None;
    }

    // Don't prefix if in scope (local variable from v-for, v-slot, etc.)
    if ctx.is_in_scope(name) {
        return None;
    }

    // Check binding metadata for setup bindings
    if let Some(bindings) = &ctx.options.binding_metadata {
        if let Some(binding_type) = bindings.bindings.get(name) {
            // Props need prefix based on mode
            if matches!(
                binding_type,
                crate::options::BindingType::Props | crate::options::BindingType::PropsAliased
            ) {
                // In inline mode: use __props. (local variable in setup)
                // In function mode: use $props. (render function parameter)
                if ctx.options.inline {
                    return Some("__props.");
                } else {
                    return Some("$props.");
                }
            }

            if ctx.options.inline {
                // In inline mode, setup bindings are accessed directly via closure
                return None;
            } else {
                // In function mode (inline = false), setup bindings use $setup. prefix
                // This is the pattern Vue's @vitejs/plugin-vue uses for proper reactivity tracking
                return Some("$setup.");
            }
        }
    }

    // Default: prefix with _ctx.
    Some("_ctx.")
}

/// Check if a simple identifier is a ref binding in inline mode
pub(crate) fn is_ref_binding_simple(name: &str, ctx: &TransformContext<'_>) -> bool {
    if ctx.options.inline {
        // Croquis first: use ReactiveKind for precise determination
        if let Some(kind) = ctx.get_reactive_kind(name) {
            return kind.needs_value_access();
        }
        // Fallback: binding_metadata
        if let Some(bindings) = &ctx.options.binding_metadata {
            if let Some(binding_type) = bindings.bindings.get(name) {
                return matches!(binding_type, crate::options::BindingType::SetupRef);
            }
        }
    }
    false
}

/// Check if string is a simple identifier
pub fn is_simple_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    if !first.is_alphabetic() && first != '_' && first != '$' {
        return false;
    }

    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

/// Prefix identifiers in expression with `_ctx.` for codegen.
///
/// This is a simpler version that doesn't require `TransformContext`.
pub fn prefix_identifiers_in_expression(content: &str) -> String {
    let allocator = OxcAllocator::default();
    let source_type = SourceType::default().with_module(true);

    // Wrap in parentheses to make it a valid expression statement
    let mut wrapped = String::with_capacity(content.len() + 2);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push(')');
    let parser = Parser::new(&allocator, &wrapped, source_type);
    let parse_result = parser.parse_expression();

    match parse_result {
        Ok(expr) => {
            // Collect identifiers and their positions
            let mut rewrites: Vec<(usize, usize, String)> = Vec::new();
            let mut local_vars: FxHashSet<String> = FxHashSet::default();

            collect_identifiers_for_prefix(&expr, &mut rewrites, &mut local_vars, content);

            if rewrites.is_empty() {
                return String::new(content);
            }

            // Sort by position (descending) to apply replacements from end to start
            rewrites.sort_by_key(|rewrite| std::cmp::Reverse(rewrite.0));

            let mut result = String::new(content);
            for (start, end, replacement) in rewrites {
                if start < result.len() && end <= result.len() {
                    result.replace_range(start..end, &replacement);
                }
            }

            result
        }
        Err(_) => String::new(content),
    }
}

/// Collect identifiers that need `_ctx.` prefix
fn collect_identifiers_for_prefix(
    expr: &oxc_ast::ast::Expression<'_>,
    rewrites: &mut Vec<(usize, usize, String)>,
    local_vars: &mut FxHashSet<String>,
    _original: &str,
) {
    use oxc_ast::ast::Expression;

    match expr {
        Expression::Identifier(id) => {
            let name = id.name.as_str();
            // Skip JS globals and local variables
            if !is_global_allowed(name) && !local_vars.contains(name) {
                // Adjust position: subtract 1 for the opening parenthesis we added
                let start = id.span.start as usize - 1;
                let end = id.span.end as usize - 1;
                let mut prefixed = String::with_capacity(5 + name.len());
                prefixed.push_str("_ctx.");
                prefixed.push_str(name);
                rewrites.push((start, end, prefixed));
            }
        }
        Expression::ArrowFunctionExpression(arrow) => {
            // Add arrow function params to local scope
            for param in &arrow.params.items {
                collect_binding_names(&param.pattern, local_vars);
            }
            // Process body statements
            for stmt in arrow.body.statements.iter() {
                if let oxc_ast::ast::Statement::ExpressionStatement(expr_stmt) = stmt {
                    collect_identifiers_for_prefix(
                        &expr_stmt.expression,
                        rewrites,
                        local_vars,
                        _original,
                    );
                }
            }
        }
        Expression::CallExpression(call) => {
            collect_identifiers_for_prefix(&call.callee, rewrites, local_vars, _original);
            for arg in &call.arguments {
                if let oxc_ast::ast::Argument::SpreadElement(spread) = arg {
                    collect_identifiers_for_prefix(
                        &spread.argument,
                        rewrites,
                        local_vars,
                        _original,
                    );
                } else if let Some(expr) = arg.as_expression() {
                    collect_identifiers_for_prefix(expr, rewrites, local_vars, _original);
                }
            }
        }
        Expression::ComputedMemberExpression(computed) => {
            collect_identifiers_for_prefix(&computed.object, rewrites, local_vars, _original);
            collect_identifiers_for_prefix(&computed.expression, rewrites, local_vars, _original);
        }
        Expression::StaticMemberExpression(static_member) => {
            collect_identifiers_for_prefix(&static_member.object, rewrites, local_vars, _original);
            // Don't prefix the property name
        }
        Expression::PrivateFieldExpression(private) => {
            collect_identifiers_for_prefix(&private.object, rewrites, local_vars, _original);
        }
        Expression::ParenthesizedExpression(paren) => {
            collect_identifiers_for_prefix(&paren.expression, rewrites, local_vars, _original);
        }
        Expression::BinaryExpression(binary) => {
            collect_identifiers_for_prefix(&binary.left, rewrites, local_vars, _original);
            collect_identifiers_for_prefix(&binary.right, rewrites, local_vars, _original);
        }
        Expression::ConditionalExpression(cond) => {
            collect_identifiers_for_prefix(&cond.test, rewrites, local_vars, _original);
            collect_identifiers_for_prefix(&cond.consequent, rewrites, local_vars, _original);
            collect_identifiers_for_prefix(&cond.alternate, rewrites, local_vars, _original);
        }
        Expression::LogicalExpression(logical) => {
            collect_identifiers_for_prefix(&logical.left, rewrites, local_vars, _original);
            collect_identifiers_for_prefix(&logical.right, rewrites, local_vars, _original);
        }
        Expression::UnaryExpression(unary) => {
            collect_identifiers_for_prefix(&unary.argument, rewrites, local_vars, _original);
        }
        Expression::ObjectExpression(obj) => {
            for prop in &obj.properties {
                match prop {
                    oxc_ast::ast::ObjectPropertyKind::ObjectProperty(p) => {
                        collect_identifiers_for_prefix(&p.value, rewrites, local_vars, _original);
                    }
                    oxc_ast::ast::ObjectPropertyKind::SpreadProperty(spread) => {
                        collect_identifiers_for_prefix(
                            &spread.argument,
                            rewrites,
                            local_vars,
                            _original,
                        );
                    }
                }
            }
        }
        Expression::ArrayExpression(arr) => {
            for elem in &arr.elements {
                match elem {
                    oxc_ast::ast::ArrayExpressionElement::SpreadElement(spread) => {
                        collect_identifiers_for_prefix(
                            &spread.argument,
                            rewrites,
                            local_vars,
                            _original,
                        );
                    }
                    oxc_ast::ast::ArrayExpressionElement::Elision(_) => {}
                    _ => {
                        if let Some(expr) = elem.as_expression() {
                            collect_identifiers_for_prefix(expr, rewrites, local_vars, _original);
                        }
                    }
                }
            }
        }
        Expression::TemplateLiteral(template) => {
            for expr in &template.expressions {
                collect_identifiers_for_prefix(expr, rewrites, local_vars, _original);
            }
        }
        _ => {}
    }
}

/// Collect binding names from a pattern
pub(crate) fn collect_binding_names(
    pattern: &oxc_ast::ast::BindingPattern<'_>,
    local_vars: &mut FxHashSet<String>,
) {
    match pattern {
        oxc_ast::ast::BindingPattern::BindingIdentifier(id) => {
            local_vars.insert(String::new(id.name.as_str()));
        }
        oxc_ast::ast::BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                collect_binding_names(&prop.value, local_vars);
            }
        }
        oxc_ast::ast::BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                collect_binding_names(elem, local_vars);
            }
        }
        oxc_ast::ast::BindingPattern::AssignmentPattern(assign) => {
            collect_binding_names(&assign.left, local_vars);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_simple_identifier;
    use vize_croquis::builtins::is_global_allowed;

    #[test]
    fn test_is_simple_identifier() {
        assert!(is_simple_identifier("foo"));
        assert!(is_simple_identifier("_bar"));
        assert!(is_simple_identifier("$baz"));
        assert!(is_simple_identifier("foo123"));
        assert!(!is_simple_identifier("123foo"));
        assert!(!is_simple_identifier("foo-bar"));
        assert!(!is_simple_identifier("foo.bar"));
        assert!(!is_simple_identifier(""));
    }

    #[test]
    fn test_js_globals() {
        assert!(is_global_allowed("Array"));
        assert!(is_global_allowed("Object"));
        assert!(is_global_allowed("console"));
        assert!(is_global_allowed("Math"));
        assert!(is_global_allowed("$event"));
        assert!(!is_global_allowed("myVar"));
    }
}
