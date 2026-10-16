//! Lexical `const vm = this` evidence shared by the Vue 2 instance API rules.
//! Match eslint-plugin-vue 10.9.2's `isThis`: aliases are direct const bindings,
//! never mutable variables, chains, parameters, or same-spelled shadowed locals.

mod components;

use oxc_ast::ast::{BindingPattern, Expression, Program, VariableDeclaration};
use oxc_ast_visit::{Visit, walk::walk_variable_declaration};
use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_span::Span;
use oxc_syntax::symbol::SymbolId;
use vize_l0::FxHashSet;

pub(super) struct VueInstance {
    scoping: Scoping,
    aliases: FxHashSet<SymbolId>,
    regions: Vec<Span>,
}

impl VueInstance {
    pub(super) fn new(program: &Program<'_>, source: &str) -> Self {
        let semantic = SemanticBuilder::new().build(program).semantic;
        let mut collector = Aliases::default();
        collector.visit_program(program);
        let regions = components::regions(program, semantic.scoping(), source);
        Self {
            regions,
            scoping: semantic.into_scoping(),
            aliases: collector.0,
        }
    }

    pub(super) fn contains(&self, span: Span) -> bool {
        self.regions
            .iter()
            .any(|region| region.start <= span.start && span.end <= region.end)
    }

    pub(super) fn is_this(&self, expression: &Expression<'_>) -> bool {
        match expression {
            Expression::ThisExpression(_) => true,
            Expression::Identifier(id) => id.reference_id.get().is_some_and(|reference| {
                self.scoping
                    .get_reference(reference)
                    .symbol_id()
                    .is_some_and(|symbol| self.aliases.contains(&symbol))
            }),
            _ => false,
        }
    }
}

#[derive(Default)]
struct Aliases(FxHashSet<SymbolId>);

impl<'a> Visit<'a> for Aliases {
    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        if declaration.kind.is_const() {
            for declarator in &declaration.declarations {
                if matches!(declarator.init, Some(Expression::ThisExpression(_)))
                    && let BindingPattern::BindingIdentifier(id) = &declarator.id
                    && let Some(symbol) = id.symbol_id.get()
                {
                    self.0.insert(symbol);
                }
            }
        }
        walk_variable_declaration(self, declaration);
    }
}
