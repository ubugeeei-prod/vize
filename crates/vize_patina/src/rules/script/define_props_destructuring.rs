//! script/define-props-destructuring
//!
//! Match Vue's props destructuring style preference. The default requires
//! destructuring for assigned and bare typed calls; `only-when-assigned` skips
//! bare calls and `never` preserves explicit props-object access. Vue 3.5
//! destructures remain reactive.
//!
//! ### Invalid
//! ```ts
//! defineProps<{ foo: string }>()
//! ```
//!
//! ### Valid
//! ```ts
//! const { foo, bar = 'default' } = defineProps<{ foo: string; bar?: string }>()
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    BindingPattern, CallExpression, Declaration, Expression, ExpressionStatement, Program,
    Statement, TSType, TSTypeName, VariableDeclarator,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_expression_statement, walk_variable_declarator},
};
use oxc_span::{GetSpan, Span};
use vize_l0::config::PropsDestructureMode;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/define-props-destructuring",
    description: "Enforce consistent style for defineProps destructuring in <script setup>",
    default_severity: Severity::Warning,
};

/// Use the default `always` props destructuring preference.
pub struct DefinePropsDestructuring;

impl DefinePropsDestructuring {
    pub fn configured(mode: PropsDestructureMode) -> impl ScriptRule {
        ConfiguredDefinePropsDestructuring(mode)
    }
}

struct ConfiguredDefinePropsDestructuring(PropsDestructureMode);

impl ScriptRule for DefinePropsDestructuring {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }
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
        check(
            program,
            source,
            offset,
            result,
            PropsDestructureMode::default(),
        );
    }
}

impl ScriptRule for ConfiguredDefinePropsDestructuring {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }
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
        check(program, source, offset, result, self.0);
    }
}

fn check<'a>(
    program: &'a Program<'a>,
    source: &'a str,
    offset: usize,
    result: &mut ScriptLintResult,
    mode: PropsDestructureMode,
) {
    DefinePropsDestructuringVisitor {
        program,
        source,
        offset,
        result,
        mode,
    }
    .visit_program(program);
}

struct DefinePropsDestructuringVisitor<'ast, 'result> {
    program: &'ast Program<'ast>,
    source: &'ast str,
    offset: usize,
    result: &'result mut ScriptLintResult,
    mode: PropsDestructureMode,
}

#[derive(Clone, Copy)]
struct DefinePropsCall {
    call_span: Span,
    with_defaults_callee_span: Option<Span>,
}

impl DefinePropsDestructuringVisitor<'_, '_> {
    fn report(&mut self, span: Span, message: &'static str, help: &'static str) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        self.result
            .add_diagnostic(LintDiagnostic::warn(META.name, message, start, end).with_help(help));
    }
    fn prefer_destructuring(&mut self, span: Span) {
        self.report(span, "Prefer destructuring the return value of defineProps().", "Use `const { prop = defaultValue } = defineProps<Props>()`; Vue 3.5+ preserves reactivity.");
    }
}

impl<'a> Visit<'a> for DefinePropsDestructuringVisitor<'a, '_> {
    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        if let Some(init) = &it.init
            && let Some(call) = define_props_call(init, self.program, self.source)
        {
            let destructured = matches!(
                it.id,
                BindingPattern::ObjectPattern(_) | BindingPattern::ArrayPattern(_)
            );
            match self.mode {
                PropsDestructureMode::Never if destructured => self.report(call.call_span, "Avoid destructuring the return value of defineProps().", "Assign props to a single binding and access `props.foo` to follow the configured style preference."),
                PropsDestructureMode::Never => {},
                _ if !destructured => self.prefer_destructuring(call.call_span),
                _ if let Some(callee_span) = call.with_defaults_callee_span => self.report(callee_span, "Avoid using withDefaults() with props destructuring.", "Put defaults directly in the destructuring pattern for Vue 3.5+."),
                _ => {},
            }
        }
        walk_variable_declarator(self, it);
    }

    fn visit_expression_statement(&mut self, it: &ExpressionStatement<'a>) {
        if self.mode == PropsDestructureMode::Always
            && let Some(call) = define_props_call(&it.expression, self.program, self.source)
        {
            self.prefer_destructuring(call.call_span);
        }
        walk_expression_statement(self, it);
    }
}

/// Recognize the compiler macro and its `withDefaults` wrapper, while skipping
/// calls that declare no props and member calls belonging to other APIs.
fn define_props_call(
    expression: &Expression<'_>,
    program: &Program<'_>,
    source: &str,
) -> Option<DefinePropsCall> {
    let Expression::CallExpression(call) = expression.get_inner_expression() else {
        return None;
    };
    if call_is_named(call, "defineProps") {
        return call_declares_props(call, program, source).then_some(DefinePropsCall {
            call_span: call.span,
            with_defaults_callee_span: None,
        });
    }
    if call_is_named(call, "withDefaults")
        && let Some(first) = call
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
    {
        return define_props_call(first, program, source).map(|mut define_props| {
            define_props.with_defaults_callee_span = Some(call.callee.span());
            define_props
        });
    }
    None
}

fn call_declares_props(call: &CallExpression<'_>, program: &Program<'_>, source: &str) -> bool {
    if let Some(argument) = call.arguments.first() {
        return match argument
            .as_expression()
            .map(Expression::get_inner_expression)
        {
            Some(Expression::ObjectExpression(object)) => !object.properties.is_empty(),
            Some(Expression::ArrayExpression(array)) => !array.elements.is_empty(),
            Some(_) | None => true,
        };
    }
    call.type_arguments
        .as_ref()
        .and_then(|arguments| arguments.params.first())
        .is_some_and(|ty| type_declares_props(ty, program, source, 0))
}

