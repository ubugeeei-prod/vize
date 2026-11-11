//! vue/require-default-export
//!
//! A component `<script>` that is not `<script setup>` must default-export
//! the component. The report sits on the script content end, which is the
//! `<` of `</script>`.

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{
    Argument, CallExpression, Declaration, Expression, ObjectPropertyKind, Statement,
};

static META: RuleMeta = RuleMeta {
    name: "vue/require-default-export",
    description: "Require a component's script to be the default export",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

const COMPONENT_KEYS: &[&str] = &[
    "props",
    "setup",
    "data",
    "methods",
    "computed",
    "components",
    "emits",
    "template",
];

/// `vue/require-default-export`.
#[derive(Default)]
pub struct RequireDefaultExport;

impl Rule for RequireDefaultExport {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        // Own the report before `report_in_sfc`. The descriptor and script
        // parse borrow `ctx` for the whole block.
        let report = {
            let Some(descriptor) = ctx.sfc_descriptor() else {
                return;
            };
            // eslint-plugin-vue skips the file when `<script setup>` is present,
            // including a sibling plain `<script>`.
            if descriptor.script_setup.is_some() {
                return;
            }
            let Some(script) = &descriptor.script else {
                return;
            };
            let Some(parsed) = ctx.sfc_scripts().and_then(|scripts| scripts.script()) else {
                return;
            };
            if parsed.program.body.is_empty() {
                return;
            }
            let has_default = parsed
                .program
                .body
                .iter()
                .any(|statement| matches!(statement, Statement::ExportDefaultDeclaration(_)));
            if has_default {
                return;
            }
            let message = if parsed.program.body.iter().any(statement_defines_component) {
                "Component must be the default export."
            } else {
                "Missing default export."
            };
            Some((message, script.loc.end as u32))
        };
        let Some((message, start)) = report else {
            return;
        };
        let end = start.saturating_add(1).min(ctx.source.len() as u32);
        ctx.report_in_sfc(LintDiagnostic::error(META.name, message, start, end));
    }
}

fn statement_defines_component(statement: &Statement<'_>) -> bool {
    match statement {
        Statement::VariableDeclaration(declaration) => declaration
            .declarations
            .iter()
            .any(|declarator| declarator.init.as_ref().is_some_and(expr_is_component)),
        Statement::ExportNamedDeclaration(export) => export
            .declaration
            .as_ref()
            .is_some_and(declaration_defines_component),
        Statement::ExpressionStatement(statement) => expr_is_component(&statement.expression),
        _ => false,
    }
}

fn declaration_defines_component(declaration: &Declaration<'_>) -> bool {
    let Declaration::VariableDeclaration(declaration) = declaration else {
        return false;
    };
    declaration
        .declarations
        .iter()
        .any(|declarator| declarator.init.as_ref().is_some_and(expr_is_component))
}

fn expr_is_component(expression: &Expression<'_>) -> bool {
    match expression.get_inner_expression() {
        Expression::CallExpression(call) => call_defines_component(call),
        Expression::ObjectExpression(object) => object.properties.iter().any(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return false;
            };
            property_name(&property.key).is_some_and(|name| COMPONENT_KEYS.contains(&name))
        }),
        _ => false,
    }
}

fn call_defines_component(call: &CallExpression<'_>) -> bool {
    let Some(name) = callee_name(&call.callee) else {
        return false;
    };
    matches!(name, "defineComponent" | "extend" | "component")
        && call.arguments.first().is_some_and(|argument| {
            matches!(
                argument,
                Argument::ObjectExpression(_) | Argument::Identifier(_)
            )
        })
}

fn callee_name<'a>(callee: &'a Expression<'a>) -> Option<&'a str> {
    match callee.get_inner_expression() {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        Expression::StaticMemberExpression(member) => Some(member.property.name.as_str()),
        _ => None,
    }
}

fn property_name<'a>(key: &'a oxc_ast::ast::PropertyKey<'a>) -> Option<&'a str> {
    match key {
        oxc_ast::ast::PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        oxc_ast::ast::PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::linter::Linter;
    use crate::preset::LintPreset;

    fn messages(source: &str) -> Vec<(u32, String)> {
        Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec!["vue/require-default-export".into()]))
            .lint_sfc(source, "Export.vue")
            .diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.rule_name == "vue/require-default-export")
            .map(|diagnostic| (diagnostic.start, diagnostic.message.to_string()))
            .collect()
    }

    #[test]
    fn a_non_empty_script_without_a_default_export_reports_the_content_end() {
        let source = "<script lang=\"ts\">\nconst label = \"a\";\n</script>\n";
        assert_eq!(
            messages(source),
            vec![(
                source.find("</script>").unwrap() as u32,
                "Missing default export.".into()
            )]
        );
    }

    #[test]
    fn a_named_define_component_must_be_the_default_export() {
        let source = "<script lang=\"ts\">\nexport const label = defineComponent({})\n</script>\n";
        assert_eq!(
            messages(source)[0].1,
            "Component must be the default export."
        );
    }

    #[test]
    fn script_setup_an_empty_body_and_a_default_export_stay_silent() {
        assert!(messages("<script setup>\nconst label = \"a\";\n</script>\n").is_empty());
        assert!(messages("<script lang=\"ts\">\n</script>\n").is_empty());
        assert!(
            messages("<script lang=\"ts\">\nexport default { name: \"A\" }\n</script>\n")
                .is_empty()
        );
        assert!(
            messages("<script>\nexport default {}\n</script>\n<script setup>\nconst label = \"a\"\n</script>\n")
                .is_empty()
        );
    }
}
