//! Same-prop provenance through wrappers around the setup props object.

use oxc_ast::ast::{
    Argument, CallExpression, ComputedMemberExpression, Expression, StaticMemberExpression,
};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::Semantic;
use oxc_syntax::symbol::SymbolId;

pub(super) fn initializer_reads_prop(
    initializer: &Expression<'_>,
    semantic: &Semantic<'_>,
    props: &[SymbolId],
    name: &str,
) -> bool {
    let mut visitor = PropRead {
        semantic,
        props,
        name,
        found: false,
    };
    visitor.visit_expression(initializer);
    visitor.found
}

struct PropRead<'semantic, 'name> {
    semantic: &'semantic Semantic<'semantic>,
    props: &'semantic [SymbolId],
    name: &'name str,
    found: bool,
}

impl<'a> Visit<'a> for PropRead<'_, '_> {
    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if self.is_props(&member.object) {
            self.found |= member.property.name == self.name;
            return;
        }
        walk::walk_static_member_expression(self, member);
    }

    fn visit_computed_member_expression(&mut self, member: &ComputedMemberExpression<'a>) {
        if self.is_props(&member.object) {
            self.found |= member.static_property_name().as_deref() == Some(self.name);
            return;
        }
        walk::walk_computed_member_expression(self, member);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if named(call, "toRef")
            && call
                .arguments
                .first()
                .and_then(Argument::as_expression)
                .is_some_and(|argument| self.is_props(argument))
        {
            self.found |= call.arguments.get(1).is_some_and(|argument| {
                matches!(argument, Argument::StringLiteral(literal) if literal.value == self.name)
            });
            return;
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_identifier_reference(&mut self, identifier: &oxc_ast::ast::IdentifierReference<'a>) {
        self.found |= identifier
            .reference_id
            .get()
            .and_then(|reference| self.semantic.scoping().get_reference(reference).symbol_id())
            .is_some_and(|symbol| self.props.contains(&symbol));
    }
}

impl PropRead<'_, '_> {
    fn is_props(&self, expression: &Expression<'_>) -> bool {
        let Expression::Identifier(identifier) = expression.get_inner_expression() else {
            return false;
        };
        identifier
            .reference_id
            .get()
            .and_then(|reference| self.semantic.scoping().get_reference(reference).symbol_id())
            .is_some_and(|symbol| self.props.contains(&symbol))
    }
}

fn named(call: &CallExpression<'_>, name: &str) -> bool {
    !call.optional && matches!(&call.callee, Expression::Identifier(id) if id.name == name)
}
