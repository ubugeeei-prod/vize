//! Vue's const, macro, nested, and rest rules for binding patterns.

use super::classify::{call, destructures_props, is_const_macro_call};
use super::{Bindings, SetupBindingKind};
use oxc_ast::ast::{BindingPattern, VariableDeclarationKind, VariableDeclarator};

impl<'a> Bindings<'a> {
    /// Walk a destructuring pattern with Vue's const/macro/rest rules.
    pub(super) fn pattern(
        &mut self,
        base: u32,
        kind: VariableDeclarationKind,
        declarator: &VariableDeclarator<'a>,
    ) {
        let init = declarator.init.as_ref();
        let is_const = kind == VariableDeclarationKind::Const;
        let is_const_macro =
            self.script_setup && is_const && init.and_then(call).is_some_and(is_const_macro_call);
        match &declarator.id {
            BindingPattern::ObjectPattern(pattern)
                if self.script_setup && init.is_some_and(destructures_props) =>
            {
                self.props_destructure(base, pattern);
            }
            BindingPattern::ObjectPattern(pattern) => {
                self.object_pattern(base, pattern, is_const, is_const_macro);
            }
            BindingPattern::ArrayPattern(pattern) => {
                self.array_pattern(base, pattern, is_const, is_const_macro);
            }
            BindingPattern::AssignmentPattern(pattern) => {
                self.binding_pattern(base, &pattern.left, is_const, is_const_macro);
            }
            // `declarator` handles this before dispatching to `pattern`.
            BindingPattern::BindingIdentifier(_) => {}
        }
    }

    fn object_pattern(
        &mut self,
        base: u32,
        pattern: &oxc_ast::ast::ObjectPattern<'a>,
        is_const: bool,
        is_const_macro: bool,
    ) {
        for property in &pattern.properties {
            self.binding_pattern(base, &property.value, is_const, is_const_macro);
        }
        if let Some(rest) = &pattern.rest {
            self.rest_pattern(base, &rest.argument, is_const);
        }
    }

    fn array_pattern(
        &mut self,
        base: u32,
        pattern: &oxc_ast::ast::ArrayPattern<'a>,
        is_const: bool,
        is_const_macro: bool,
    ) {
        for element in pattern.elements.iter().flatten() {
            self.binding_pattern(base, element, is_const, is_const_macro);
        }
        if let Some(rest) = &pattern.rest {
            self.rest_pattern(base, &rest.argument, is_const);
        }
    }

    fn binding_pattern(
        &mut self,
        base: u32,
        pattern: &BindingPattern<'a>,
        is_const: bool,
        is_const_macro: bool,
    ) {
        match pattern {
            BindingPattern::BindingIdentifier(id) => {
                let kind = if is_const_macro {
                    SetupBindingKind::SetupConst
                } else if is_const {
                    SetupBindingKind::SetupMaybeRef
                } else {
                    SetupBindingKind::SetupLet
                };
                self.push(base, id.span, kind);
            }
            // Vue intentionally drops the macro flag once a pattern nests.
            BindingPattern::ObjectPattern(pattern) => {
                self.object_pattern(base, pattern, is_const, false);
            }
            BindingPattern::ArrayPattern(pattern) => {
                self.array_pattern(base, pattern, is_const, false);
            }
            BindingPattern::AssignmentPattern(pattern) => {
                self.binding_pattern(base, &pattern.left, is_const, is_const_macro);
            }
        }
    }

    fn rest_pattern(&mut self, base: u32, pattern: &BindingPattern<'a>, is_const: bool) {
        let kind = if is_const {
            SetupBindingKind::SetupConst
        } else {
            SetupBindingKind::SetupLet
        };
        pattern.all_binding_identifiers(&mut |id| {
            self.push(base, id.span, kind);
            true
        });
    }
}
