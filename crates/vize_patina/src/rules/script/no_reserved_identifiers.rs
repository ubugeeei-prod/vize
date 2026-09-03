//! script/no-reserved-identifiers
//!
//! Disallow using Vue compiler reserved identifiers.
//!
//! Vue's compiler generates internal variables with specific prefixes like
//! `__props`, `__emit`, `__sfc__`, etc. Using these identifiers in your
//! code can cause conflicts and unexpected behavior.
//!
//! ## Reserved Identifiers
//!
//! - `__props` - Internal props reference
//! - `__emit` - Internal emit function
//! - `__expose` - Internal expose function
//! - `__sfc__` - SFC metadata
//! - `__sfc_main` - Main component export
//! - `_ctx` - Render context
//! - `_cache` - Render cache
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! const __props = { name: 'test' }
//! const __emit = () => {}
//! let __sfc__ = {}
//! ```
//!
//! ### Valid
//! ```ts
//! const props = defineProps<Props>()
//! const emit = defineEmits<Emits>()
//! const myData = {}
//! ```

use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    AssignmentExpression, AssignmentTarget, BindingIdentifier, BindingPattern, Function, Program,
    VariableDeclarator,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_assignment_expression, walk_function, walk_variable_declarator},
};
use oxc_span::Span;
use oxc_syntax::scope::ScopeFlags;

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-reserved-identifiers",
    description: "Disallow using Vue compiler reserved identifiers",
    default_severity: Severity::Error,
};

/// Reserved identifiers used by Vue compiler
const RESERVED_IDENTIFIERS: &[&str] = &[
    "__props",
    "__emit",
    "__expose",
    "__sfc__",
    "__sfc_main",
    "__injectCSSVars__",
    "_ctx",
    "_cache",
    "_setupState",
    "_hoisted_",
    "_createBlock",
    "_createVNode",
    "_createElementVNode",
    "_resolveComponent",
    "_resolveDirective",
    "_withCtx",
    "_openBlock",
];

/// No reserved identifiers rule
pub struct NoReservedIdentifiers;

impl ScriptRule for NoReservedIdentifiers {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    #[inline]
    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let mut visitor = NoReservedIdentifiersVisitor { offset, result };
        visitor.visit_program(program);
    }
}

struct NoReservedIdentifiersVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for NoReservedIdentifiersVisitor<'_> {
    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        self.check_binding_pattern(&it.id);
        walk_variable_declarator(self, it);
    }

    fn visit_function(&mut self, it: &Function<'a>, flags: ScopeFlags) {
        if let Some(id) = &it.id {
            self.check_binding_identifier(id);
        }
        walk_function(self, it, flags);
    }

    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        if let AssignmentTarget::AssignmentTargetIdentifier(identifier) = &it.left {
            self.check_identifier(identifier.name.as_str(), identifier.span);
        }
        walk_assignment_expression(self, it);
    }
}

impl NoReservedIdentifiersVisitor<'_> {
    fn check_binding_pattern(&mut self, pattern: &BindingPattern<'_>) {
        match pattern {
            BindingPattern::BindingIdentifier(identifier) => {
                self.check_binding_identifier(identifier);
            }
            BindingPattern::ObjectPattern(object) => {
                for property in &object.properties {
                    self.check_binding_pattern(&property.value);
                }
                if let Some(rest) = &object.rest {
                    self.check_binding_pattern(&rest.argument);
                }
            }
            BindingPattern::ArrayPattern(array) => {
                for element in array.elements.iter().flatten() {
                    self.check_binding_pattern(element);
                }
                if let Some(rest) = &array.rest {
                    self.check_binding_pattern(&rest.argument);
                }
            }
            BindingPattern::AssignmentPattern(assignment) => {
                self.check_binding_pattern(&assignment.left);
            }
        }
    }

    fn check_binding_identifier(&mut self, identifier: &BindingIdentifier<'_>) {
        self.check_identifier(identifier.name.as_str(), identifier.span);
    }

    fn check_identifier(&mut self, name: &str, span: Span) {
        if is_reserved_identifier(name) {
            let start = self.offset as u32 + span.start;
            let end = self.offset as u32 + span.end;
            self.result.add_diagnostic(
                LintDiagnostic::error(
                    META.name,
                    "Vue compiler reserved identifier should not be used",
                    start,
                    end,
                )
                .with_help(
                    "Choose a different variable name to avoid conflicts with Vue internals",
                ),
            );
        }
    }
}

fn is_reserved_identifier(name: &str) -> bool {
    RESERVED_IDENTIFIERS.contains(&name) || name.starts_with("_hoisted_")
}

#[cfg(test)]
mod tests {
    use super::NoReservedIdentifiers;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoReservedIdentifiers));
        linter
    }

    #[test]
    fn test_valid_normal_identifier() {
        let linter = create_linter();
        let result = linter.lint("const props = defineProps()", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_private_instance_identifier() {
        let linter = create_linter();
        let result = linter.lint("const _instance = getCurrentInstance()", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_reserved_props() {
        let linter = create_linter();
        let result = linter.lint("const __props = {}", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_reserved_emit() {
        let linter = create_linter();
        let result = linter.lint("let __emit = () => {}", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_reserved_sfc() {
        let linter = create_linter();
        let result = linter.lint("var __sfc__ = {}", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_reserved_hoisted_prefix() {
        let linter = create_linter();
        let result = linter.lint("const _hoisted_1 = {}", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_reserved_function_name() {
        let linter = create_linter();
        let result = linter.lint("function _openBlock() {}", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_reserved_assignment() {
        let linter = create_linter();
        let result = linter.lint("__props = {}", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_reserved_identifier_string_not_matched() {
        let linter = create_linter();
        let result = linter.lint(r#"const text = "const __props = {}""#, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_reserved_member_assignment_not_matched() {
        let linter = create_linter();
        let result = linter.lint("state.__props = {}", 0);
        assert_eq!(result.error_count, 0);
    }
}
