//! script/no-deprecated-delete-set
//!
//! Vue 3 removed `Vue.set`, `Vue.delete`, `this.$set`, and `this.$delete`.
//! Reactive objects are updated by assignment.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   created() {
//!     this.$set(this.item, 'ready', true)
//!   }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   created() {
//!     this.item.ready = true
//!   }
//! }
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    BindingPattern, Expression, ImportDeclarationSpecifier, Program, VariableDeclarator,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_call_expression, walk_import_declaration, walk_static_member_expression,
        walk_variable_declarator,
    },
};
use oxc_span::Span;
use vize_l0::{CompactString, FxHashSet};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-deprecated-delete-set",
    description: "Disallow Vue 2 $set and $delete, which were removed in Vue 3",
    default_severity: Severity::Error,
};

const MESSAGE: &str = "`$set` and `$delete` were removed in Vue 3.";

pub struct NoDeprecatedDeleteSet;

impl ScriptRule for NoDeprecatedDeleteSet {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        if !(source.contains("$set")
            || source.contains("$delete")
            || source.contains("set")
            || source.contains("delete")
            || source.contains("del"))
        {
            return;
        }
        let mut visitor = DeleteSetVisitor {
            offset,
            result,
            vue_modules: FxHashSet::default(),
            callables: FxHashSet::default(),
        };
        visitor.visit_program(program);
    }
}

struct DeleteSetVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
    vue_modules: FxHashSet<CompactString>,
    callables: FxHashSet<CompactString>,
}

impl<'a> Visit<'a> for DeleteSetVisitor<'_> {
    fn visit_import_declaration(&mut self, it: &oxc_ast::ast::ImportDeclaration<'a>) {
        if it.source.value.as_str() == "vue"
            && let Some(specifiers) = &it.specifiers
        {
            for specifier in specifiers {
                match specifier {
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        self.vue_modules
                            .insert(CompactString::new(specifier.local.name.as_str()));
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        self.vue_modules
                            .insert(CompactString::new(specifier.local.name.as_str()));
                    }
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        if matches!(specifier.imported.name().as_str(), "set" | "del") {
                            self.callables
                                .insert(CompactString::new(specifier.local.name.as_str()));
                        }
                    }
                }
            }
        }
        walk_import_declaration(self, it);
    }

    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        if let Some(init) = &it.init
            && is_vue_require(init)
        {
            if let BindingPattern::BindingIdentifier(identifier) = &it.id {
                self.vue_modules
                    .insert(CompactString::new(identifier.name.as_str()));
            } else {
                self.bind_require_destructure(&it.id);
            }
        }
        walk_variable_declarator(self, it);
    }

    fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
        if let Expression::Identifier(identifier) = call.callee.get_inner_expression()
            && self.callables.contains(identifier.name.as_str())
        {
            self.report(identifier.span);
        }
        walk_call_expression(self, call);
    }

    fn visit_static_member_expression(&mut self, it: &oxc_ast::ast::StaticMemberExpression<'a>) {
        let name = it.property.name.as_str();
        let flagged = match name {
            "$set" | "$delete" => is_this(&it.object),
            "set" | "delete" => self.is_vue_receiver(&it.object),
            _ => false,
        };
        if flagged {
            self.report(it.span);
        }
        walk_static_member_expression(self, it);
    }
}

impl DeleteSetVisitor<'_> {
    fn bind_require_destructure(&mut self, pattern: &BindingPattern<'_>) {
        let BindingPattern::ObjectPattern(object) = pattern else {
            return;
        };
        for property in &object.properties {
            let Some(key) = binding_key(&property.key) else {
                continue;
            };
            if !matches!(key, "set" | "del") {
                continue;
            }
            if let Some(name) = simple_name(&property.value) {
                self.callables.insert(CompactString::new(name));
            }
        }
    }

    fn is_vue_receiver(&self, expression: &Expression<'_>) -> bool {
        match expression.get_inner_expression() {
            Expression::Identifier(identifier) => {
                identifier.name.as_str() == "Vue"
                    || self.vue_modules.contains(identifier.name.as_str())
            }
            Expression::CallExpression(call) => is_vue_require_call(call),
            Expression::ChainExpression(chain) => chain
                .expression
                .as_member_expression()
                .is_some_and(|member| self.is_vue_receiver(member.object())),
            _ => false,
        }
    }

    fn report(&mut self, span: Span) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        self.result
            .add_diagnostic(LintDiagnostic::error(META.name, MESSAGE, start, end));
    }
}

fn is_this(expression: &Expression<'_>) -> bool {
    matches!(
        expression.get_inner_expression(),
        Expression::ThisExpression(_)
    )
}

fn is_vue_require(expression: &Expression<'_>) -> bool {
    match expression.get_inner_expression() {
        Expression::CallExpression(call) => is_vue_require_call(call),
        _ => false,
    }
}

fn is_vue_require_call(call: &oxc_ast::ast::CallExpression<'_>) -> bool {
    let Expression::Identifier(callee) = call.callee.get_inner_expression() else {
        return false;
    };
    if callee.name.as_str() != "require" {
        return false;
    }
    matches!(
        call.arguments.first(),
        Some(oxc_ast::ast::Argument::StringLiteral(literal)) if literal.value.as_str() == "vue"
    )
}

fn binding_key<'a>(key: &'a oxc_ast::ast::PropertyKey<'a>) -> Option<&'a str> {
    match key {
        oxc_ast::ast::PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        oxc_ast::ast::PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

fn simple_name<'a>(pattern: &'a BindingPattern<'a>) -> Option<&'a str> {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        BindingPattern::AssignmentPattern(assignment) => simple_name(&assignment.left),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::NoDeprecatedDeleteSet;
    use crate::rules::script::ScriptLinter;

    fn lint(source: &str) -> crate::rules::script::ScriptLintResult {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoDeprecatedDeleteSet));
        linter.lint(source, 0)
    }

    #[test]
    fn flags_instance_vue_and_imported_helpers() {
        assert_eq!(
            lint("export default { created() { this.$set(this, 'a', 1) } }").error_count,
            1
        );
        assert_eq!(lint("Vue.delete(obj, 'a')").error_count, 1);
        assert_eq!(
            lint("import { set, del as remove } from 'vue'\nset(obj, 'a', 1)\nremove(obj, 'a')")
                .error_count,
            2
        );
        assert_eq!(
            lint("const { set } = require('vue')\nset(obj, 'a', 1)").error_count,
            1
        );
    }

    #[test]
    fn ignores_emit_and_unrelated_set() {
        assert_eq!(
            lint("export default { created() { this.$emit('a') } }").error_count,
            0
        );
        assert_eq!(
            lint("function set(value) { return value }\nset(1)").error_count,
            0
        );
    }
}
