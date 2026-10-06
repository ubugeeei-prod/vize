//! script/prefer-use-id
//!
//! Recommend using useId() for generating unique IDs.
//!
//! useId() generates unique IDs that are stable across server and client
//! rendering, making it ideal for accessibility attributes and form elements.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! // Manual ID generation (not SSR-safe)
//! const id = `input-${Math.random()}`
//! const id = `field-${Date.now()}`
//! let counter = 0; const id = `el-${counter++}`
//! ```
//!
//! ### Valid
//! ```ts
//! // Using useId() (Vue 3.5+)
//! const id = useId()
//!
//! // In template
//! <label :for="id">Name</label>
//! <input :id="id" />
//! ```
//!
//! ## Benefits
//!
//! - SSR-safe: Same ID on server and client
//! - Unique: No collisions between component instances
//! - Accessible: Perfect for aria-labelledby, aria-describedby

use oxc_ast::ast::{
    AssignmentExpression, AssignmentTarget, BindingPattern, CallExpression, Expression, Function,
    ObjectProperty, Program, PropertyKey, VariableDeclarator,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_function},
};
use oxc_syntax::scope::ScopeFlags;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/prefer-use-id",
    description: "Recommend using useId() for generating unique IDs (Vue 3.5+)",
    default_severity: Severity::Warning,
};

/// Prefer useId() rule
pub struct PreferUseId;

impl ScriptRule for PreferUseId {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        PreferUseIdVisitor {
            offset,
            result,
            id_context: false,
        }
        .visit_program(program);
    }
}

struct PreferUseIdVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
    id_context: bool,
}

impl<'a> Visit<'a> for PreferUseIdVisitor<'_> {
    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        let previous = self.id_context;
        self.id_context |= function
            .id
            .as_ref()
            .is_some_and(|identifier| is_id_name(identifier.name.as_str()));
        walk_function(self, function, flags);
        self.id_context = previous;
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        self.visit_binding_pattern(&declarator.id);
        let previous = self.id_context;
        self.id_context = matches!(&declarator.id,
            BindingPattern::BindingIdentifier(identifier) if is_id_name(identifier.name.as_str()));
        if let Some(initializer) = &declarator.init {
            self.visit_expression(initializer);
        }
        self.id_context = previous;
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        self.visit_assignment_target(&assignment.left);
        let previous = self.id_context;
        self.id_context = match &assignment.left {
            AssignmentTarget::AssignmentTargetIdentifier(identifier) => {
                is_id_name(identifier.name.as_str())
            }
            AssignmentTarget::StaticMemberExpression(member) => {
                is_id_name(member.property.name.as_str())
            }
            AssignmentTarget::ComputedMemberExpression(member) => {
                matches!(member.expression.get_inner_expression(),
                    Expression::StringLiteral(string) if is_id_name(string.value.as_str()))
            }
            _ => false,
        };
        self.visit_expression(&assignment.right);
        self.id_context = previous;
    }

    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        self.visit_property_key(&property.key);
        let previous = self.id_context;
        self.id_context |= !property.computed
            && match &property.key {
                PropertyKey::StaticIdentifier(identifier) => is_id_name(identifier.name.as_str()),
                PropertyKey::StringLiteral(string) => is_id_name(string.value.as_str()),
                _ => false,
            };
        self.visit_expression(&property.value);
        self.id_context = previous;
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.id_context && is_id_generator(call) {
            self.result.add_diagnostic(
                LintDiagnostic::warn(
                    META.name,
                    "Consider using useId() for generating IDs (Vue 3.5+)",
                    self.offset as u32 + call.span.start,
                    self.offset as u32 + call.span.end,
                )
                .with_help("useId() provides SSR-safe, unique IDs: `const id = useId()`"),
            );
        }
        walk_call_expression(self, call);
    }
}

fn is_id_name(name: &str) -> bool {
    name.as_bytes()
        .windows(2)
        .any(|part| part.eq_ignore_ascii_case(b"id"))
        || name.contains("unique")
}

fn is_id_generator(call: &CallExpression<'_>) -> bool {
    if !call.arguments.is_empty() {
        return false;
    }
    let Expression::StaticMemberExpression(member) = call.callee.get_inner_expression() else {
        return false;
    };
    let Expression::Identifier(object) = member.object.get_inner_expression() else {
        return false;
    };
    matches!(
        (object.name.as_str(), member.property.name.as_str()),
        ("Math", "random") | ("Date", "now") | ("crypto", "randomUUID")
    )
}

#[cfg(test)]
mod tests {
    use super::PreferUseId;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(PreferUseId));
        linter
    }

    #[test]
    fn test_valid_use_id() {
        let linter = create_linter();
        let result = linter.lint("const id = useId()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_warns_math_random_id() {
        let linter = create_linter();
        let result = linter.lint("const id = `input-${Math.random()}`", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_warns_date_now_id() {
        let linter = create_linter();
        let result = linter.lint("const uniqueId = `el-${Date.now()}`", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_no_warn_random_not_id() {
        let linter = create_linter();
        let result = linter.lint("const value = Math.random() * 100", 0);
        assert_eq!(result.warning_count, 0);
    }
}
