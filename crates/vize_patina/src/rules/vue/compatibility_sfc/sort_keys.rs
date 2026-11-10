//! `vue/sort-keys` compatibility rule.

use super::{absolute_range, callee_name, collect_script_programs, object_argument, property_name};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{CallExpression, Expression, ObjectExpression, ObjectPropertyKind};
use oxc_ast_visit::{Visit, walk};
use oxc_span::GetSpan;
use vize_l0::{String, cstr};

static META: RuleMeta = RuleMeta {
    name: "vue/sort-keys",
    description: "Require sorted keys in component option objects",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require component option object keys in the configured order.
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

impl Rule for SortKeys {
    fn meta(&self) -> &'static RuleMeta {
        &META
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
            && let Some(object) = call.arguments.first().and_then(object_argument)
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
        let mut previous = None;
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
