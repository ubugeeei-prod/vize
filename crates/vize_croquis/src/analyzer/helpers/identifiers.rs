//! Identifier extraction from Vue template expressions.
//!
//! Provides hybrid extraction strategies:
//! - **Fast path**: String-based scanning for simple expressions
//! - **Slow path**: OXC AST-based extraction for complex expressions
//!   (object literals, type assertions, arrow functions)
//!
//! Only "root" identifiers are extracted -- property accesses like
//! `item.name` yield only `"item"`, not `"name"`.

use std::borrow::Cow;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{profile, CompactString};

#[allow(clippy::disallowed_types)]
/// Strip JS/TS comments while preserving string literals.
pub fn strip_js_comments(expr: &str) -> Cow<'_, str> {
    let bytes = expr.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut changed = false;
    let mut out = std::string::String::with_capacity(expr.len());

    while i < len {
        let c = bytes[i];

        if c == b'\'' || c == b'"' || c == b'`' {
            let quote = c;
            if changed {
                out.push(quote as char);
            }
            i += 1;

            while i < len {
                let current = bytes[i];
                if changed {
                    out.push(current as char);
                }
                i += 1;

                if current == b'\\' {
                    if i < len {
                        if changed {
                            out.push(bytes[i] as char);
                        }
                        i += 1;
                    }
                    continue;
                }

                if current == quote {
                    break;
                }
            }

            continue;
        }

        if c == b'/' && i + 1 < len {
            let next = bytes[i + 1];

            if next == b'/' {
                if !changed {
                    out.push_str(&expr[..i]);
                    changed = true;
                }

                i += 2;
                while i < len && bytes[i] != b'\n' {
                    i += 1;
                }
                if i < len && bytes[i] == b'\n' {
                    out.push('\n');
                    i += 1;
                }
                continue;
            }

            if next == b'*' {
                if !changed {
                    out.push_str(&expr[..i]);
                    changed = true;
                }

                i += 2;
                while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    if bytes[i] == b'\n' {
                        out.push('\n');
                    }
                    i += 1;
                }
                if i + 1 < len {
                    i += 2;
                } else {
                    i = len;
                }
                out.push(' ');
                continue;
            }
        }

        if changed {
            out.push(c as char);
        }
        i += 1;
    }

    if changed {
        Cow::Owned(out)
    } else {
        Cow::Borrowed(expr)
    }
}

/// Hybrid identifier extraction - fast path for simple expressions, OXC for complex ones.
/// Only extracts "root" identifiers - identifiers that are references, not:
/// - Property accesses (item.name -> only "item" extracted)
/// - Object literal keys ({ active: value } -> only "value" extracted)
/// - String literals, computed property names, etc.
#[inline]
pub fn extract_identifiers_oxc(expr: &str) -> Vec<CompactString> {
    let stripped = strip_js_comments(expr);
    let expr = stripped.as_ref();

    // Use OXC parser for complex expressions:
    // - Object literals: { }
    // - Type assertions: as Type
    // - Arrow functions: () =>
    if expr.contains('{') || expr.contains(" as ") || expr.contains("=>") {
        return profile!(
            "croquis.helpers.identifiers.slow",
            extract_identifiers_oxc_slow(expr)
        );
    }

    // Fast path: simple expressions without complex constructs
    profile!(
        "croquis.helpers.identifiers.fast",
        extract_identifiers_fast(expr)
    )
}

