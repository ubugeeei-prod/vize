//! script/no-computed-properties-in-data
//!
//! `data()` runs before computed properties are initialized, so reading one
//! through `this` copies `undefined` instead of the computed value.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   data() {
//!     return { value: this.doubled }
//!   },
//!   computed: {
//!     doubled() { return 1 }
//!   }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   data() {
//!     return { value: 1 }
//!   },
//!   computed: {
//!     doubled() { return this.value * 2 }
//!   }
//! }
//! ```

use super::component_options::{component_definitions, property_key_name, static_property};
use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, Program, Statement};
use oxc_ast_visit::{
    Visit,
    walk::{walk_arrow_function_expression, walk_function, walk_static_member_expression},
};
use oxc_span::Span;
use oxc_syntax::scope::ScopeFlags;
use vize_l0::FxHashSet;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-computed-properties-in-data",
    description: "Disallow using computed properties inside the component data function",
    default_severity: Severity::Error,
};

const MESSAGE: &str =
    "Computed properties cannot be used in data() because data runs before they are initialized.";

pub struct NoComputedPropertiesInData;

impl ScriptRule for NoComputedPropertiesInData {
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
        if !(source.contains("data") && source.contains("computed")) {
            return;
        }
        for options in component_definitions(program) {
            let names = computed_names(options);
            if names.is_empty() {
                continue;
            }
            let Some(value) = static_property(options, "data") else {
                continue;
            };
            let Some(statements) = data_statements(value) else {
                continue;
            };
            let mut visitor = DataVisitor {
                names: &names,
                depth: 1,
                offset,
                result,
            };
            for statement in statements {
                visitor.visit_statement(statement);
            }
        }
    }
}

fn computed_names<'a>(options: &'a ObjectExpression<'a>) -> FxHashSet<&'a str> {
    let mut names = FxHashSet::default();
    let Some(value) = static_property(options, "computed") else {
        return names;
    };
    let Expression::ObjectExpression(object) = value.get_inner_expression() else {
        return names;
    };
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        if let Some(name) = property_key_name(&property.key) {
            names.insert(name);
        }
    }
    names
}

fn data_statements<'a>(value: &'a Expression<'a>) -> Option<&'a [Statement<'a>]> {
    match value.get_inner_expression() {
        Expression::FunctionExpression(function) => function
            .body
            .as_ref()
            .map(|body| body.statements.as_slice()),
        Expression::ArrowFunctionExpression(arrow) => Some(arrow.body.statements.as_slice()),
        _ => None,
    }
}

struct DataVisitor<'a, 'result> {
    names: &'a FxHashSet<&'a str>,
    depth: u32,
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for DataVisitor<'a, '_> {
    fn visit_function(&mut self, function: &oxc_ast::ast::Function<'a>, flags: ScopeFlags) {
        self.depth += 1;
        walk_function(self, function, flags);
        self.depth -= 1;
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        self.depth += 1;
        walk_arrow_function_expression(self, arrow);
        self.depth -= 1;
    }

    fn visit_static_member_expression(&mut self, it: &oxc_ast::ast::StaticMemberExpression<'a>) {
        if self.depth == 1
            && self.names.contains(it.property.name.as_str())
            && matches!(
                it.object.get_inner_expression(),
                Expression::ThisExpression(_)
            )
        {
            self.report(it.property.span);
        }
        walk_static_member_expression(self, it);
    }
}

impl DataVisitor<'_, '_> {
    fn report(&mut self, span: Span) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        self.result
            .add_diagnostic(LintDiagnostic::error(META.name, MESSAGE, start, end));
    }
}

#[cfg(test)]
mod tests {
    use super::NoComputedPropertiesInData;
    use crate::rules::script::ScriptLinter;

    fn lint(source: &str) -> crate::rules::script::ScriptLintResult {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoComputedPropertiesInData));
        linter.lint(source, 0)
    }

    #[test]
    fn flags_direct_this_computed_read_in_data() {
        let result = lint(
            "export default { data() { return { value: this.doubled } }, computed: { doubled() { return 1 } } }",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn allows_nested_function_and_unrelated_properties() {
        let nested = lint(
            "export default { data() { const read = () => this.doubled; return { read } }, computed: { doubled() { return 1 } } }",
        );
        assert_eq!(nested.error_count, 0);
        let other = lint(
            "export default { data() { return { value: this.count } }, computed: { doubled() { return 1 } } }",
        );
        assert_eq!(other.error_count, 0);
    }
}
