//! `is_constant_simple_expression`'s runtime-dependency walk.
//!
//! The shipped codegen decides "does this expression change at runtime?"
//! with `RuntimeDependencyVisitor`: every free identifier must resolve to
//! a *constant* binding, an allowed global, a runtime helper alias, or a
//! local the expression itself binds. Literals alone are not the rule —
//! `:style="theme"` over a `const theme = {…}` is constant too, which is
//! why the shipped lane skips `normalizeStyle` for it.
//!
//! The shipped visitor reads the *prefixed* content; this one reads the
//! authored S2 expression with the binding table, which answers the same
//! way: a name the prefixer would have turned into `_ctx.` / `$setup.` /
//! `__props.` is exactly a name whose binding is absent or non-constant.

use oxc_ast::ast as oxc_ast_types;
use oxc_ast_visit::{
    Visit,
    walk::{walk_arrow_function_expression, walk_function},
};
use oxc_syntax::scope::ScopeFlags;

use alloc::vec::Vec as StdVec;
use vize_s0::String;

use super::options::BindingKind;
use super::prefix::PrefixScope;
use super::prefix::globals::is_global_allowed;

/// Whether `expr` is free of runtime dependencies, as the shipped
/// codegen's `is_constant_simple_expression` decides it.
pub(super) fn is_constant_expression(
    expr: &oxc_ast_types::Expression<'_>,
    scope: &PrefixScope<'_>,
) -> bool {
    let mut visitor = RuntimeDependencyVisitor::new(scope);
    visitor.visit_expression(expr);
    !visitor.dynamic
}

/// The binding kinds `is_constant_binding` accepts.
fn is_constant_binding(kind: BindingKind) -> bool {
    matches!(
        kind,
        BindingKind::SetupConst
            | BindingKind::LiteralConst
            | BindingKind::ExternalModule
            | BindingKind::JsGlobalUniversal
            | BindingKind::JsGlobalBrowser
            | BindingKind::JsGlobalNode
            | BindingKind::JsGlobalDeno
            | BindingKind::JsGlobalBun
    )
}

/// The helper aliases the shipped visitor lets through: they are the
/// codegen's own imports, not template names.
fn is_runtime_helper_ident(name: &str) -> bool {
    matches!(
        name,
        "_unref"
            | "_normalizeClass"
            | "_normalizeStyle"
            | "_toDisplayString"
            | "_toHandlerKey"
            | "_mergeProps"
            | "_toHandlers"
            | "_guardReactiveProps"
            | "_normalizeProps"
    )
}

struct RuntimeDependencyVisitor<'s> {
    scope: &'s PrefixScope<'s>,
    locals: StdVec<StdVec<String>>,
    dynamic: bool,
}

impl<'s> RuntimeDependencyVisitor<'s> {
    fn new(scope: &'s PrefixScope<'s>) -> Self {
        Self {
            scope,
            locals: StdVec::new(),
            dynamic: false,
        }
    }

    fn is_local(&self, name: &str) -> bool {
        self.locals
            .iter()
            .rev()
            .any(|frame| frame.iter().any(|bound| bound.as_str() == name))
    }

    fn bind_pattern(&mut self, pattern: &oxc_ast_types::BindingPattern<'_>) {
        match pattern {
            oxc_ast_types::BindingPattern::BindingIdentifier(ident) => {
                if let Some(frame) = self.locals.last_mut() {
                    frame.push(String::from(ident.name.as_str()));
                }
            }
            oxc_ast_types::BindingPattern::ObjectPattern(object) => {
                for property in &object.properties {
                    self.bind_pattern(&property.value);
                }
                if let Some(rest) = &object.rest {
                    self.bind_pattern(&rest.argument);
                }
            }
            oxc_ast_types::BindingPattern::ArrayPattern(array) => {
                for element in array.elements.iter().flatten() {
                    self.bind_pattern(element);
                }
                if let Some(rest) = &array.rest {
                    self.bind_pattern(&rest.argument);
                }
            }
            oxc_ast_types::BindingPattern::AssignmentPattern(assignment) => {
                self.bind_pattern(&assignment.left);
            }
        }
    }
}

impl Visit<'_> for RuntimeDependencyVisitor<'_> {
    fn visit_identifier_reference(&mut self, ident: &oxc_ast_types::IdentifierReference<'_>) {
        if self.dynamic {
            return;
        }
        let name = ident.name.as_str();
        if self.is_local(name) || is_global_allowed(name) || is_runtime_helper_ident(name) {
            return;
        }
        if matches!(name, "_ctx" | "$setup" | "__props" | "$props") {
            self.dynamic = true;
            return;
        }
        // A template scope name (`v-for` alias, slot param) is bound by the
        // render function, not by the script: it changes per item.
        if self.scope.is_slot_param(name) || self.scope.binds_in_pattern(name) {
            self.dynamic = true;
            return;
        }
        // The shipped visitor reads the *prefixed* content and bails on
        // anything carrying `_ctx.` / `$setup.` / `__props.` / `$props.`.
        // Only an inlined render function leaves a script binding bare
        // enough for the table lookup to decide it — everywhere else the
        // prefixer has already made the name a member access, which the
        // shipped early-out reads as dynamic.
        if !self.scope.inline()
            || !self
                .scope
                .bindings()
                .and_then(|table| table.kind(name))
                .is_some_and(is_constant_binding)
        {
            self.dynamic = true;
        }
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast_types::ArrowFunctionExpression<'_>,
    ) {
        self.locals.push(StdVec::new());
        for param in &arrow.params.items {
            self.bind_pattern(&param.pattern);
        }
        walk_arrow_function_expression(self, arrow);
        self.locals.pop();
    }

    fn visit_function(&mut self, function: &oxc_ast_types::Function<'_>, flags: ScopeFlags) {
        self.locals.push(StdVec::new());
        for param in &function.params.items {
            self.bind_pattern(&param.pattern);
        }
        walk_function(self, function, flags);
        self.locals.pop();
    }

    fn visit_variable_declarator(&mut self, declarator: &oxc_ast_types::VariableDeclarator<'_>) {
        if let Some(init) = &declarator.init {
            self.visit_expression(init);
        }
        self.bind_pattern(&declarator.id);
    }
}
