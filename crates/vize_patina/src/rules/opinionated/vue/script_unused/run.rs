//! Run one of the two script rules against every parsed SFC script.

use oxc_ast::ast::Program;
use vize_relief::RootNode;

use super::art_use::is_art_variant_source;
use super::decide::{Which, analyze};
use super::options::{PreferConstOptions, UnusedOptions};
use super::sfc_use::note_sfc_uses;
use super::template_use::{TemplateUse, template_use};
use crate::context::LintContext;
use crate::diagnostic::LintDiagnostic;

pub(super) fn report(
    ctx: &mut LintContext<'_>,
    root: Option<&RootNode<'_>>,
    which: Which,
    unused: &UnusedOptions,
    prefer: &PreferConstOptions,
    rule: &'static str,
) {
    if !ctx.is_rule_enabled(rule) {
        return;
    }
    if root.is_some()
        && ctx
            .sfc_descriptor()
            .is_some_and(|descriptor| is_art_variant_source(descriptor, ctx.source))
    {
        return;
    }
    let mut template = root.map(template_use).unwrap_or_else(TemplateUse::empty);
    if let Some(descriptor) = ctx.sfc_descriptor() {
        note_sfc_uses(descriptor, &mut template);
    }
    let has_setup = ctx
        .sfc_descriptor()
        .is_some_and(|descriptor| descriptor.script_setup.is_some());
    let mut findings = Vec::new();
    each_script(ctx, |program, offset, setup| {
        findings.extend(analyze(
            program,
            offset,
            &template,
            setup || !has_setup,
            unused,
            prefer,
            match which {
                Which::Unused => Which::Unused,
                Which::PreferConst => Which::PreferConst,
            },
        ));
    });
    for finding in findings {
        ctx.report_in_sfc(LintDiagnostic::warn(
            rule,
            finding.message,
            finding.start,
            finding.end,
        ));
    }
}

fn each_script<'a>(ctx: &LintContext<'a>, mut collect: impl FnMut(&Program<'a>, u32, bool)) {
    let (Some(descriptor), Some(scripts)) = (ctx.sfc_descriptor(), ctx.sfc_scripts()) else {
        return;
    };
    if let (Some(block), Some(parsed)) = (descriptor.script.as_ref(), scripts.script())
        && !parsed.panicked
        && parsed.diagnostics.is_empty()
    {
        collect(&parsed.program, block.loc.start as u32, false);
    }
    if let (Some(block), Some(parsed)) = (descriptor.script_setup.as_ref(), scripts.script_setup())
        && !parsed.panicked
        && parsed.diagnostics.is_empty()
    {
        collect(&parsed.program, block.loc.start as u32, true);
    }
}
