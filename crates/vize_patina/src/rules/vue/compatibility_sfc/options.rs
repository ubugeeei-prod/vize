//! Compatibility rules backed by setup bindings and component option objects.

use super::{absolute_range, callee_name, collect_script_programs, property_name, setup_program};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Expression, ObjectExpression, ObjectPropertyKind,
    Program, Statement,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::GetSpan;
use vize_l0::{FxHashSet, String, cstr};

static NO_PROPS_SHADOW_META: RuleMeta = RuleMeta {
    name: "vue/no-props-shadow",
    description: "Disallow setup declarations that shadow props",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};
static SORT_KEYS_META: RuleMeta = RuleMeta {
    name: "vue/sort-keys",
    description: "Require sorted keys in component option objects",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-props-shadow`.
pub struct NoPropsShadow;

impl Rule for NoPropsShadow {
    fn meta(&self) -> &'static RuleMeta {
        &NO_PROPS_SHADOW_META
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

/// `vue/sort-keys`; default ascending, case-sensitive lexical order.
pub struct SortKeys {
    descending: bool,
    case_sensitive: bool,
    natural: bool,
}

impl Default for SortKeys {
    fn default() -> Self {
        Self {
            descending: false,
            case_sensitive: true,
            natural: false,
        }
    }
}

impl SortKeys {
    /// Build the rule from its ordering, case and natural-sort policy.
    pub fn new(descending: bool, case_sensitive: bool, natural: bool) -> Self {
        Self {
            descending,
            case_sensitive,
            natural,
        }
    }
}

impl Rule for SortKeys {
    fn meta(&self) -> &'static RuleMeta {
        &SORT_KEYS_META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let mut findings = Vec::new();
        collect_script_programs(ctx, |program, offset, _| {
            let mut visitor = SortedVueObjectKeys {
                offset,
                rule: self,
                findings: Vec::new(),
            };
            visitor.visit_program(program);
            findings.extend(visitor.findings);
        });
        for (current, previous, range) in findings {
            ctx.warn_at_with_help(
                cstr!("`{}` should be before `{}`", current, previous),
                range,
                "Keep keys in the configured order inside component option objects",
            );
        }
    }
}

struct SortedVueObjectKeys<'r> {
    offset: u32,
    rule: &'r SortKeys,
    findings: Vec<(String, String, ByteRange)>,
}

impl<'a> Visit<'a> for SortedVueObjectKeys<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if matches!(callee_name(call), Some("defineComponent" | "Vue.extend"))
            && let Some(object) = call.arguments.first().and_then(super::object_argument)
        {
            self.check_vue_object(object);
            return;
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_export_default_declaration(
        &mut self,
        declaration: &oxc_ast::ast::ExportDefaultDeclaration<'a>,
    ) {
        if let Some(expression) = declaration.declaration.as_expression()
            && let Expression::ObjectExpression(object) = expression.get_inner_expression()
        {
            self.check_vue_object(object);
            return;
        }
        walk::walk_export_default_declaration(self, declaration);
    }
}

impl SortedVueObjectKeys<'_> {
    fn check_vue_object(&mut self, object: &ObjectExpression<'_>) {
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            let Some(parent) = property_name(&property.key) else {
                continue;
            };
            if parent == "model" {
                continue;
            }
            if let Expression::ObjectExpression(child) = property.value.get_inner_expression() {
                self.check_object(child);
            }
        }
    }

    fn check_object(&mut self, object: &ObjectExpression<'_>) {
        let mut previous: Option<&str> = None;
        for entry in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = entry else {
                previous = None;
                continue;
            };
            let Some(current) = property_name(&property.key) else {
                previous = None;
                continue;
            };
            if let Some(before) = previous
                && !self.rule.in_order(before, current)
            {
                self.findings.push((
                    current.into(),
                    before.into(),
                    absolute_range(self.offset, property.key.span()),
                ));
            }
            previous = Some(current);
        }
    }
}

impl SortKeys {
    fn in_order(&self, left: &str, right: &str) -> bool {
        let (left, right) = if self.case_sensitive {
            (left.into(), right.into())
        } else {
            (left.to_ascii_lowercase(), right.to_ascii_lowercase())
        };
        let ordering = if self.natural {
            natural_cmp(&left, &right)
        } else {
            left.cmp(&right)
        };
        if self.descending {
            !ordering.is_lt()
        } else {
            !ordering.is_gt()
        }
    }
}

fn natural_cmp(left: &str, right: &str) -> std::cmp::Ordering {
    natural_key(left).cmp(&natural_key(right))
}

fn natural_key(value: &str) -> Vec<NaturalPart<'_>> {
    let mut parts = Vec::new();
    let mut start = 0;
    let bytes = value.as_bytes();
    while let Some(first) = bytes.get(start) {
        let digits = first.is_ascii_digit();
        let mut end = start + 1;
        while bytes
            .get(end)
            .is_some_and(|byte| byte.is_ascii_digit() == digits)
        {
            end += 1;
        }
        if let Some(part) = value.get(start..end) {
            parts.push(if digits {
                NaturalPart::Number(part.parse().unwrap_or(u64::MAX))
            } else {
                NaturalPart::Text(part)
            });
        }
        start = end;
    }
    parts
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum NaturalPart<'a> {
    Number(u64),
    Text(&'a str),
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
