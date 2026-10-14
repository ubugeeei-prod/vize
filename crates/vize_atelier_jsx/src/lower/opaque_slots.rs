//! Slot objects that are forwarded verbatim instead of expanded.
//!
//! `@vue/babel-plugin-jsx` copies a slots object's entries as written, so
//! `{{ default: slots.icon }}` hands the component the function `slots.icon`.
//! Vize expands an entry into a slot template only when it is an inline
//! function, and cannot see inside a spread or a function reference. When such
//! an object contains no JSX and no TypeScript-only syntax, its source is
//! already the JavaScript Babel would emit, so it is forwarded as an opaque
//! slots object instead of having those entries dropped.

use oxc_ast::ast::{
    Expression, JSXElement, JSXFragment, ObjectExpression, ObjectPropertyKind, TSAsExpression,
    TSNonNullExpression, TSSatisfiesExpression, TSTypeAnnotation, TSTypeAssertion,
    TSTypeParameterInstantiation,
};
use oxc_ast_visit::{Visit, walk};

/// Whether `object` should be forwarded verbatim rather than expanded.
pub(crate) fn forwards_verbatim(object: &ObjectExpression<'_>) -> bool {
    let opaque = object.properties.iter().any(|property| match property {
        ObjectPropertyKind::SpreadProperty(_) => true,
        ObjectPropertyKind::ObjectProperty(property) => !matches!(
            property.value.get_inner_expression(),
            Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
        ),
    });
    if !opaque {
        return false;
    }
    let mut finder = NotPlainJs::default();
    finder.visit_object_expression(object);
    !finder.found
}

/// Finds syntax whose source is not plain JavaScript: JSX, or TS-only syntax.
#[derive(Default)]
struct NotPlainJs {
    found: bool,
}

impl<'a> Visit<'a> for NotPlainJs {
    fn visit_jsx_element(&mut self, _: &JSXElement<'a>) {
        self.found = true;
    }
    fn visit_jsx_fragment(&mut self, _: &JSXFragment<'a>) {
        self.found = true;
    }
    fn visit_ts_as_expression(&mut self, _: &TSAsExpression<'a>) {
        self.found = true;
    }
    fn visit_ts_satisfies_expression(&mut self, _: &TSSatisfiesExpression<'a>) {
        self.found = true;
    }
    fn visit_ts_non_null_expression(&mut self, _: &TSNonNullExpression<'a>) {
        self.found = true;
    }
    fn visit_ts_type_assertion(&mut self, _: &TSTypeAssertion<'a>) {
        self.found = true;
    }
    fn visit_ts_type_annotation(&mut self, _: &TSTypeAnnotation<'a>) {
        self.found = true;
    }
    fn visit_ts_type_parameter_instantiation(&mut self, _: &TSTypeParameterInstantiation<'a>) {
        self.found = true;
    }
    fn visit_expression(&mut self, expression: &Expression<'a>) {
        if !self.found {
            walk::walk_expression(self, expression);
        }
    }
}
