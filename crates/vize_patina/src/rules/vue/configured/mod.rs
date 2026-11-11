//! Configurable eslint-plugin-vue rules.
//!
//! Each rule is opt-in and checks nothing until `linter.ruleOptions` names a
//! limit or a forbidden name. An empty object, a missing key, or a limit of
//! zero leaves the check quiet.

mod await_ast;
mod await_call;
mod blocks;
mod component_names;
mod component_options;
mod component_scan;
mod max_props;
mod max_template_depth;
mod props_scan;
mod props_type;
mod restricted_class;
mod restricted_custom_event;
mod restricted_html;
mod restricted_props;
mod restricted_static_attribute;
mod restricted_v_on;
mod template_walk;
mod unsupported_features;

use crate::context::LintContext;
use crate::diagnostic::LintDiagnostic;
use crate::rule::Rule;
use oxc_ast::ast::Program;
use vize_l0::config::RestrictedNames;
use vize_l0::{CompactString, String};

pub(crate) use await_call::NoRestrictedCallAfterAwait;
pub(crate) use blocks::NoRestrictedBlock;
pub(crate) use component_names::NoRestrictedComponentNames;
pub(crate) use component_options::NoRestrictedComponentOptions;
pub(crate) use max_props::MaxProps;
pub(crate) use max_template_depth::MaxTemplateDepth;
pub(crate) use restricted_class::NoRestrictedClass;
pub(crate) use restricted_custom_event::NoRestrictedCustomEvent;
pub(crate) use restricted_html::NoRestrictedHtmlElements;
pub(crate) use restricted_props::NoRestrictedProps;
pub(crate) use restricted_static_attribute::NoRestrictedStaticAttribute;
pub(crate) use restricted_v_on::NoRestrictedVOn;
pub(crate) use unsupported_features::NoUnsupportedFeatures;

pub(super) type Finding = (CompactString, u32, u32);

/// Register the thirteen rules with empty options, so enabling one without
/// `ruleOptions` still checks nothing.
pub(crate) fn register_opt_in(registry: &mut crate::rule::RuleRegistry) {
    for rule in [
        Box::new(MaxProps::default()) as Box<dyn Rule>,
        Box::new(MaxTemplateDepth::default()),
        Box::new(NoRestrictedBlock::default()),
        Box::new(NoRestrictedCallAfterAwait::default()),
        Box::new(NoRestrictedClass::default()),
        Box::new(NoRestrictedComponentNames::default()),
        Box::new(NoRestrictedComponentOptions::default()),
        Box::new(NoRestrictedCustomEvent::default()),
        Box::new(NoRestrictedHtmlElements::default()),
        Box::new(NoRestrictedProps::default()),
        Box::new(NoRestrictedStaticAttribute::default()),
        Box::new(NoRestrictedVOn::default()),
        Box::new(NoUnsupportedFeatures::default()),
    ] {
        if !registry.has_rule(rule.meta().name) {
            registry.register(rule);
        }
    }
}

pub(super) fn listed(names: &RestrictedNames) -> Vec<String> {
    names
        .names()
        .into_iter()
        .filter(|name| !name.is_empty())
        .collect()
}

pub(super) fn hits(names: &[String], name: &str) -> bool {
    names.iter().any(|candidate| candidate == name)
}

pub(super) fn each_program(ctx: &LintContext<'_>, mut visit: impl FnMut(&Program<'_>, u32)) {
    super::compatibility_sfc::collect_script_programs(ctx, |program, offset, _source| {
        visit(program, offset);
    });
}

pub(super) fn report(ctx: &mut LintContext<'_>, rule: &'static str, findings: Vec<Finding>) {
    for (message, start, end) in findings {
        if end > start {
            ctx.report_in_sfc(LintDiagnostic::warn(rule, message, start, end));
        }
    }
}

/// Template findings address `ctx.source`. [`LintContext::report`] adds the
/// template block's offset so the stored range is file-absolute.
pub(super) fn report_template(
    ctx: &mut LintContext<'_>,
    rule: &'static str,
    findings: Vec<Finding>,
) {
    for (message, start, end) in findings {
        if end > start {
            ctx.report(LintDiagnostic::warn(rule, message, start, end));
        }
    }
}

#[cfg(test)]
mod tests;
