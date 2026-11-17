//! eslint-plugin-vue's `isThis` for a `$nextTick` receiver: `this`, or an
//! identifier whose only declaration is `const name = this`.
//!
//! Declarations are recorded per lexical scope in a first pass; a second pass
//! that enters scopes in the same order resolves a receiver to the innermost
//! scope declaring its name.

use std::cell::Cell;

use oxc_ast::ast::{
    BindingIdentifier, BindingPattern, Expression, VariableDeclarationKind, VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use oxc_syntax::scope::{ScopeFlags, ScopeId};
use vize_l0::{CompactString, FxHashMap};

/// Declarations by `(scope, name)`: `true` for `const name = this`.
#[derive(Default)]
pub(super) struct ThisAliases {
    declarations: FxHashMap<(u32, CompactString), Vec<bool>>,
}

/// The lexical scopes entered so far, innermost last.
#[derive(Default)]
pub(super) struct ScopeTrail {
    open: Vec<(u32, bool)>,
    entered: u32,
}

impl ScopeTrail {
    pub(super) fn enter(&mut self, flags: ScopeFlags) {
        self.open.push((self.entered, flags.is_var()));
        self.entered += 1;
    }

    pub(super) fn leave(&mut self) {
        self.open.pop();
    }

    fn innermost(&self) -> u32 {
        self.open.last().map_or(0, |&(scope, _)| scope)
    }

    fn innermost_var(&self) -> u32 {
        self.open
            .iter()
            .rev()
            .find(|&&(_, var)| var)
            .map_or(0, |&(scope, _)| scope)
    }
}

impl ThisAliases {
    pub(super) fn collect(program: &oxc_ast::ast::Program<'_>) -> Self {
        let mut collector = Collector::default();
        collector.visit_program(program);
        collector.aliases
    }

    /// Whether `name`, read with `trail` open, resolves to a `this` alias.
    pub(super) fn is_alias(&self, trail: &ScopeTrail, name: &str) -> bool {
        trail.open.iter().rev().find_map(|&(scope, _)| {
            self.declarations
                .get(&(scope, CompactString::from(name)))
                .map(|declarations| declarations.as_slice() == [true])
        }) == Some(true)
    }

    fn declare(&mut self, scope: u32, name: &str, this_alias: bool) {
        self.declarations
            .entry((scope, CompactString::from(name)))
            .or_default()
            .push(this_alias);
    }
}

#[derive(Default)]
struct Collector {
    aliases: ThisAliases,
    trail: ScopeTrail,
}

impl<'a> Visit<'a> for Collector {
    fn enter_scope(&mut self, flags: ScopeFlags, _: &Cell<Option<ScopeId>>) {
        self.trail.enter(flags);
    }

    fn leave_scope(&mut self) {
        self.trail.leave();
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        let BindingPattern::BindingIdentifier(binding) = &declarator.id else {
            walk::walk_variable_declarator(self, declarator);
            return;
        };
        let scope = if declarator.kind == VariableDeclarationKind::Var {
            self.trail.innermost_var()
        } else {
            self.trail.innermost()
        };
        let this_alias = declarator.kind == VariableDeclarationKind::Const
            && matches!(
                declarator
                    .init
                    .as_ref()
                    .map(Expression::without_parentheses),
                Some(Expression::ThisExpression(_))
            );
        self.aliases
            .declare(scope, binding.name.as_str(), this_alias);
        if let Some(init) = &declarator.init {
            self.visit_expression(init);
        }
    }

    /// Parameters, functions, classes, catch bindings and patterns shadow.
    fn visit_binding_identifier(&mut self, binding: &BindingIdentifier<'a>) {
        let scope = self.trail.innermost();
        self.aliases.declare(scope, binding.name.as_str(), false);
    }
}
