//! AST-based identifier rewrite collection.
//!
//! Walks the AST to find identifier references to destructured props,
//! collecting rewrites while respecting lexical scoping and shadowing.

use oxc_ast::ast::{BindingPattern, Expression, Program, Statement};
use vize_carton::FxHashMap;

use super::helpers::gen_props_access_exp;
use vize_carton::{String, ToCompactString};

/// Collect identifier rewrites from AST
pub(crate) fn collect_identifier_rewrites<'a>(
    program: &Program<'a>,
    source: &str,
    local_to_key: &FxHashMap<&str, &str>,
    rewrites: &mut Vec<(usize, usize, String)>,
) {
    // Track local bindings that shadow destructured props
    let mut local_bindings: FxHashMap<String, bool> = FxHashMap::default();

    // Walk statements
    for stmt in program.body.iter() {
        collect_from_statement(stmt, source, local_to_key, &mut local_bindings, rewrites);
    }
}

pub(crate) fn collect_from_statement<'a>(
    stmt: &Statement<'a>,
    source: &str,
    local_to_key: &FxHashMap<&str, &str>,
    local_bindings: &mut FxHashMap<String, bool>,
    rewrites: &mut Vec<(usize, usize, String)>,
) {
    match stmt {
        Statement::VariableDeclaration(decl) => {
            for declarator in decl.declarations.iter() {
                // Check initializer BEFORE registering bindings
                // (so we don't accidentally skip references to props that will be shadowed)
                if let Some(init) = &declarator.init {
                    collect_from_expression(init, source, local_to_key, local_bindings, rewrites);
                }
                // Register local bindings
                register_binding_pattern(&declarator.id, local_bindings);
            }
        }
        Statement::ExpressionStatement(expr_stmt) => {
            collect_from_expression(
                &expr_stmt.expression,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
        }
        Statement::ReturnStatement(ret) => {
            if let Some(arg) = &ret.argument {
                collect_from_expression(arg, source, local_to_key, local_bindings, rewrites);
            }
        }
        Statement::IfStatement(if_stmt) => {
            collect_from_expression(
                &if_stmt.test,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
            // Walk consequent
            collect_from_statement(
                &if_stmt.consequent,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
            // Walk alternate if present
            if let Some(alt) = &if_stmt.alternate {
                collect_from_statement(alt, source, local_to_key, local_bindings, rewrites);
            }
        }
        Statement::BlockStatement(block) => {
            // Create new scope for block
            let mut inner_bindings = local_bindings.clone();
            // First pass: collect variable declarations in this block
            for block_stmt in block.body.iter() {
                if let Statement::VariableDeclaration(decl) = block_stmt {
                    for declarator in decl.declarations.iter() {
                        register_binding_pattern(&declarator.id, &mut inner_bindings);
                    }
                }
            }
            // Second pass: walk statements with updated bindings
            for block_stmt in block.body.iter() {
                collect_from_statement(
                    block_stmt,
                    source,
                    local_to_key,
                    &mut inner_bindings,
                    rewrites,
                );
            }
        }
        Statement::FunctionDeclaration(func) => {
            // Register function name as local binding
            if let Some(id) = &func.id {
                local_bindings.insert(id.name.to_compact_string(), true);
            }
            // Walk function body with new scope
            if let Some(body) = &func.body {
                let mut inner_bindings = local_bindings.clone();
                // Register parameters
                for param in func.params.items.iter() {
                    register_binding_pattern(&param.pattern, &mut inner_bindings);
                }
                // Walk body statements
                for body_stmt in body.statements.iter() {
                    collect_from_statement(
                        body_stmt,
                        source,
                        local_to_key,
                        &mut inner_bindings,
                        rewrites,
                    );
                }
            }
        }
        Statement::ForStatement(for_stmt) => {
            let mut inner_bindings = local_bindings.clone();
            // Handle init
            if let Some(init) = &for_stmt.init {
                match init {
                    oxc_ast::ast::ForStatementInit::VariableDeclaration(decl) => {
                        for declarator in decl.declarations.iter() {
                            if let Some(init_expr) = &declarator.init {
                                collect_from_expression(
                                    init_expr,
                                    source,
                                    local_to_key,
                                    &inner_bindings,
                                    rewrites,
                                );
                            }
                            register_binding_pattern(&declarator.id, &mut inner_bindings);
                        }
                    }
                    _ => {
                        if let Some(expr) = init.as_expression() {
                            collect_from_expression(
                                expr,
                                source,
                                local_to_key,
                                &inner_bindings,
                                rewrites,
                            );
                        }
                    }
                }
            }
            // Handle test
            if let Some(test) = &for_stmt.test {
                collect_from_expression(test, source, local_to_key, &inner_bindings, rewrites);
            }
            // Handle update
            if let Some(update) = &for_stmt.update {
                collect_from_expression(update, source, local_to_key, &inner_bindings, rewrites);
            }
            // Handle body
            collect_from_statement(
                &for_stmt.body,
                source,
                local_to_key,
                &mut inner_bindings,
                rewrites,
            );
        }
        Statement::ForInStatement(for_in) => {
            let mut inner_bindings = local_bindings.clone();
            // Handle left (binding)
            if let oxc_ast::ast::ForStatementLeft::VariableDeclaration(decl) = &for_in.left {
                for declarator in decl.declarations.iter() {
                    register_binding_pattern(&declarator.id, &mut inner_bindings);
                }
            }
            // Handle right (collection being iterated)
            collect_from_expression(
                &for_in.right,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
            // Handle body
            collect_from_statement(
                &for_in.body,
                source,
                local_to_key,
                &mut inner_bindings,
                rewrites,
            );
        }
        Statement::ForOfStatement(for_of) => {
            let mut inner_bindings = local_bindings.clone();
            // Handle left (binding)
            if let oxc_ast::ast::ForStatementLeft::VariableDeclaration(decl) = &for_of.left {
                for declarator in decl.declarations.iter() {
                    register_binding_pattern(&declarator.id, &mut inner_bindings);
                }
            }
            // Handle right (collection being iterated)
            collect_from_expression(
                &for_of.right,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
            // Handle body
            collect_from_statement(
                &for_of.body,
                source,
                local_to_key,
                &mut inner_bindings,
                rewrites,
            );
        }
        Statement::WhileStatement(while_stmt) => {
            collect_from_expression(
                &while_stmt.test,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
            collect_from_statement(
                &while_stmt.body,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
        }
        Statement::DoWhileStatement(do_while) => {
            collect_from_statement(
                &do_while.body,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
            collect_from_expression(
                &do_while.test,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
        }
        Statement::TryStatement(try_stmt) => {
            // Walk try block
            for try_body_stmt in try_stmt.block.body.iter() {
                collect_from_statement(
                    try_body_stmt,
                    source,
                    local_to_key,
                    local_bindings,
                    rewrites,
                );
            }
            // Walk catch clause with new scope
            if let Some(handler) = &try_stmt.handler {
                let mut catch_bindings = local_bindings.clone();
                // Register catch parameter
                if let Some(param) = &handler.param {
                    register_binding_pattern(&param.pattern, &mut catch_bindings);
                }
                // Walk catch body
                for catch_stmt in handler.body.body.iter() {
                    collect_from_statement(
                        catch_stmt,
                        source,
                        local_to_key,
                        &mut catch_bindings,
                        rewrites,
                    );
                }
            }
            // Walk finally block
            if let Some(finalizer) = &try_stmt.finalizer {
                for finally_stmt in finalizer.body.iter() {
                    collect_from_statement(
                        finally_stmt,
                        source,
                        local_to_key,
                        local_bindings,
                        rewrites,
                    );
                }
            }
        }
        Statement::SwitchStatement(switch) => {
            collect_from_expression(
                &switch.discriminant,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
            for case in switch.cases.iter() {
                if let Some(test) = &case.test {
                    collect_from_expression(test, source, local_to_key, local_bindings, rewrites);
                }
                for case_stmt in case.consequent.iter() {
                    collect_from_statement(
                        case_stmt,
                        source,
                        local_to_key,
                        local_bindings,
                        rewrites,
                    );
                }
            }
        }
        Statement::ThrowStatement(throw) => {
            collect_from_expression(
                &throw.argument,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
        }
        Statement::LabeledStatement(labeled) => {
            collect_from_statement(
                &labeled.body,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
        }
        Statement::WithStatement(with) => {
            collect_from_expression(&with.object, source, local_to_key, local_bindings, rewrites);
            collect_from_statement(&with.body, source, local_to_key, local_bindings, rewrites);
        }
        _ => {}
    }
}

pub(crate) fn collect_from_expression<'a>(
    expr: &Expression<'a>,
    source: &str,
    local_to_key: &FxHashMap<&str, &str>,
    local_bindings: &FxHashMap<String, bool>,
    rewrites: &mut Vec<(usize, usize, String)>,
) {
    match expr {
        Expression::Identifier(id) => {
            let name = id.name.as_str();
            // Check if this is a destructured prop and not shadowed
            if let Some(key) = local_to_key.get(name) {
                if !local_bindings.contains_key(name) {
                    rewrites.push((
                        id.span.start as usize,
                        id.span.end as usize,
                        gen_props_access_exp(key),
                    ));
                }
            }
        }
        Expression::CallExpression(call) => {
            // Check arguments
            for arg in call.arguments.iter() {
                if let Some(expr) = arg.as_expression() {
                    collect_from_expression(expr, source, local_to_key, local_bindings, rewrites);
                }
            }
            // Check callee
            collect_from_expression(&call.callee, source, local_to_key, local_bindings, rewrites);
        }
        Expression::ArrowFunctionExpression(arrow) => {
            // Create new scope for arrow function
            let mut inner_bindings = local_bindings.clone();
            // Register parameters
            for param in arrow.params.items.iter() {
                register_binding_pattern(&param.pattern, &mut inner_bindings);
            }
            // Walk body statements - for expression bodies, OXC wraps the expression in a statement
            for stmt in arrow.body.statements.iter() {
                collect_from_statement(stmt, source, local_to_key, &mut inner_bindings, rewrites);
            }
        }
        Expression::FunctionExpression(func) => {
            // Create new scope for function
            let mut inner_bindings = local_bindings.clone();
            // Register parameters
            for param in func.params.items.iter() {
                register_binding_pattern(&param.pattern, &mut inner_bindings);
            }
            // Walk body statements
            if let Some(body) = &func.body {
                for stmt in body.statements.iter() {
                    collect_from_statement(
                        stmt,
                        source,
                        local_to_key,
                        &mut inner_bindings,
                        rewrites,
                    );
                }
            }
        }
        Expression::BinaryExpression(bin) => {
            collect_from_expression(&bin.left, source, local_to_key, local_bindings, rewrites);
            collect_from_expression(&bin.right, source, local_to_key, local_bindings, rewrites);
        }
        _ if expr.is_member_expression() => {
            // Handle MemberExpression via helper method
            if let Some(member) = expr.as_member_expression() {
                collect_from_expression(
                    member.object(),
                    source,
                    local_to_key,
                    local_bindings,
                    rewrites,
                );
            }
        }
        Expression::ObjectExpression(obj) => {
            for prop in obj.properties.iter() {
                match prop {
                    oxc_ast::ast::ObjectPropertyKind::ObjectProperty(p) => {
                        // Check for shorthand: { foo } should become { foo: __props.foo }
                        if p.shorthand {
                            if let oxc_ast::ast::PropertyKey::StaticIdentifier(id) = &p.key {
                                let name = id.name.as_str();
                                if let Some(key) = local_to_key.get(name) {
                                    if !local_bindings.contains_key(name) {
                                        // For shorthand, we need to expand it
                                        // { foo } -> { foo: __props.foo }
                                        let end = p.span.end as usize;
                                        let access = gen_props_access_exp(key);
                                        let mut suffix = String::with_capacity(access.len() + 2);
                                        suffix.push_str(": ");
                                        suffix.push_str(&access);
                                        rewrites.push((end, end, suffix));
                                    }
                                }
                            }
                        } else {
                            collect_from_expression(
                                &p.value,
                                source,
                                local_to_key,
                                local_bindings,
                                rewrites,
                            );
                        }
                    }
                    oxc_ast::ast::ObjectPropertyKind::SpreadProperty(spread) => {
                        collect_from_expression(
                            &spread.argument,
                            source,
                            local_to_key,
                            local_bindings,
                            rewrites,
                        );
                    }
                }
            }
        }
        Expression::ArrayExpression(arr) => {
            for elem in arr.elements.iter() {
                match elem {
                    oxc_ast::ast::ArrayExpressionElement::SpreadElement(spread) => {
                        collect_from_expression(
                            &spread.argument,
                            source,
                            local_to_key,
                            local_bindings,
                            rewrites,
                        );
                    }
                    oxc_ast::ast::ArrayExpressionElement::Elision(_) => {}
                    _ => {
                        if let Some(e) = elem.as_expression() {
                            collect_from_expression(
                                e,
                                source,
                                local_to_key,
                                local_bindings,
                                rewrites,
                            );
                        }
                    }
                }
            }
        }
        Expression::TemplateLiteral(template) => {
            for expr in template.expressions.iter() {
                collect_from_expression(expr, source, local_to_key, local_bindings, rewrites);
            }
        }
        Expression::ConditionalExpression(cond) => {
            collect_from_expression(&cond.test, source, local_to_key, local_bindings, rewrites);
            collect_from_expression(
                &cond.consequent,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
            collect_from_expression(
                &cond.alternate,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
        }
        Expression::AssignmentExpression(assign) => {
            // Handle: target = value (e.g., title.value = alt.replace(...))
            // Visit the right side for prop references
            collect_from_expression(
                &assign.right,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
            // Visit the left side for computed member access (e.g., obj[prop] = ...)
            match &assign.left {
                oxc_ast::ast::AssignmentTarget::StaticMemberExpression(member) => {
                    collect_from_expression(
                        &member.object,
                        source,
                        local_to_key,
                        local_bindings,
                        rewrites,
                    );
                }
                oxc_ast::ast::AssignmentTarget::ComputedMemberExpression(member) => {
                    collect_from_expression(
                        &member.object,
                        source,
                        local_to_key,
                        local_bindings,
                        rewrites,
                    );
                    collect_from_expression(
                        &member.expression,
                        source,
                        local_to_key,
                        local_bindings,
                        rewrites,
                    );
                }
                _ => {}
            }
        }
        Expression::LogicalExpression(log) => {
            collect_from_expression(&log.left, source, local_to_key, local_bindings, rewrites);
            collect_from_expression(&log.right, source, local_to_key, local_bindings, rewrites);
        }
        Expression::UnaryExpression(unary) => {
            collect_from_expression(
                &unary.argument,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
        }
        Expression::AwaitExpression(await_expr) => {
            collect_from_expression(
                &await_expr.argument,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
        }
        Expression::NewExpression(new_expr) => {
            collect_from_expression(
                &new_expr.callee,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
            for arg in new_expr.arguments.iter() {
                if let Some(e) = arg.as_expression() {
                    collect_from_expression(e, source, local_to_key, local_bindings, rewrites);
                }
            }
        }
        Expression::SequenceExpression(seq) => {
            for expr in seq.expressions.iter() {
                collect_from_expression(expr, source, local_to_key, local_bindings, rewrites);
            }
        }
        Expression::TaggedTemplateExpression(tagged) => {
            collect_from_expression(&tagged.tag, source, local_to_key, local_bindings, rewrites);
            for expr in tagged.quasi.expressions.iter() {
                collect_from_expression(expr, source, local_to_key, local_bindings, rewrites);
            }
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            collect_from_expression(
                &ts_non_null.expression,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
        }
        Expression::TSAsExpression(ts_as) => {
            collect_from_expression(
                &ts_as.expression,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            collect_from_expression(
                &ts_satisfies.expression,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
        }
        Expression::ParenthesizedExpression(paren) => {
            collect_from_expression(
                &paren.expression,
                source,
                local_to_key,
                local_bindings,
                rewrites,
            );
        }
        Expression::ChainExpression(chain) => {
            // Handle optional chaining: socialUrls?.x, foo?.bar()
            match &chain.expression {
                oxc_ast::ast::ChainElement::CallExpression(call) => {
                    for arg in call.arguments.iter() {
                        if let Some(e) = arg.as_expression() {
                            collect_from_expression(
                                e,
                                source,
                                local_to_key,
                                local_bindings,
                                rewrites,
                            );
                        }
                    }
                    collect_from_expression(
                        &call.callee,
                        source,
                        local_to_key,
                        local_bindings,
                        rewrites,
                    );
                }
                oxc_ast::ast::ChainElement::StaticMemberExpression(member) => {
                    collect_from_expression(
                        &member.object,
                        source,
                        local_to_key,
                        local_bindings,
                        rewrites,
                    );
                }
                oxc_ast::ast::ChainElement::ComputedMemberExpression(member) => {
                    collect_from_expression(
                        &member.object,
                        source,
                        local_to_key,
                        local_bindings,
                        rewrites,
                    );
                    collect_from_expression(
                        &member.expression,
                        source,
                        local_to_key,
                        local_bindings,
                        rewrites,
                    );
                }
                oxc_ast::ast::ChainElement::PrivateFieldExpression(member) => {
                    collect_from_expression(
                        &member.object,
                        source,
                        local_to_key,
                        local_bindings,
                        rewrites,
                    );
                }
                _ => {}
            }
        }
        _ => {}
    }
}

pub(crate) fn register_binding_pattern(
    pattern: &BindingPattern<'_>,
    bindings: &mut FxHashMap<String, bool>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            bindings.insert(id.name.to_compact_string(), true);
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in obj.properties.iter() {
                register_binding_pattern(&prop.value, bindings);
            }
            if let Some(rest) = &obj.rest {
                register_binding_pattern(&rest.argument, bindings);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                register_binding_pattern(elem, bindings);
            }
            if let Some(rest) = &arr.rest {
                register_binding_pattern(&rest.argument, bindings);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            register_binding_pattern(&assign.left, bindings);
        }
    }
}
