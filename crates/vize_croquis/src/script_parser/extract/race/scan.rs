use oxc_ast::ast::{BindingPattern, Expression, Statement};

use vize_carton::{CompactString, FxHashMap, FxHashSet, cstr};

use super::super::super::ScriptParseResult;
use crate::BindingType;

#[derive(Default)]
pub(super) struct RaceScan {
    pub(super) async_operations: Vec<CompactString>,
    pub(super) mutated_targets: FxHashSet<CompactString>,
    pub(super) cleanup_names: FxHashSet<CompactString>,
    pub(super) has_cleanup_call: bool,
    /// Locals copied from a reactive read, keyed by the local name.
    pub(super) snapshots: FxHashMap<CompactString, Snapshot>,
    /// How many `await`s the callback has already passed.
    pub(super) async_generation: u32,
    /// A compare-after-await guard dominates the statements being scanned.
    pub(super) guarded: bool,
}

pub(super) struct Snapshot {
    pub(super) live: CompactString,
    pub(super) generation: u32,
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
                    if let Some(name) = binding_identifier(&decl.id)
                        && let Some(live) = reactive_read_label(result, init)
                    {
                        let generation = scan.async_generation;
                        scan.snapshots.insert(name, Snapshot { live, generation });
                    }
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
            if let Some(guard) = freshness_guard(result, scan, &if_stmt.test) {
                match guard {
                    FreshnessGuard::RejectStale if statement_exits(&if_stmt.consequent) => {
                        scan_statement_for_race(result, &if_stmt.consequent, scan);
                        scan.guarded = true;
                        if let Some(alternate) = &if_stmt.alternate {
                            scan_statement_for_race(result, alternate, scan);
                        }
                    }
                    FreshnessGuard::AcceptFresh => {
                        let previous = scan.guarded;
                        scan.guarded = true;
                        scan_statement_for_race(result, &if_stmt.consequent, scan);
                        scan.guarded = previous;
                        if let Some(alternate) = &if_stmt.alternate {
                            scan_statement_for_race(result, alternate, scan);
                        }
                    }
                    FreshnessGuard::RejectStale => {
                        scan_statement_for_race(result, &if_stmt.consequent, scan);
                        if let Some(alternate) = &if_stmt.alternate {
                            scan_statement_for_race(result, alternate, scan);
                        }
                    }
                }
            } else {
                scan_statement_for_race(result, &if_stmt.consequent, scan);
                if let Some(alternate) = &if_stmt.alternate {
                    scan_statement_for_race(result, alternate, scan);
                }
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

enum FreshnessGuard {
    /// `if (requested !== query) return` — later writes see the fresh value.
    RejectStale,
    /// `if (requested === query) write` — only the consequent is fresh.
    AcceptFresh,
}

fn freshness_guard(
    result: &ScriptParseResult,
    scan: &RaceScan,
    test: &Expression<'_>,
) -> Option<FreshnessGuard> {
    let test = peel_expression(test);
    let Expression::BinaryExpression(binary) = test else {
        return None;
    };
    let equal = match binary.operator {
        oxc_ast::ast::BinaryOperator::StrictEquality | oxc_ast::ast::BinaryOperator::Equality => {
            true
        }
        oxc_ast::ast::BinaryOperator::StrictInequality
        | oxc_ast::ast::BinaryOperator::Inequality => false,
        _ => return None,
    };
    let (snapshot_name, live_expr) = snapshot_operand(scan, &binary.left, &binary.right)?;
    let snapshot = scan.snapshots.get(snapshot_name)?;
    if snapshot.generation >= scan.async_generation {
        return None;
    }
    let live = reactive_read_label(result, live_expr)?;
    (live == snapshot.live).then_some(if equal {
        FreshnessGuard::AcceptFresh
    } else {
        FreshnessGuard::RejectStale
    })
}

fn snapshot_operand<'a>(
    scan: &RaceScan,
    left: &'a Expression<'a>,
    right: &'a Expression<'a>,
) -> Option<(&'a str, &'a Expression<'a>)> {
    let left = peel_expression(left);
    let right = peel_expression(right);
    if let Some(name) = identifier_name(left)
        && scan.snapshots.contains_key(name)
    {
        return Some((name, right));
    }
    if let Some(name) = identifier_name(right)
        && scan.snapshots.contains_key(name)
    {
        return Some((name, left));
    }
    None
}

fn binding_identifier(pattern: &BindingPattern<'_>) -> Option<CompactString> {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => {
            Some(CompactString::new(identifier.name.as_str()))
        }
        _ => None,
    }
}

fn reactive_read_label(result: &ScriptParseResult, expr: &Expression<'_>) -> Option<CompactString> {
    match peel_expression(expr) {
        Expression::Identifier(identifier) if is_live_source(result, identifier.name.as_str()) => {
            Some(CompactString::new(identifier.name.as_str()))
        }
        Expression::StaticMemberExpression(member)
            if member.property.name.as_str() == "value"
                && let Expression::Identifier(identifier) = peel_expression(&member.object)
                && result
                    .reactivity
                    .needs_value_access(identifier.name.as_str()) =>
        {
            Some(cstr!("{}.value", identifier.name))
        }
        _ => None,
    }
}

fn is_live_source(result: &ScriptParseResult, name: &str) -> bool {
    result.reactivity.is_reactive(name)
        || matches!(
            result.bindings.get(name),
            Some(BindingType::Props | BindingType::PropsAliased)
        )
}

fn identifier_name<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    match peel_expression(expr) {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn peel_expression<'a>(expr: &'a Expression<'a>) -> &'a Expression<'a> {
    match expr {
        Expression::ParenthesizedExpression(paren) => peel_expression(&paren.expression),
        Expression::TSAsExpression(ts_as) => peel_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            peel_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => peel_expression(&ts_non_null.expression),
        _ => expr,
    }
}

fn statement_exits(statement: &Statement<'_>) -> bool {
    match statement {
        Statement::ReturnStatement(_) | Statement::ThrowStatement(_) => true,
        Statement::BlockStatement(block) => block.body.last().is_some_and(statement_exits),
        _ => false,
    }
}
