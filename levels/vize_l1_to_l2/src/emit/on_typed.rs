//! TypeScript-specific `v-on` handler detection.

use oxc_ast::ast as js;
use oxc_ast_visit::{Visit, walk};

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
