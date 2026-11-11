//! Script binding seeds and one all-variant unused conclusion.

use crate::context::LintContext;
use crate::linter::config::{LintResult, Linter};
use crate::rules::facts::{NoUnusedSetupBindings, v_for_source_reads};
use vize_atelier_sfc::SfcDescriptor;
use vize_croquis::{Croquis, Drawer, DrawerOptions};
use vize_l0::Allocator;

pub(super) fn script_summary(
    linter: &Linter,
    filename: &str,
    descriptor: &SfcDescriptor<'_>,
    scripts: &crate::linter::sfc_scripts::SfcScripts<'_>,
) -> Croquis {
    let unused = linter.has_unused_bindings_demand();
    let allocator = Allocator::default();
    // An Art file can also own an ordinary template. Its reads participate in
    // the same physical-script conclusion rather than producing an early one.
    let parsed = descriptor
        .template
        .as_ref()
        .filter(|_| unused)
        .map(|template| crate::markup::L2Template::lower(&allocator, &template.content));
    let admitted = parsed
        .as_ref()
        .filter(|l2| !Linter::has_fatal_l1_template_parse_errors(l2));
    let mut summary = super::super::super::analyze_descriptor_for_lint(
        scripts,
        admitted,
        linter.has_resolved_macro_type_demand().then_some(filename),
        unused,
        false,
    );
    if let Some(l2) = admitted {
        let reads = l2
            .lint_template()
            .with_compatibility_root(v_for_source_reads);
        summary.unused_bindings.retain(|name| !reads.contains(name));
    }
    summary
}

pub(super) fn variant(
    l2: &crate::markup::L2Template<'_>,
    script: &Croquis,
    unused: bool,
) -> Croquis {
    // Copy only the existing script value map and candidate provenance. Each
    // original variant still builds fresh template scopes and template facts;
    // no sibling template scope or mutable script-summary state is shared.
    let mut seed = Croquis::new();
    seed.bindings = script.bindings.clone();
    seed.binding_spans = script.binding_spans.clone();
    if unused {
        seed.unused_bindings = script.unused_bindings.clone();
    }
    let mut drawer = Drawer::with_summary(DrawerOptions::for_lint(), seed, true);
    if unused {
        drawer = drawer.with_unused_bindings();
    }
    drawer.draw_lint_template(l2.lint_template());
    let mut analysis = drawer.finish();
    if unused {
        let reads = l2
            .lint_template()
            .with_compatibility_root(v_for_source_reads);
        analysis
            .unused_bindings
            .retain(|name| !reads.contains(name));
    }
    analysis
}

pub(super) fn report_unused(
    linter: &Linter,
    filename: &str,
    descriptor: &SfcDescriptor<'_>,
    summary: &Croquis,
) -> LintResult {
    let allocator = Allocator::default();
    let mut ctx = LintContext::with_locale(
        &allocator,
        descriptor.source.as_ref(),
        filename,
        linter.locale,
    );
    ctx.set_enabled_rules(linter.enabled_rules.clone());
    ctx.set_config_disabled_rules(linter.disabled_rules.clone());
    ctx.set_config_rule_severities(linter.severity_overrides.clone());
    ctx.set_help_level(linter.help_level);
    ctx.set_no_inline_config(linter.no_inline_config);
    ctx.set_sfc_descriptor(descriptor);
    ctx.set_analysis(summary);
    NoUnusedSetupBindings::report(&mut ctx, None);
    LintResult {
        filename: filename.into(),
        error_count: ctx.error_count(),
        warning_count: ctx.warning_count(),
        diagnostics: ctx.into_diagnostics(),
    }
}
