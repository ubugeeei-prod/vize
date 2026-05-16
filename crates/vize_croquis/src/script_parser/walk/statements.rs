//! Statement walking for scope discovery.
//!
//! Walks statement nodes to find block scopes (if, for, while, try/catch),
//! nested function/class declarations, and variable declarations.

use oxc_ast::ast::Statement;

use super::{
    add_binding_pattern_to_scope, extract_function_params, extract_param_names, walk_expression,
    BindingType, BlockKind, BlockScopeData, ClosureScopeData, CompactString, GetSpan, ScopeBinding,
    ScriptParseResult,
};

/// Walk a statement to find nested scopes
#[inline]
pub(in crate::script_parser) fn walk_statement(
    result: &mut ScriptParseResult,
    stmt: &Statement<'_>,
    source: &str,
) {
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => {
            walk_expression(result, &expr_stmt.expression, source);
        }
        Statement::VariableDeclaration(var_decl) => {
            // Add variable bindings to current scope and check for reactivity losses
            for decl in var_decl.declarations.iter() {
                add_binding_pattern_to_scope(result, &decl.id, decl.span.start);
                if let Some(init) = &decl.init {
                    walk_expression(result, init, source);

                    // Check for ref.value extraction: const x = someRef.value
                    // This also applies in block scopes (e.g., { const x = countRef.value })
                    super::super::extract::check_ref_value_extraction(result, &decl.id, init);
                    super::super::extract::check_reactive_property_extraction(
                        result, &decl.id, init,
                    );
                    super::super::extract::check_getter_call_extraction(result, &decl.id, init);
                    super::super::extract::check_reactive_plain_alias_extraction(
                        result, &decl.id, init,
                    );
                }
            }
        }
        // Nested function declarations
        Statement::FunctionDeclaration(func) => {
            // Add function name as binding
            if let Some(id) = &func.id {
                result.scopes.add_binding(
                    CompactString::new(id.name.as_str()),
                    ScopeBinding::new(BindingType::SetupConst, func.span.start),
                );
            }

            // Create closure scope
            let params = extract_function_params(&func.params);
            let name = func
                .id
                .as_ref()
                .map(|id| CompactString::new(id.name.as_str()));

            result.scopes.enter_closure_scope(
                ClosureScopeData {
                    name,
                    param_names: params,
                    is_arrow: false,
                    is_async: func.r#async,
                    is_generator: func.generator,
                },
                func.span.start,
                func.span.end,
            );

            if let Some(body) = &func.body {
                for stmt in body.statements.iter() {
                    walk_statement(result, stmt, source);
                }
            }

            result.scopes.exit_scope();
        }
        // Nested class declarations
        Statement::ClassDeclaration(class) => {
            // Add class name as binding
            if let Some(id) = &class.id {
                result.scopes.add_binding(
                    CompactString::new(id.name.as_str()),
                    ScopeBinding::new(BindingType::SetupConst, class.span.start),
                );
            }
            // Walk class body for methods
            for element in class.body.body.iter() {
                if let oxc_ast::ast::ClassElement::MethodDefinition(method) = element {
                    if let Some(body) = &method.value.body {
                        let params = extract_function_params(&method.value.params);
                        result.scopes.enter_closure_scope(
                            ClosureScopeData {
                                name: None,
                                param_names: params,
                                is_arrow: false,
                                is_async: method.value.r#async,
                                is_generator: method.value.generator,
                            },
                            method.span.start,
                            method.span.end,
                        );
                        for stmt in body.statements.iter() {
                            walk_statement(result, stmt, source);
                        }
                        result.scopes.exit_scope();
                    }
                }
            }
        }
        Statement::ReturnStatement(ret) => {
            if let Some(arg) = &ret.argument {
                walk_expression(result, arg, source);
            }
        }
        Statement::BlockStatement(block) => {
            result.scopes.enter_block_scope(
                BlockScopeData {
                    kind: BlockKind::Block,
                },
                block.span.start,
                block.span.end,
            );
            for stmt in block.body.iter() {
                walk_statement(result, stmt, source);
            }
            result.scopes.exit_scope();
        }
        Statement::IfStatement(if_stmt) => {
            walk_expression(result, &if_stmt.test, source);

            // Consequent block
            result.scopes.enter_block_scope(
                BlockScopeData {
                    kind: BlockKind::If,
                },
                if_stmt.consequent.span().start,
                if_stmt.consequent.span().end,
            );
            walk_statement(result, &if_stmt.consequent, source);
            result.scopes.exit_scope();

            // Alternate block (else/else if)
            if let Some(alt) = &if_stmt.alternate {
                result.scopes.enter_block_scope(
                    BlockScopeData {
                        kind: BlockKind::Else,
                    },
                    alt.span().start,
                    alt.span().end,
                );
                walk_statement(result, alt, source);
                result.scopes.exit_scope();
            }
        }
        Statement::ForStatement(for_stmt) => {
            result.scopes.enter_block_scope(
                BlockScopeData {
                    kind: BlockKind::For,
                },
                for_stmt.span.start,
                for_stmt.span.end,
            );
            // Add loop variable bindings
            if let Some(init) = &for_stmt.init {
                match init {
                    oxc_ast::ast::ForStatementInit::VariableDeclaration(var_decl) => {
                        for decl in var_decl.declarations.iter() {
                            add_binding_pattern_to_scope(result, &decl.id, decl.span.start);
                            if let Some(init_expr) = &decl.init {
                                walk_expression(result, init_expr, source);
                            }
                        }
                    }
                    _ => {
                        // Expression init (e.g., for (i = 0; ...))
                        if let Some(expr) = init.as_expression() {
                            walk_expression(result, expr, source);
                        }
                    }
                }
            }
            if let Some(test) = &for_stmt.test {
                walk_expression(result, test, source);
            }
            if let Some(update) = &for_stmt.update {
                walk_expression(result, update, source);
            }
            walk_statement(result, &for_stmt.body, source);
            result.scopes.exit_scope();
        }
        Statement::ForInStatement(for_in) => {
            result.scopes.enter_block_scope(
                BlockScopeData {
                    kind: BlockKind::ForIn,
                },
                for_in.span.start,
                for_in.span.end,
            );
            // Add loop variable binding
            if let oxc_ast::ast::ForStatementLeft::VariableDeclaration(var_decl) = &for_in.left {
                for decl in var_decl.declarations.iter() {
                    add_binding_pattern_to_scope(result, &decl.id, decl.span.start);
                }
            }
            walk_expression(result, &for_in.right, source);
            walk_statement(result, &for_in.body, source);
            result.scopes.exit_scope();
        }
        Statement::ForOfStatement(for_of) => {
            result.scopes.enter_block_scope(
                BlockScopeData {
                    kind: BlockKind::ForOf,
                },
                for_of.span.start,
                for_of.span.end,
            );
            // Add loop variable binding
            if let oxc_ast::ast::ForStatementLeft::VariableDeclaration(var_decl) = &for_of.left {
                for decl in var_decl.declarations.iter() {
                    add_binding_pattern_to_scope(result, &decl.id, decl.span.start);
                }
            }
            walk_expression(result, &for_of.right, source);
            walk_statement(result, &for_of.body, source);
            result.scopes.exit_scope();
        }
        Statement::WhileStatement(while_stmt) => {
            result.scopes.enter_block_scope(
                BlockScopeData {
                    kind: BlockKind::While,
                },
                while_stmt.span.start,
                while_stmt.span.end,
            );
            walk_expression(result, &while_stmt.test, source);
            walk_statement(result, &while_stmt.body, source);
            result.scopes.exit_scope();
        }
        Statement::DoWhileStatement(do_while) => {
            result.scopes.enter_block_scope(
                BlockScopeData {
                    kind: BlockKind::DoWhile,
                },
                do_while.span.start,
                do_while.span.end,
            );
            walk_statement(result, &do_while.body, source);
            walk_expression(result, &do_while.test, source);
            result.scopes.exit_scope();
        }
        Statement::SwitchStatement(switch_stmt) => {
            walk_expression(result, &switch_stmt.discriminant, source);
            result.scopes.enter_block_scope(
                BlockScopeData {
                    kind: BlockKind::Switch,
                },
                switch_stmt.span.start,
                switch_stmt.span.end,
            );
            for case in switch_stmt.cases.iter() {
                if let Some(test) = &case.test {
                    walk_expression(result, test, source);
                }
                for stmt in case.consequent.iter() {
                    walk_statement(result, stmt, source);
                }
            }
            result.scopes.exit_scope();
        }
        Statement::TryStatement(try_stmt) => {
            // try block
            result.scopes.enter_block_scope(
                BlockScopeData {
                    kind: BlockKind::Try,
                },
                try_stmt.block.span.start,
                try_stmt.block.span.end,
            );
            for stmt in try_stmt.block.body.iter() {
                walk_statement(result, stmt, source);
            }
            result.scopes.exit_scope();

            // catch block
            if let Some(handler) = &try_stmt.handler {
                result.scopes.enter_block_scope(
                    BlockScopeData {
                        kind: BlockKind::Catch,
                    },
                    handler.span.start,
                    handler.span.end,
                );
                // Add catch parameter as binding if present
                if let Some(param) = &handler.param {
                    let mut names = vize_carton::SmallVec::<[CompactString; 4]>::new();
                    extract_param_names(&param.pattern, &mut names);
                    for name in names {
                        result.scopes.add_binding(
                            name,
                            ScopeBinding::new(BindingType::SetupConst, handler.span.start),
                        );
                    }
                }
                for stmt in handler.body.body.iter() {
                    walk_statement(result, stmt, source);
                }
                result.scopes.exit_scope();
            }

            // finally block
            if let Some(finalizer) = &try_stmt.finalizer {
                result.scopes.enter_block_scope(
                    BlockScopeData {
                        kind: BlockKind::Finally,
                    },
                    finalizer.span.start,
                    finalizer.span.end,
                );
                for stmt in finalizer.body.iter() {
                    walk_statement(result, stmt, source);
                }
                result.scopes.exit_scope();
            }
        }
        Statement::WithStatement(with_stmt) => {
            walk_expression(result, &with_stmt.object, source);
            result.scopes.enter_block_scope(
                BlockScopeData {
                    kind: BlockKind::With,
                },
                with_stmt.body.span().start,
                with_stmt.body.span().end,
            );
            walk_statement(result, &with_stmt.body, source);
            result.scopes.exit_scope();
        }
        _ => {}
    }
}
