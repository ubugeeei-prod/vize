//! Script binding seeds and one all-variant unused conclusion.

use crate::context::LintContext;
use crate::linter::config::{LintResult, Linter};
use crate::rules::facts::{NoUnusedSetupBindings, v_for_source_reads};
use vize_armature::Parser;
use vize_atelier_sfc::SfcDescriptor;
use vize_croquis::{Croquis, Drawer, DrawerOptions, drawer::LintTemplate};
use vize_l0::Allocator;
use vize_relief::RootNode;

pub(super) fn script_summary(
    linter: &Linter,
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
        .map(|template| {
            let l2 = crate::markup::L2Template::lower(&allocator, &template.content);
            let parsed = Parser::new(&allocator, &template.content).parse();
            (l2, parsed)
        });
    let admitted = parsed.as_ref().filter(|(l2, (_, errors))| {
        !Linter::has_fatal_l1_template_parse_errors(l2.surface_errors())
            && !Linter::has_fatal_template_parse_errors(errors)
    });
    let mut summary = super::super::super::analyze_descriptor_for_lint(
        scripts,
        admitted.map(|(l2, _)| l2),
        unused,
        false,
    );
    if let Some((_, (root, _))) = admitted {
        let reads = v_for_source_reads(root);
        summary.unused_bindings.retain(|name| !reads.contains(name));
    }
    summary
}

pub(super) fn variant(
    root: &RootNode<'_>,
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
    let lowered = l2.lowered();
    drawer.draw_lint_template(LintTemplate::new(
        l2.surface(),
        lowered.allocator,
        &lowered.root,
    ));
    let mut analysis = drawer.finish();
    if unused {
        let reads = v_for_source_reads(root);
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
