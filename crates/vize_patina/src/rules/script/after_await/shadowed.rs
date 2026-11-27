//! Names a `setup()` function declares itself. They hide a Vue import of the
//! same name, so a call through them is not Vue's API.

use oxc_ast::ast::{
    ArrowFunctionExpression, BindingIdentifier, Class, Expression, FormalParameters, Function,
    FunctionBody,
};
use oxc_ast_visit::Visit;
use oxc_syntax::scope::ScopeFlags;
use vize_l0::{CompactString, FxHashSet};

/// Every name the parameters and body of a setup function declare, at any
/// block depth but not inside nested functions or classes. A block-scoped
/// name counts for the whole function, which can only drop a report.
pub(super) fn declared_names(
    params: &FormalParameters<'_>,
    body: &FunctionBody<'_>,
) -> FxHashSet<CompactString> {
    let mut declared = Declared::default();
    declared.visit_formal_parameters(params);
    declared.visit_function_body(body);
    declared.names
}

impl super::Scan<'_> {
    /// The tracked export `callee` names, unless a declaration of the setup
    /// function hides the import it goes through.
    pub(super) fn resolve(&self, callee: &Expression<'_>) -> Option<&'static str> {
        let through = match callee {
            Expression::Identifier(identifier) => identifier,
            Expression::StaticMemberExpression(member) => match &member.object {
                Expression::Identifier(object) => object,
                _ => return None,
            },
            _ => return None,
        };
        if self.shadowed.contains(through.name.as_str()) {
            return None;
        }
        self.imports.resolve(callee)
    }
}

#[derive(Default)]
struct Declared {
    names: FxHashSet<CompactString>,
}

impl<'a> Visit<'a> for Declared {
    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        self.names.insert(it.name.as_str().into());
    }

    // A nested function or class declares its own name here, but its
    // parameters and body belong to its own scope.
    fn visit_function(&mut self, it: &Function<'a>, _flags: ScopeFlags) {
        if let Some(id) = &it.id {
            self.visit_binding_identifier(id);
        }
    }

    fn visit_arrow_function_expression(&mut self, _it: &ArrowFunctionExpression<'a>) {}

    fn visit_class(&mut self, it: &Class<'a>) {
        if let Some(id) = &it.id {
            self.visit_binding_identifier(id);
        }
    }
}
