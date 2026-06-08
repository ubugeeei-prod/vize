use oxc_ast::ast::{Expression, Statement};

use vize_carton::{CompactString, FxHashSet};

use super::super::super::ScriptParseResult;

#[derive(Default)]
pub(super) struct RaceScan {
    pub(super) async_operations: Vec<CompactString>,
    pub(super) mutated_targets: FxHashSet<CompactString>,
    pub(super) cleanup_names: FxHashSet<CompactString>,
    pub(super) has_cleanup_call: bool,
}

impl RaceScan {
    pub(super) fn has_async_boundary(&self) -> bool {
        !self.async_operations.is_empty()
    }

    pub(super) fn add_async_operation(&mut self, operation: &str) {
        if !self
            .async_operations
            .iter()
            .any(|existing| existing.as_str() == operation)
        {
            self.async_operations.push(CompactString::new(operation));
        }
    }

    pub(super) fn primary_async_operation(&self) -> CompactString {
        self.async_operations
            .iter()
            .find(|operation| operation.as_str() != "async callback")
            .or_else(|| self.async_operations.first())
            .cloned()
            .unwrap_or_else(|| CompactString::new("async callback"))
    }

    pub(super) fn mutated_targets(&self) -> Vec<CompactString> {
        let mut targets = self.mutated_targets.iter().cloned().collect::<Vec<_>>();
        targets.sort();
        targets
    }
}

pub(super) fn scan_callback_for_race(
    result: &ScriptParseResult,
    callback: &Expression<'_>,
) -> RaceScan {
    let mut scan = RaceScan::default();
    for name in callback_param_names(callback) {
        scan.cleanup_names.insert(name);
    }

    match callback {
        Expression::ArrowFunctionExpression(arrow) => {
            if arrow.r#async {
                scan.add_async_operation("async callback");
            }
            for stmt in arrow.body.statements.iter() {
                scan_statement_for_race(result, stmt, &mut scan);
            }
        }
        Expression::FunctionExpression(func) => {
            if func.r#async {
                scan.add_async_operation("async callback");
            }
            if let Some(body) = &func.body {
                for stmt in body.statements.iter() {
                    scan_statement_for_race(result, stmt, &mut scan);
                }
            }
        }
        _ => super::expression::scan_expression_for_race(result, callback, &mut scan),
    }

    scan
}

fn callback_param_names(callback: &Expression<'_>) -> Vec<CompactString> {
    match callback {
        Expression::ArrowFunctionExpression(arrow) => {
            super::super::super::walk::extract_function_params(&arrow.params).into_vec()
        }
        Expression::FunctionExpression(func) => {
            super::super::super::walk::extract_function_params(&func.params).into_vec()
        }
        _ => Vec::new(),
    }
}

fn scan_statement_for_race(result: &ScriptParseResult, stmt: &Statement<'_>, scan: &mut RaceScan) {
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => {
            super::expression::scan_expression_for_race(result, &expr_stmt.expression, scan);
        }
        Statement::VariableDeclaration(var_decl) => {
            for decl in var_decl.declarations.iter() {
                if let Some(init) = &decl.init {
                    super::expression::scan_expression_for_race(result, init, scan);
                }
            }
        }
        Statement::ReturnStatement(ret) => {
            if let Some(arg) = &ret.argument {
                super::expression::scan_expression_for_race(result, arg, scan);
            }
        }
        Statement::BlockStatement(block) => {
            for stmt in block.body.iter() {
                scan_statement_for_race(result, stmt, scan);
            }
        }
        Statement::IfStatement(if_stmt) => {
            super::expression::scan_expression_for_race(result, &if_stmt.test, scan);
            scan_statement_for_race(result, &if_stmt.consequent, scan);
            if let Some(alt) = &if_stmt.alternate {
                scan_statement_for_race(result, alt, scan);
            }
        }
        Statement::ForStatement(for_stmt) => {
            if let Some(init) = &for_stmt.init
                && let Some(expr) = init.as_expression()
            {
                super::expression::scan_expression_for_race(result, expr, scan);
            }
            if let Some(test) = &for_stmt.test {
                super::expression::scan_expression_for_race(result, test, scan);
            }
            if let Some(update) = &for_stmt.update {
                super::expression::scan_expression_for_race(result, update, scan);
            }
            scan_statement_for_race(result, &for_stmt.body, scan);
        }
        Statement::ForInStatement(for_in) => {
            super::expression::scan_expression_for_race(result, &for_in.right, scan);
            scan_statement_for_race(result, &for_in.body, scan);
        }
        Statement::ForOfStatement(for_of) => {
            super::expression::scan_expression_for_race(result, &for_of.right, scan);
            scan_statement_for_race(result, &for_of.body, scan);
        }
        Statement::WhileStatement(while_stmt) => {
            super::expression::scan_expression_for_race(result, &while_stmt.test, scan);
            scan_statement_for_race(result, &while_stmt.body, scan);
        }
        Statement::DoWhileStatement(do_while) => {
            scan_statement_for_race(result, &do_while.body, scan);
            super::expression::scan_expression_for_race(result, &do_while.test, scan);
        }
        Statement::SwitchStatement(switch_stmt) => {
            super::expression::scan_expression_for_race(result, &switch_stmt.discriminant, scan);
            for case in switch_stmt.cases.iter() {
                if let Some(test) = &case.test {
                    super::expression::scan_expression_for_race(result, test, scan);
                }
                for stmt in case.consequent.iter() {
                    scan_statement_for_race(result, stmt, scan);
                }
            }
        }
        Statement::TryStatement(try_stmt) => {
            for stmt in try_stmt.block.body.iter() {
                scan_statement_for_race(result, stmt, scan);
            }
            if let Some(handler) = &try_stmt.handler {
                for stmt in handler.body.body.iter() {
                    scan_statement_for_race(result, stmt, scan);
                }
            }
            if let Some(finalizer) = &try_stmt.finalizer {
                for stmt in finalizer.body.iter() {
                    scan_statement_for_race(result, stmt, scan);
                }
            }
        }
        _ => {}
    }
}
