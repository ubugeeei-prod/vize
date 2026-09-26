//! Conservative runtime boolean proofs for structural JSX `&&` lowering.

use oxc_ast::ast::{
    BindingPattern, Expression, Program, UnaryOperator, VariableDeclarationKind, VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::Scoping;
use oxc_syntax::symbol::SymbolId;
use vize_l0::FxHashSet;

use super::Lowerer;

#[derive(Default)]
pub(super) struct BooleanBindings(FxHashSet<SymbolId>);

impl Lowerer<'_, '_, '_> {
    pub(crate) fn collect_boolean_bindings(&mut self, program: &Program<'_>) {
        let mut collector = Collector {
            scoping: self.scoping.as_ref(),
            bindings: &mut self.boolean_bindings,
        };
        collector.visit_program(program);
    }

    pub(super) fn is_provably_boolean(&self, expression: &Expression<'_>) -> bool {
        is_boolean(expression, self.scoping.as_ref(), &self.boolean_bindings)
    }
}

struct Collector<'a> {
    scoping: Option<&'a Scoping>,
    bindings: &'a mut BooleanBindings,
}

impl<'a> Visit<'a> for Collector<'_> {
    fn visit_variable_declarator(&mut self, declaration: &VariableDeclarator<'a>) {
        if declaration.kind == VariableDeclarationKind::Const
            && let BindingPattern::BindingIdentifier(binding) = &declaration.id
            && let Some(symbol) = binding.symbol_id.get()
            && let Some(initializer) = &declaration.init
            && is_boolean(initializer, self.scoping, self.bindings)
        {
            self.bindings.0.insert(symbol);
        }
        walk::walk_variable_declarator(self, declaration);
    }
}

fn is_boolean(
    expression: &Expression<'_>,
    scoping: Option<&Scoping>,
    bindings: &BooleanBindings,
) -> bool {
    match expression {
        Expression::ParenthesizedExpression(parenthesized) => {
            is_boolean(&parenthesized.expression, scoping, bindings)
        }
        Expression::BooleanLiteral(_) => true,
        Expression::UnaryExpression(unary) => unary.operator == UnaryOperator::LogicalNot,
        Expression::BinaryExpression(binary) => {
            binary.operator.is_equality()
                || binary.operator.is_compare()
                || binary.operator.is_relational()
        }
        Expression::LogicalExpression(logical) => {
            is_boolean(&logical.left, scoping, bindings)
                && is_boolean(&logical.right, scoping, bindings)
        }
        Expression::ConditionalExpression(conditional) => {
            is_boolean(&conditional.consequent, scoping, bindings)
                && is_boolean(&conditional.alternate, scoping, bindings)
        }
        Expression::Identifier(identifier) => identifier
            .reference_id
            .get()
            .and_then(|reference| scoping?.get_reference(reference).symbol_id())
            .is_some_and(|symbol| bindings.0.contains(&symbol)),
        // Type annotations/assertions, mutable bindings and function calls do
        // not prove a JavaScript value's type. Unknown values use a scope.
        _ => false,
    }
}
