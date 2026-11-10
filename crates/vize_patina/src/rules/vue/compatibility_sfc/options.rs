//! Compatibility rules backed by setup bindings and component option objects.

use super::{absolute_range, callee_name, property_name, setup_program};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Expression, ObjectPropertyKind, Program, Statement,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::GetSpan;
use vize_l0::{FxHashSet, String, cstr};

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
        let findings = setup_program(ctx, |program, offset, source| {
            let mut props = semantic_props;
            collect_runtime_props(program, &mut props);
            collect_prop_shadows(program, source, offset, &props)
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
                            if let Some(Expression::StringLiteral(prop)) = element.as_expression() {
                                self.props.insert(prop.value.as_str().into());
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
    source: &str,
    offset: u32,
    props: &FxHashSet<String>,
) -> Vec<(String, ByteRange)> {
    let mut findings = Vec::new();
    for statement in &program.body {
        match statement {
            Statement::VariableDeclaration(declaration) if !declaration.declare => {
                for declarator in &declaration.declarations {
                    let is_props_binding = declarator.init.as_ref().is_some_and(|initializer| {
                        source
                            .get(initializer.span().start as usize..initializer.span().end as usize)
                            .is_some_and(|text| text.contains("defineProps"))
                    });
                    if !is_props_binding {
                        collect_shadowed_pattern(&declarator.id, props, offset, &mut findings);
                    }
                }
            }
            Statement::FunctionDeclaration(function) if !function.declare => {
                if let Some(identifier) = &function.id
                    && props.contains(identifier.name.as_str())
                {
                    findings.push((
                        identifier.name.as_str().into(),
                        absolute_range(offset, identifier.span),
                    ));
                }
            }
            Statement::ClassDeclaration(class) if !class.declare => {
                if let Some(identifier) = &class.id
                    && props.contains(identifier.name.as_str())
                {
                    findings.push((
                        identifier.name.as_str().into(),
                        absolute_range(offset, identifier.span),
                    ));
                }
            }
            _ => {}
        }
    }
    findings
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
