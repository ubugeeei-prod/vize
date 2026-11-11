//! `vue/no-unsupported-features`.
//!
//! The check covers the script macros `defineModel`, `defineOptions`, and
//! `defineSlots`. Template syntaxes from eslint-plugin-vue are left to their
//! own rules.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::CallExpression;
use oxc_ast_visit::{Visit, walk};
use oxc_span::Span;
use vize_l0::config::UnsupportedFeaturesOptions;
use vize_l0::{String, cstr};

use super::component_scan::{callee_name, callee_span};
use super::{each_program, report};

static META: RuleMeta = RuleMeta {
    name: "vue/no-unsupported-features",
    description: "disallow unsupported Vue.js syntax on the specified version",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-unsupported-features`.
#[derive(Default)]
pub struct NoUnsupportedFeatures {
    minimum: Option<(u32, u32, u32)>,
    ignores: Vec<String>,
}

impl NoUnsupportedFeatures {
    /// Build the rule from `linter.ruleOptions["vue/no-unsupported-features"]`.
    pub(crate) fn from_options(options: &UnsupportedFeaturesOptions) -> Self {
        Self {
            minimum: options.version.as_deref().and_then(minimum_version),
            ignores: options.ignores.clone().unwrap_or_default(),
        }
    }
}

impl Rule for NoUnsupportedFeatures {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let Some(minimum) = self.minimum else {
            return;
        };
        let mut findings = Vec::new();
        each_program(ctx, |program, offset| {
            let mut visitor = FeatureVisitor {
                minimum,
                ignores: &self.ignores,
                findings: Vec::new(),
            };
            visitor.visit_program(program);
            for (message, span) in visitor.findings {
                findings.push((message, offset + span.start, offset + span.end));
            }
        });
        report(ctx, META.name, findings);
    }
}

struct FeatureVisitor<'a> {
    minimum: (u32, u32, u32),
    ignores: &'a [String],
    findings: Vec<(vize_l0::CompactString, Span)>,
}

impl<'a> Visit<'a> for FeatureVisitor<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(name) = callee_name(&call.callee)
            && let Some((floor, printed, id)) = feature(name)
            && self.minimum < floor
            && !self.ignores.iter().any(|ignore| ignore == id)
        {
            let span = callee_span(&call.callee);
            self.findings.push((
                cstr!("`{name}()` macros are not supported until Vue.js \"{printed}\"."),
                span,
            ));
        }
        walk::walk_call_expression(self, call);
    }
}

fn feature(name: &str) -> Option<((u32, u32, u32), &'static str, &'static str)> {
    match name {
        "defineModel" => Some(((3, 4, 0), "3.4.0", "define-model")),
        "defineOptions" => Some(((3, 3, 0), "3.3.0", "define-options")),
        "defineSlots" => Some(((3, 3, 0), "3.3.0", "define-slots")),
        _ => None,
    }
}

/// Minimum of `^` / `~` / `>=` / `>` / exact `x.y.z`. Anything else checks nothing.
fn minimum_version(range: &str) -> Option<(u32, u32, u32)> {
    let range = range.trim();
    let rest = range
        .strip_prefix(">=")
        .or_else(|| range.strip_prefix('>'))
        .or_else(|| range.strip_prefix('^'))
        .or_else(|| range.strip_prefix('~'))
        .unwrap_or(range)
        .trim();
    if !rest
        .bytes()
        .all(|byte| byte.is_ascii_digit() || byte == b'.')
    {
        return None;
    }
    let mut parts = rest.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    parts.next().is_none().then_some((major, minor, patch))
}
