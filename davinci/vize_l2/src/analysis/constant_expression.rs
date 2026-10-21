//! Constness of retained expressions under the self-bound rule.
//!
//! This is an L2 property, independent of the backend decision that consumes
//! it. A retained JS expression is constant when every identifier reference
//! names a local introduced by that expression. Free names, `this`, and
//! TypeScript-only syntax are deliberately refused. Opaque and foreign
//! expressions follow L2's pessimal law and are never constant.

use alloc::vec::Vec as StdVec;

use oxc_ast::ast as js;
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk::{walk_arrow_function_expression, walk_binding_pattern, walk_function};
use oxc_syntax::scope::ScopeFlags;

use crate::expr::ExprRef;

/// Return whether `expr` is constant under the backend-neutral self-bound
/// rule.
#[must_use]
pub fn is_self_bound_constant(expr: &ExprRef<'_>) -> bool {
    match expr {
        ExprRef::Opaque(opaque) => opaque.is_constant(),
        ExprRef::Foreign(_) | ExprRef::Filter(_) => false,
        ExprRef::Js(retained) => is_self_bound_js_constant(retained.ast, retained.source),
    }
}

/// Return whether a retained JS expression is constant under the self-bound
/// rule.
#[must_use]
pub fn is_self_bound_js_constant(expr: &js::Expression<'_>, source: &str) -> bool {
    if source.contains("_ctx.")
        || source.contains("$setup.")
        || source.contains("__props.")
        || source.contains("$props.")
    {
        return false;
    }
    let mut walk = ConstWalk {
        locals: StdVec::new(),
        dynamic: false,
    };
    walk.visit_expression(expr);
    !walk.dynamic
}

struct ConstWalk {
    locals: StdVec<StdVec<vize_l0::String>>,
    dynamic: bool,
}

impl ConstWalk {
    fn is_local(&self, name: &str) -> bool {
        self.locals
            .iter()
            .any(|frame| frame.iter().any(|bound| bound.as_str() == name))
    }

    fn bind_pattern(&mut self, pattern: &js::BindingPattern<'_>) {
        match pattern {
            js::BindingPattern::BindingIdentifier(ident) => {
                if let Some(frame) = self.locals.last_mut() {
                    frame.push(vize_l0::String::from(ident.name.as_str()));
                }
            }
            js::BindingPattern::ObjectPattern(object) => {
                for property in &object.properties {
                    self.bind_pattern(&property.value);
                }
                if let Some(rest) = &object.rest {
                    self.bind_pattern(&rest.argument);
                }
            }
            js::BindingPattern::ArrayPattern(array) => {
                for element in array.elements.iter().flatten() {
                    self.bind_pattern(element);
                }
                if let Some(rest) = &array.rest {
                    self.bind_pattern(&rest.argument);
                }
            }
            js::BindingPattern::AssignmentPattern(assignment) => {
                self.bind_pattern(&assignment.left);
            }
        }
    }
}

impl<'a> Visit<'a> for ConstWalk {
    fn visit_identifier_reference(&mut self, ident: &js::IdentifierReference<'a>) {
        if !self.is_local(ident.name.as_str()) {
            self.dynamic = true;
        }
    }

    fn visit_this_expression(&mut self, _this: &js::ThisExpression) {
        self.dynamic = true;
    }

    fn visit_arrow_function_expression(&mut self, arrow: &js::ArrowFunctionExpression<'a>) {
        self.locals.push(StdVec::new());
        for param in &arrow.params.items {
            self.bind_pattern(&param.pattern);
        }
        walk_arrow_function_expression(self, arrow);
        self.locals.pop();
    }

    fn visit_function(&mut self, function: &js::Function<'a>, flags: ScopeFlags) {
        self.locals.push(StdVec::new());
        for param in &function.params.items {
            self.bind_pattern(&param.pattern);
        }
        walk_function(self, function, flags);
        self.locals.pop();
    }

    fn visit_variable_declarator(&mut self, declarator: &js::VariableDeclarator<'a>) {
        if let Some(init) = &declarator.init {
            self.visit_expression(init);
        }
        self.bind_pattern(&declarator.id);
        walk_binding_pattern(self, &declarator.id);
    }

    fn visit_ts_as_expression(&mut self, _expr: &js::TSAsExpression<'a>) {
        self.dynamic = true;
    }

    fn visit_ts_satisfies_expression(&mut self, _expr: &js::TSSatisfiesExpression<'a>) {
        self.dynamic = true;
    }

    fn visit_ts_type_assertion(&mut self, _expr: &js::TSTypeAssertion<'a>) {
        self.dynamic = true;
    }

    fn visit_ts_non_null_expression(&mut self, _expr: &js::TSNonNullExpression<'a>) {
        self.dynamic = true;
    }

    fn visit_ts_instantiation_expression(&mut self, _expr: &js::TSInstantiationExpression<'a>) {
        self.dynamic = true;
    }
}

#[cfg(test)]
mod tests {
    use vize_l0::{Allocator, Span};

    use super::is_self_bound_constant;
    use crate::expr::ExprRef;

    fn classify(source: &str) -> bool {
        let allocator = Allocator::new();
        let end = u32::try_from(source.len()).expect("test expression length fits u32");
        let expr = ExprRef::parse_js_in(&allocator, source, Span::new(0, end));
        is_self_bound_constant(&expr)
    }

    #[test]
    fn accepts_literals_and_locals_bound_by_the_expression() {
        assert!(classify("42"));
        assert!(classify("(value) => value.toFixed(2)"));
    }

    #[test]
    fn refuses_dynamic_or_unclassified_inputs() {
        for source in ["Math.PI", "this.value", "100 as any", "'_ctx.'", "a b"] {
            assert!(!classify(source), "{source} must not be constant");
        }
    }
}
