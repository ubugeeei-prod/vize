//! TypeScript-specific `v-on` handler detection.

use oxc_ast::ast as js;
use oxc_ast_visit::{Visit, walk};

pub(super) fn legacy_raw_non_null_call(expr: &js::Expression<'_>) -> bool {
    let Some(call) = root_call(expr) else {
        return false;
    };
    call.type_arguments.is_none()
        && callee_has_non_null(&call.callee)
        && !callee_has_disallowed_ts(&call.callee)
        && call
            .arguments
            .iter()
            .all(|argument| !argument.as_expression().is_some_and(uses_ts_only_syntax))
}

pub(super) fn legacy_raw_non_null_assignment(expr: &js::Expression<'_>) -> bool {
    let Some(assignment) = root_assignment(expr) else {
        return false;
    };
    assignment_target_has_non_null(&assignment.left)
        && !assignment_target_has_disallowed_ts(&assignment.left)
        && !uses_ts_only_syntax(&assignment.right)
}

pub(super) fn uses_ts_only_syntax(expr: &js::Expression<'_>) -> bool {
    if is_typed_arrow(expr) {
        return true;
    }
    let mut scan = TsOnlySyntaxScan { seen: false };
    scan.visit_expression(expr);
    scan.seen
}

fn is_typed_arrow(expr: &js::Expression<'_>) -> bool {
    let js::Expression::ArrowFunctionExpression(arrow) = expr else {
        return false;
    };
    arrow_has_ts_types(arrow)
}

fn arrow_has_ts_types(arrow: &js::ArrowFunctionExpression<'_>) -> bool {
    arrow.type_parameters.is_some()
        || arrow.return_type.is_some()
        || arrow
            .params
            .items
            .iter()
            .any(|param| param.type_annotation.is_some())
        || arrow
            .params
            .rest
            .as_ref()
            .is_some_and(|rest| rest.type_annotation.is_some())
}

fn root_call<'a>(expr: &'a js::Expression<'a>) -> Option<&'a js::CallExpression<'a>> {
    match expr {
        js::Expression::CallExpression(call) => Some(call),
        js::Expression::ChainExpression(chain) => match &chain.expression {
            js::ChainElement::CallExpression(call) => Some(call),
            _ => None,
        },
        js::Expression::ParenthesizedExpression(paren) => root_call(&paren.expression),
        _ => None,
    }
}

fn root_assignment<'a>(expr: &'a js::Expression<'a>) -> Option<&'a js::AssignmentExpression<'a>> {
    match expr {
        js::Expression::AssignmentExpression(assignment) => Some(assignment),
        js::Expression::ParenthesizedExpression(paren) => root_assignment(&paren.expression),
        _ => None,
    }
}

fn callee_has_non_null(expr: &js::Expression<'_>) -> bool {
    match expr {
        js::Expression::TSNonNullExpression(_) => true,
        js::Expression::ParenthesizedExpression(paren) => callee_has_non_null(&paren.expression),
        js::Expression::StaticMemberExpression(member) => callee_has_non_null(&member.object),
        js::Expression::ComputedMemberExpression(member) => callee_has_non_null(&member.object),
        js::Expression::PrivateFieldExpression(member) => callee_has_non_null(&member.object),
        js::Expression::ChainExpression(chain) => chain_callee_has_non_null(&chain.expression),
        js::Expression::CallExpression(call) => callee_has_non_null(&call.callee),
        _ => false,
    }
}

fn assignment_target_has_non_null(target: &js::AssignmentTarget<'_>) -> bool {
    match target {
        js::AssignmentTarget::TSNonNullExpression(_) => true,
        js::AssignmentTarget::StaticMemberExpression(member) => callee_has_non_null(&member.object),
        js::AssignmentTarget::ComputedMemberExpression(member) => {
            callee_has_non_null(&member.object)
        }
        js::AssignmentTarget::PrivateFieldExpression(member) => callee_has_non_null(&member.object),
        _ => false,
    }
}

fn chain_callee_has_non_null(element: &js::ChainElement<'_>) -> bool {
    match element {
        js::ChainElement::TSNonNullExpression(_) => true,
        js::ChainElement::CallExpression(call) => callee_has_non_null(&call.callee),
        js::ChainElement::StaticMemberExpression(member) => callee_has_non_null(&member.object),
        js::ChainElement::ComputedMemberExpression(member) => callee_has_non_null(&member.object),
        js::ChainElement::PrivateFieldExpression(member) => callee_has_non_null(&member.object),
    }
}

