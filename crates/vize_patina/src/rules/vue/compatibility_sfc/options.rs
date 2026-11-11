//! Compatibility rules backed by setup bindings and component option objects.

use super::{absolute_range, callee_name, property_name, setup_program};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Declaration, ExportDefaultDeclarationKind,
    Expression, ImportDeclarationSpecifier, ObjectPropertyKind, Program, Statement,
};
use oxc_ast_visit::{Visit, walk};
use vize_l0::{FxHashSet, String, cstr};

use crate::rules::script::props_emits::type_prop_names::same_file_type_prop_names;

static META: RuleMeta = RuleMeta {
    name: "vue/no-props-shadow",
    description: "Disallow setup declarations that shadow props",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-props-shadow`.
pub struct NoPropsShadow;

impl Rule for NoPropsShadow {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let mut semantic_props = FxHashSet::default();
        if let Some(analysis) = ctx.analysis() {
            semantic_props.extend(
                analysis
                    .macros
                    .props()
                    .iter()
                    .map(|prop| String::from(prop.name.as_str())),
            );
        }
        let findings = setup_program(ctx, |program, offset, _source| {
            let mut props = semantic_props;
            collect_runtime_props(program, &mut props);
            for (name, _) in same_file_type_prop_names(program) {
                props.insert(String::from(name.as_str()));
            }
            collect_prop_shadows(program, offset, &props)
        });
        for (name, range) in findings {
            ctx.warn_at_with_help(
                cstr!(
                    "Declaration of `{}` shadows the prop with the same name",
                    name
                ),
                range,
                "Rename the local declaration or access the prop through a distinct binding",
            );
        }
    }
}

fn collect_runtime_props(program: &Program<'_>, props: &mut FxHashSet<String>) {
    struct RuntimeProps<'p> {
        props: &'p mut FxHashSet<String>,
    }
    impl<'a> Visit<'a> for RuntimeProps<'_> {
        fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
            if callee_name(call) == Some("defineProps")
                && let Some(argument) = call.arguments.first().and_then(Argument::as_expression)
            {
                match argument.get_inner_expression() {
                    Expression::ArrayExpression(array) => {
                        for element in &array.elements {
                            match element
                                .as_expression()
                                .map(Expression::get_inner_expression)
                            {
                                Some(Expression::StringLiteral(prop)) => {
                                    self.props.insert(prop.value.as_str().into());
                                }
                                Some(Expression::TemplateLiteral(template)) => {
                                    if let Some(name) = template.single_quasi() {
                                        self.props.insert(name.as_str().into());
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Expression::ObjectExpression(object) => {
                        for property in &object.properties {
                            let ObjectPropertyKind::ObjectProperty(property) = property else {
                                continue;
                            };
                            if let Some(name) = property_name(&property.key) {
                                self.props.insert(name.into());
                            }
                        }
                    }
                    _ => {}
                }
            }
            walk::walk_call_expression(self, call);
        }
    }
    RuntimeProps { props }.visit_program(program);
}

fn collect_prop_shadows(
    program: &Program<'_>,
    offset: u32,
    props: &FxHashSet<String>,
) -> Vec<(String, ByteRange)> {
    let mut findings = Vec::new();
    for statement in &program.body {
        collect_statement_shadow(statement, offset, props, &mut findings);
    }
    findings
}

/// Module-scope value bindings. The declarator that holds `defineProps()` or
/// `withDefaults(defineProps(), ...)` is the prop binding itself. Type-only
/// declarations (`type`, `interface`, `import type`) are not values.
fn collect_statement_shadow(
    statement: &Statement<'_>,
    offset: u32,
    props: &FxHashSet<String>,
    findings: &mut Vec<(String, ByteRange)>,
) {
    match statement {
        Statement::VariableDeclaration(declaration) => {
            collect_variable_shadow(declaration, offset, props, findings);
        }
        Statement::FunctionDeclaration(function) => {
            if let Some(identifier) = &function.id {
                push_shadow(
                    identifier.name.as_str(),
                    identifier.span,
                    offset,
                    props,
                    findings,
                );
            }
        }
        Statement::ClassDeclaration(class) => {
            if let Some(identifier) = &class.id {
                push_shadow(
                    identifier.name.as_str(),
                    identifier.span,
                    offset,
                    props,
                    findings,
                );
            }
        }
        Statement::TSEnumDeclaration(enumeration) => {
            push_shadow(
                enumeration.id.name.as_str(),
                enumeration.id.span,
                offset,
                props,
                findings,
            );
        }
        Statement::ImportDeclaration(import) if !import.import_kind.is_type() => {
            for specifier in import.specifiers.iter().flatten() {
                let local = match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier)
                        if !specifier.import_kind.is_type() =>
                    {
                        &specifier.local
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        &specifier.local
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        &specifier.local
                    }
                    _ => continue,
                };
                push_shadow(local.name.as_str(), local.span, offset, props, findings);
            }
        }
        Statement::ExportNamedDeclaration(export) => {
            if let Some(declaration) = &export.declaration {
                collect_declaration_shadow(declaration, offset, props, findings);
            }
        }
        Statement::ExportDefaultDeclaration(export) => match &export.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
                if let Some(identifier) = &function.id {
                    push_shadow(
                        identifier.name.as_str(),
                        identifier.span,
                        offset,
                        props,
                        findings,
                    );
                }
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                if let Some(identifier) = &class.id {
                    push_shadow(
                        identifier.name.as_str(),
                        identifier.span,
                        offset,
                        props,
                        findings,
                    );
                }
            }
            _ => {}
        },
        Statement::TSImportEqualsDeclaration(import) if !import.import_kind.is_type() => {
            push_shadow(
                import.id.name.as_str(),
                import.id.span,
                offset,
                props,
                findings,
            );
        }
        _ => {}
    }
}

fn collect_declaration_shadow(
    declaration: &Declaration<'_>,
    offset: u32,
    props: &FxHashSet<String>,
    findings: &mut Vec<(String, ByteRange)>,
) {
    match declaration {
        Declaration::VariableDeclaration(declaration) => {
            collect_variable_shadow(declaration, offset, props, findings);
        }
        Declaration::FunctionDeclaration(function) => {
            if let Some(identifier) = &function.id {
                push_shadow(
                    identifier.name.as_str(),
                    identifier.span,
                    offset,
                    props,
                    findings,
                );
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(identifier) = &class.id {
                push_shadow(
                    identifier.name.as_str(),
                    identifier.span,
                    offset,
                    props,
                    findings,
                );
            }
        }
        Declaration::TSEnumDeclaration(enumeration) => {
            push_shadow(
                enumeration.id.name.as_str(),
                enumeration.id.span,
                offset,
                props,
                findings,
            );
        }
        _ => {}
    }
}

fn collect_variable_shadow(
    declaration: &oxc_ast::ast::VariableDeclaration<'_>,
    offset: u32,
    props: &FxHashSet<String>,
    findings: &mut Vec<(String, ByteRange)>,
) {
    for declarator in &declaration.declarations {
        let holds_props_macro = declarator
            .init
            .as_ref()
            .is_some_and(initializer_is_props_macro);
        if !holds_props_macro {
            collect_shadowed_pattern(&declarator.id, props, offset, findings);
        }
    }
}

/// The initializer is the `defineProps()` call, or `withDefaults` around it.
fn initializer_is_props_macro(expression: &Expression<'_>) -> bool {
    let Some(call) = macro_call(expression) else {
        return false;
    };
    match callee_name(call) {
        Some("defineProps") => true,
        Some("withDefaults") => call
            .arguments
            .first()
            .and_then(Argument::as_expression)
            .is_some_and(|argument| {
                macro_call(argument).is_some_and(|inner| callee_name(inner) == Some("defineProps"))
            }),
        _ => false,
    }
}

fn macro_call<'a>(expression: &'a Expression<'a>) -> Option<&'a CallExpression<'a>> {
    match expression.get_inner_expression() {
        Expression::CallExpression(call) => Some(call),
        _ => None,
    }
}

fn push_shadow(
    name: &str,
    span: oxc_span::Span,
    offset: u32,
    props: &FxHashSet<String>,
    findings: &mut Vec<(String, ByteRange)>,
) {
    if props.contains(name) {
        findings.push((name.into(), absolute_range(offset, span)));
    }
}

fn collect_shadowed_pattern(
    pattern: &BindingPattern<'_>,
    props: &FxHashSet<String>,
    offset: u32,
    findings: &mut Vec<(String, ByteRange)>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => {
            if props.contains(identifier.name.as_str()) {
                findings.push((
                    identifier.name.as_str().into(),
                    absolute_range(offset, identifier.span),
                ));
            }
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                collect_shadowed_pattern(&property.value, props, offset, findings);
            }
            if let Some(rest) = &object.rest {
                collect_shadowed_pattern(&rest.argument, props, offset, findings);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                collect_shadowed_pattern(element, props, offset, findings);
            }
            if let Some(rest) = &array.rest {
                collect_shadowed_pattern(&rest.argument, props, offset, findings);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => {
            collect_shadowed_pattern(&assignment.left, props, offset, findings);
        }
    }
}