/// Fast string-based identifier extraction for simple expressions.
#[inline]
fn extract_identifiers_fast(expr: &str) -> Vec<CompactString> {
    let mut identifiers = Vec::with_capacity(4);
    let bytes = expr.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        let c = bytes[i];

        // Skip single-quoted strings
        if c == b'\'' {
            i += 1;
            while i < len && bytes[i] != b'\'' {
                if bytes[i] == b'\\' && i + 1 < len {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if i < len {
                i += 1;
            }
            continue;
        }

        // Skip double-quoted strings
        if c == b'"' {
            i += 1;
            while i < len && bytes[i] != b'"' {
                if bytes[i] == b'\\' && i + 1 < len {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if i < len {
                i += 1;
            }
            continue;
        }

        // Handle template literals
        if c == b'`' {
            i += 1;
            while i < len {
                if bytes[i] == b'\\' && i + 1 < len {
                    i += 2;
                    continue;
                }
                if bytes[i] == b'`' {
                    i += 1;
                    break;
                }
                if bytes[i] == b'$' && i + 1 < len && bytes[i + 1] == b'{' {
                    i += 2;
                    let interp_start = i;
                    let mut brace_depth = 1;
                    while i < len && brace_depth > 0 {
                        match bytes[i] {
                            b'{' => brace_depth += 1,
                            b'}' => brace_depth -= 1,
                            _ => {}
                        }
                        if brace_depth > 0 {
                            i += 1;
                        }
                    }
                    if interp_start < i {
                        let interp_content = &expr[interp_start..i];
                        for ident in extract_identifiers_fast(interp_content) {
                            identifiers.push(ident);
                        }
                    }
                    if i < len {
                        i += 1;
                    }
                    continue;
                }
                i += 1;
            }
            continue;
        }

        // Start of identifier
        if c.is_ascii_alphabetic() || c == b'_' || c == b'$' {
            let start = i;
            i += 1;
            while i < len
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
            {
                i += 1;
            }

            // Check if preceded by '.' (property access)
            let is_property_access = if start > 0 {
                let mut j = start - 1;
                loop {
                    let prev = bytes[j];
                    if prev == b' ' || prev == b'\t' || prev == b'\n' || prev == b'\r' {
                        if j == 0 {
                            break false;
                        }
                        j -= 1;
                    } else {
                        break prev == b'.';
                    }
                }
            } else {
                false
            };

            if !is_property_access {
                identifiers.push(CompactString::new(&expr[start..i]));
            }
        } else {
            i += 1;
        }
    }

    identifiers
}

/// OXC-based identifier extraction for expressions with object literals.
#[inline]
fn extract_identifiers_oxc_slow(expr: &str) -> Vec<CompactString> {
    use oxc_ast::ast::{
        ArrayExpressionElement, BindingPattern, Expression, ObjectPropertyKind, PropertyKey,
    };

    let allocator = Allocator::default();
    let source_type = SourceType::from_path("expr.ts").unwrap_or_default();

    let ret = profile!(
        "croquis.helpers.identifiers.oxc_parse",
        Parser::new(&allocator, expr, source_type).parse_expression()
    );
    let parsed_expr = match ret {
        Ok(expr) => expr,
        Err(_) => return Vec::new(),
    };

    let mut identifiers = Vec::with_capacity(4);

    // Collect binding names from a pattern (for arrow function parameters)
    fn collect_binding_names<'a>(pattern: &'a BindingPattern<'a>, names: &mut Vec<&'a str>) {
        match pattern {
            BindingPattern::BindingIdentifier(id) => {
                names.push(id.name.as_str());
            }
            BindingPattern::ObjectPattern(obj) => {
                for prop in obj.properties.iter() {
                    collect_binding_names(&prop.value, names);
                }
                if let Some(rest) = &obj.rest {
                    collect_binding_names(&rest.argument, names);
                }
            }
            BindingPattern::ArrayPattern(arr) => {
                for elem in arr.elements.iter().flatten() {
                    collect_binding_names(elem, names);
                }
                if let Some(rest) = &arr.rest {
                    collect_binding_names(&rest.argument, names);
                }
            }
            BindingPattern::AssignmentPattern(assign) => {
                collect_binding_names(&assign.left, names);
            }
        }
    }

    // Recursive AST walker to collect identifier references
    fn walk_expr(expr: &Expression<'_>, identifiers: &mut Vec<CompactString>) {
        match expr {
            // Direct identifier reference - this is what we want
            Expression::Identifier(id) => {
                identifiers.push(CompactString::new(id.name.as_str()));
            }

            // Member expressions - only extract the object, not the property
            Expression::StaticMemberExpression(member) => {
                walk_expr(&member.object, identifiers);
            }
            Expression::ComputedMemberExpression(member) => {
                walk_expr(&member.object, identifiers);
                walk_expr(&member.expression, identifiers);
            }
            Expression::PrivateFieldExpression(field) => {
                walk_expr(&field.object, identifiers);
            }

            // Object expressions - skip keys, only process values
            Expression::ObjectExpression(obj) => {
                for prop in obj.properties.iter() {
                    match prop {
                        ObjectPropertyKind::ObjectProperty(p) => {
                            if p.computed {
                                if let Some(key_expr) = p.key.as_expression() {
                                    walk_expr(key_expr, identifiers);
                                }
                            }
                            if p.shorthand {
                                if let PropertyKey::StaticIdentifier(id) = &p.key {
                                    identifiers.push(CompactString::new(id.name.as_str()));
                                }
                            } else {
                                walk_expr(&p.value, identifiers);
                            }
                        }
                        ObjectPropertyKind::SpreadProperty(spread) => {
                            walk_expr(&spread.argument, identifiers);
                        }
                    }
                }
            }

            // Array expressions
            Expression::ArrayExpression(arr) => {
                for elem in arr.elements.iter() {
                    match elem {
                        ArrayExpressionElement::SpreadElement(spread) => {
                            walk_expr(&spread.argument, identifiers);
                        }
                        ArrayExpressionElement::Elision(_) => {}
                        _ => {
                            if let Some(e) = elem.as_expression() {
                                walk_expr(e, identifiers);
                            }
                        }
                    }
                }
            }

            // Binary/Logical/Conditional expressions
            Expression::BinaryExpression(binary) => {
                walk_expr(&binary.left, identifiers);
                walk_expr(&binary.right, identifiers);
            }
            Expression::LogicalExpression(logical) => {
                walk_expr(&logical.left, identifiers);
                walk_expr(&logical.right, identifiers);
            }
            Expression::ConditionalExpression(cond) => {
                walk_expr(&cond.test, identifiers);
                walk_expr(&cond.consequent, identifiers);
                walk_expr(&cond.alternate, identifiers);
            }

            // Unary expressions
            Expression::UnaryExpression(unary) => {
                walk_expr(&unary.argument, identifiers);
            }
            Expression::UpdateExpression(update) => match &update.argument {
                oxc_ast::ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => {
                    identifiers.push(CompactString::new(id.name.as_str()));
                }
                oxc_ast::ast::SimpleAssignmentTarget::StaticMemberExpression(member) => {
                    walk_expr(&member.object, identifiers);
                }
                oxc_ast::ast::SimpleAssignmentTarget::ComputedMemberExpression(member) => {
                    walk_expr(&member.object, identifiers);
                    walk_expr(&member.expression, identifiers);
                }
                oxc_ast::ast::SimpleAssignmentTarget::PrivateFieldExpression(field) => {
                    walk_expr(&field.object, identifiers);
                }
                _ => {}
            },

            // Call expressions
            Expression::CallExpression(call) => {
                walk_expr(&call.callee, identifiers);
                for arg in call.arguments.iter() {
                    if let Some(e) = arg.as_expression() {
                        walk_expr(e, identifiers);
                    }
                }
            }
            Expression::NewExpression(new_expr) => {
                walk_expr(&new_expr.callee, identifiers);
                for arg in new_expr.arguments.iter() {
                    if let Some(e) = arg.as_expression() {
                        walk_expr(e, identifiers);
                    }
                }
            }

            // Arrow/Function expressions - parameters are local scope, don't extract them
            Expression::ArrowFunctionExpression(arrow) => {
                // Collect parameter names to exclude from identifiers
                let mut param_names: Vec<&str> = Vec::new();
                for param in arrow.params.items.iter() {
                    collect_binding_names(&param.pattern, &mut param_names);
                }

                if arrow.expression {
                    if let Some(oxc_ast::ast::Statement::ExpressionStatement(expr_stmt)) =
                        arrow.body.statements.first()
                    {
                        // Walk body but filter out parameter references
                        let mut body_idents = Vec::new();
                        walk_expr(&expr_stmt.expression, &mut body_idents);
                        for ident in body_idents {
                            if !param_names.contains(&ident.as_str()) {
                                identifiers.push(ident);
                            }
                        }
                    }
                }
            }

            // Sequence expressions
            Expression::SequenceExpression(seq) => {
                for e in seq.expressions.iter() {
                    walk_expr(e, identifiers);
                }
            }

            // Assignment expressions
            Expression::AssignmentExpression(assign) => {
                walk_expr(&assign.right, identifiers);
            }

            // Template literals
            Expression::TemplateLiteral(template) => {
                for expr in template.expressions.iter() {
                    walk_expr(expr, identifiers);
                }
            }
            Expression::TaggedTemplateExpression(tagged) => {
                walk_expr(&tagged.tag, identifiers);
                for expr in tagged.quasi.expressions.iter() {
                    walk_expr(expr, identifiers);
                }
            }

            // Parenthesized/Await/Yield
            Expression::ParenthesizedExpression(paren) => {
                walk_expr(&paren.expression, identifiers);
            }
            Expression::AwaitExpression(await_expr) => {
                walk_expr(&await_expr.argument, identifiers);
            }
            Expression::YieldExpression(yield_expr) => {
                if let Some(arg) = &yield_expr.argument {
                    walk_expr(arg, identifiers);
                }
            }

            // Chained expressions
            Expression::ChainExpression(chain) => match &chain.expression {
                oxc_ast::ast::ChainElement::CallExpression(call) => {
                    walk_expr(&call.callee, identifiers);
                    for arg in call.arguments.iter() {
                        if let Some(e) = arg.as_expression() {
                            walk_expr(e, identifiers);
                        }
                    }
                }
                oxc_ast::ast::ChainElement::TSNonNullExpression(non_null) => {
                    walk_expr(&non_null.expression, identifiers);
                }
                oxc_ast::ast::ChainElement::StaticMemberExpression(member) => {
                    walk_expr(&member.object, identifiers);
                }
                oxc_ast::ast::ChainElement::ComputedMemberExpression(member) => {
                    walk_expr(&member.object, identifiers);
                    walk_expr(&member.expression, identifiers);
                }
                oxc_ast::ast::ChainElement::PrivateFieldExpression(field) => {
                    walk_expr(&field.object, identifiers);
                }
            },

            // TypeScript specific
            Expression::TSAsExpression(as_expr) => {
                walk_expr(&as_expr.expression, identifiers);
            }
            Expression::TSSatisfiesExpression(satisfies) => {
                walk_expr(&satisfies.expression, identifiers);
            }
            Expression::TSNonNullExpression(non_null) => {
                walk_expr(&non_null.expression, identifiers);
            }
            Expression::TSTypeAssertion(assertion) => {
                walk_expr(&assertion.expression, identifiers);
            }
            Expression::TSInstantiationExpression(inst) => {
                walk_expr(&inst.expression, identifiers);
            }

            // Literals - no identifiers
            Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::RegExpLiteral(_) => {}

            _ => {}
        }
    }

    profile!(
        "croquis.helpers.identifiers.walk_expr",
        walk_expr(&parsed_expr, &mut identifiers)
    );
    identifiers
}

#[cfg(test)]
mod tests {
    use super::extract_identifiers_oxc;
    use vize_carton::CompactString;

    #[test]
    fn test_extract_identifiers_oxc() {
        fn to_strings(ids: Vec<CompactString>) -> Vec<CompactString> {
            ids
        }

        let ids = to_strings(extract_identifiers_oxc("count + 1"));
        assert_eq!(ids, vec!["count"]);

        let ids = to_strings(extract_identifiers_oxc("user.name + item.value"));
        assert_eq!(ids, vec!["user", "item"]);

        let ids = to_strings(extract_identifiers_oxc("{ active: isActive }"));
        assert_eq!(ids, vec!["isActive"]);

        let ids = to_strings(extract_identifiers_oxc("{ foo }"));
        assert_eq!(ids, vec!["foo"]);

        let ids = to_strings(extract_identifiers_oxc("cond ? a : b"));
        assert_eq!(ids, vec!["cond", "a", "b"]);
    }

    #[test]
    fn test_extract_identifiers_ignores_comment_words() {
        fn to_strings(ids: Vec<CompactString>) -> Vec<CompactString> {
            ids
        }

        let ids = to_strings(extract_identifiers_oxc(
            "/** comment words should disappear */ disabled ? true : undefined",
        ));
        assert_eq!(ids, vec!["disabled", "true", "undefined"]);
    }
}
