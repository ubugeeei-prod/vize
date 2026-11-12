//! Run one of the two script rules against every parsed SFC script.

use oxc_ast::ast::Program;
use oxc_semantic::SemanticBuilder;
use vize_l0::{CompactString, FxHashSet};
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
    // `<script>` and `<script setup>` compile as one module, and both blocks'
    // root bindings are visible to the template. Those borrows end before
    // diagnostics are reported.
    let findings = {
        let scripts = script_programs(ctx);
        let unresolved: Vec<FxHashSet<CompactString>> = scripts
            .iter()
            .map(|(program, _)| unresolved_names(program))
            .collect();
        let mut findings = Vec::new();
        for (index, (program, offset)) in scripts.iter().enumerate() {
            let mut sibling_reads = FxHashSet::default();
            for (other, names) in unresolved.iter().enumerate() {
                if other != index {
                    sibling_reads.extend(names.iter().cloned());
                }
            }
            findings.extend(analyze(
                program,
                *offset,
                &template,
                true,
                &sibling_reads,
                unused,
                prefer,
                match which {
                    Which::Unused => Which::Unused,
                    Which::PreferConst => Which::PreferConst,
                },
            ));
        }
        findings
    };
    for finding in findings {
        ctx.report_in_sfc(LintDiagnostic::warn(
            rule,
            finding.message,
            finding.start,
            finding.end,
        ));
    }
}

fn script_programs<'a>(ctx: &'a LintContext<'a>) -> Vec<(&'a Program<'a>, u32)> {
    let mut found = Vec::new();
    let (Some(descriptor), Some(scripts)) = (ctx.sfc_descriptor(), ctx.sfc_scripts()) else {
        return found;
    };
    if let (Some(block), Some(parsed)) = (descriptor.script.as_ref(), scripts.script())
        && !parsed.panicked
        && parsed.diagnostics.is_empty()
    {
        found.push((&parsed.program, block.loc.start as u32));
    }
    if let (Some(block), Some(parsed)) = (descriptor.script_setup.as_ref(), scripts.script_setup())
        && !parsed.panicked
        && parsed.diagnostics.is_empty()
    {
        found.push((&parsed.program, block.loc.start as u32));
    }
    found
}

/// Names the other script block can resolve by reaching into this module.
fn unresolved_names(program: &Program<'_>) -> FxHashSet<CompactString> {
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(program)
        .semantic;
    let scoping = semantic.scoping();
    let mut names = FxHashSet::default();
    for ids in scoping.root_unresolved_references_ids() {
        for id in ids {
            let reference = scoping.get_reference(id);
            names.insert(CompactString::new(semantic.reference_name(reference)));
        }
    }
    names
}