fn assignment_target_has_disallowed_ts(target: &js::AssignmentTarget<'_>) -> bool {
    match target {
        js::AssignmentTarget::TSAsExpression(_)
        | js::AssignmentTarget::TSSatisfiesExpression(_)
        | js::AssignmentTarget::TSTypeAssertion(_) => true,
        js::AssignmentTarget::TSNonNullExpression(ts_non_null) => {
            callee_has_disallowed_ts(&ts_non_null.expression)
        }
        js::AssignmentTarget::StaticMemberExpression(member) => {
            callee_has_disallowed_ts(&member.object)
        }
        js::AssignmentTarget::ComputedMemberExpression(member) => {
            callee_has_disallowed_ts(&member.object) || uses_ts_only_syntax(&member.expression)
        }
        js::AssignmentTarget::PrivateFieldExpression(member) => {
            callee_has_disallowed_ts(&member.object)
        }
        _ => false,
    }
}

fn callee_has_disallowed_ts(expr: &js::Expression<'_>) -> bool {
    match expr {
        js::Expression::TSAsExpression(_)
        | js::Expression::TSSatisfiesExpression(_)
        | js::Expression::TSTypeAssertion(_)
        | js::Expression::TSInstantiationExpression(_) => true,
        js::Expression::TSNonNullExpression(ts_non_null) => {
            callee_has_disallowed_ts(&ts_non_null.expression)
        }
        js::Expression::ParenthesizedExpression(paren) => {
            callee_has_disallowed_ts(&paren.expression)
        }
        js::Expression::StaticMemberExpression(member) => callee_has_disallowed_ts(&member.object),
        js::Expression::ComputedMemberExpression(member) => {
            callee_has_disallowed_ts(&member.object) || uses_ts_only_syntax(&member.expression)
        }
        js::Expression::PrivateFieldExpression(member) => callee_has_disallowed_ts(&member.object),
        js::Expression::ChainExpression(chain) => chain_callee_has_disallowed_ts(&chain.expression),
        js::Expression::CallExpression(call) => {
            call.type_arguments.is_some()
                || callee_has_disallowed_ts(&call.callee)
                || call
                    .arguments
                    .iter()
                    .any(|argument| argument.as_expression().is_some_and(uses_ts_only_syntax))
        }
        _ => false,
    }
}

fn chain_callee_has_disallowed_ts(element: &js::ChainElement<'_>) -> bool {
    match element {
        js::ChainElement::TSNonNullExpression(ts_non_null) => {
            callee_has_disallowed_ts(&ts_non_null.expression)
        }
        js::ChainElement::CallExpression(call) => {
            call.type_arguments.is_some()
                || callee_has_disallowed_ts(&call.callee)
                || call
                    .arguments
                    .iter()
                    .any(|argument| argument.as_expression().is_some_and(uses_ts_only_syntax))
        }
        js::ChainElement::StaticMemberExpression(member) => {
            callee_has_disallowed_ts(&member.object)
        }
        js::ChainElement::ComputedMemberExpression(member) => {
            callee_has_disallowed_ts(&member.object) || uses_ts_only_syntax(&member.expression)
        }
        js::ChainElement::PrivateFieldExpression(member) => {
            callee_has_disallowed_ts(&member.object)
        }
    }
}

struct TsOnlySyntaxScan {
    seen: bool,
}

impl<'a> Visit<'a> for TsOnlySyntaxScan {
    fn visit_arrow_function_expression(&mut self, arrow: &js::ArrowFunctionExpression<'a>) {
        if arrow_has_ts_types(arrow) {
            self.seen = true;
            return;
        }
        walk::walk_arrow_function_expression(self, arrow);
    }

    fn visit_ts_as_expression(&mut self, _expr: &js::TSAsExpression<'a>) {
        self.seen = true;
    }

    fn visit_ts_satisfies_expression(&mut self, _expr: &js::TSSatisfiesExpression<'a>) {
        self.seen = true;
    }

    fn visit_ts_type_assertion(&mut self, _expr: &js::TSTypeAssertion<'a>) {
        self.seen = true;
    }

    fn visit_ts_non_null_expression(&mut self, _expr: &js::TSNonNullExpression<'a>) {
        self.seen = true;
    }

    fn visit_ts_instantiation_expression(&mut self, _expr: &js::TSInstantiationExpression<'a>) {
        self.seen = true;
    }
}