fn type_declares_props(ty: &TSType<'_>, program: &Program<'_>, source: &str, depth: u8) -> bool {
    if depth >= 16 {
        return true;
    }
    match ty {
        TSType::TSTypeLiteral(literal) => !literal.members.is_empty(),
        TSType::TSTypeReference(reference) => {
            let TSTypeName::IdentifierReference(identifier) = &reference.type_name else {
                return true;
            };
            let name = identifier.name.as_str();
            for statement in &program.body {
                match declaration(statement) {
                    Some(Declaration::TSInterfaceDeclaration(interface))
                        if interface.id.name == name =>
                    {
                        if !interface.body.body.is_empty() {
                            return true;
                        }
                        if interface.extends.is_empty() {
                            return false;
                        }
                        let authored = source
                            .get(interface.span.start as usize..interface.span.end as usize)
                            .unwrap_or_default();
                        return !authored.contains("@vue-ignore");
                    }
                    Some(Declaration::TSTypeAliasDeclaration(alias)) if alias.id.name == name => {
                        return type_declares_props(
                            &alias.type_annotation,
                            program,
                            source,
                            depth + 1,
                        );
                    }
                    _ => {}
                }
            }
            true
        }
        TSType::TSIntersectionType(intersection) => intersection
            .types
            .iter()
            .any(|ty| type_declares_props(ty, program, source, depth + 1)),
        TSType::TSUnionType(union) => union
            .types
            .iter()
            .any(|ty| type_declares_props(ty, program, source, depth + 1)),
        TSType::TSParenthesizedType(parenthesized) => {
            type_declares_props(&parenthesized.type_annotation, program, source, depth + 1)
        }
        _ => true,
    }
}

fn declaration<'a>(statement: &'a Statement<'a>) -> Option<&'a Declaration<'a>> {
    match statement {
        Statement::ExportNamedDeclaration(export) => export.declaration.as_ref(),
        _ => statement.as_declaration(),
    }
}

fn call_is_named(call: &CallExpression<'_>, name: &str) -> bool {
    matches!(call.callee.get_inner_expression(), Expression::Identifier(identifier) if identifier.name.as_str() == name)
}
#[cfg(test)]
mod tests {
    use super::DefinePropsDestructuring;
    use crate::rules::script::ScriptLinter;
    use vize_l0::config::PropsDestructureMode;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(DefinePropsDestructuring::configured(
            PropsDestructureMode::Never,
        )));
        linter
    }

    fn always_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(DefinePropsDestructuring));
        linter
    }

    #[test]
    fn test_valid_assigned_to_identifier() {
        let result = create_linter().lint("const props = defineProps<{ foo: string }>()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_no_assignment() {
        let result = create_linter().lint("defineProps<{ foo: string }>()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_unrelated_destructure() {
        let result = create_linter().lint("const { foo } = someObject", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_object_destructure() {
        let result = create_linter().lint(
            "const { foo, bar } = defineProps<{ foo: string; bar?: number }>()",
            0,
        );
        assert_eq!(result.warning_count, 1);
        assert_eq!(
            result.diagnostics[0].help.as_deref(),
            Some(
                "Assign props to a single binding and access `props.foo` to follow the configured style preference."
            )
        );
    }

    #[test]
    fn test_invalid_object_destructure_with_defaults() {
        let result =
            create_linter().lint("const { count = 0 } = defineProps<{ count?: number }>()", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_array_destructure() {
        let result = create_linter().lint("const [first] = defineProps<[string]>()", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_with_defaults_destructure() {
        let result = create_linter().lint(
            "const { count = 0 } = withDefaults(defineProps<{ count?: number }>(), { count: 0 })",
            0,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_offset_applied() {
        let source = "const { foo } = defineProps<{ foo: string }>()";
        let result = create_linter().lint(source, 200);
        assert_eq!(result.warning_count, 1);
        let call_start = source.find("defineProps").unwrap() as u32 + 200;
        assert_eq!(result.diagnostics[0].start, call_start);
    }

    #[test]
    fn always_skips_macros_that_resolve_to_no_props() {
        for source in [
            "interface Props extends /* @vue-ignore */ ExternalProps {}\ndefineProps<Props>()",
            "interface Props {}\nconst props = defineProps<Props>()",
            "defineProps<{}>()",
            "defineProps({})",
            "defineProps([])",
        ] {
            assert_eq!(always_linter().lint(source, 0).warning_count, 0, "{source}");
        }
    }

    #[test]
    fn always_reports_locally_declared_and_unresolved_props() {
        for source in [
            "interface Props { label: string }\ndefineProps<Props>()",
            "interface Props extends ExternalProps {}\ndefineProps<Props>()",
            "defineProps<ImportedProps>()",
            "defineProps({ label: String })",
            "defineProps(['label'])",
        ] {
            assert_eq!(always_linter().lint(source, 0).warning_count, 1, "{source}");
        }
    }

    #[test]
    fn always_reports_the_inner_define_props_call_in_a_bare_wrapper() {
        let source = "withDefaults(defineProps<{ label?: string }>(), { label: '' })";
        let result = always_linter().lint(source, 0);
        assert_eq!(result.warning_count, 1);
        let start = source.find("defineProps").unwrap() as u32;
        let end = source.find(">(),").unwrap() as u32 + 3;
        assert_eq!(
            (result.diagnostics[0].start, result.diagnostics[0].end),
            (start, end)
        );
    }
}
