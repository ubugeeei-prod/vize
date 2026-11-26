//! The one binding analysis a script rule may read.
//!
//! The template pass owns the analysis of an SFC with a template. A
//! script-only SFC has no template pass, so the facts a script rule demands
//! are drawn here once, from the same shared script parses.

use crate::linter::config::descriptor_may_have_typed_macros;
use crate::linter::sfc_scripts::ScriptSemantics;
use crate::linter::{Linter, MacroTypeResolution, ReadDemand, analyze_descriptor_for_lint};
use crate::rules::script::TemplateSideReads;
use vize_atelier_sfc::SfcDescriptor;
use vize_croquis::Croquis;

/// Musea `<variant>` markup reads script names in templates of its own.
fn has_art_variants(descriptor: &SfcDescriptor<'_>) -> bool {
    descriptor
        .custom_blocks
        .iter()
        .any(|block| block.block_type.as_ref() == "art")
}

/// A template-less SFC whose styles may bind script names with `v-bind()`.
fn styles_need_reads(linter: &Linter, descriptor: &SfcDescriptor<'_>) -> bool {
    descriptor.template.is_none()
        && !descriptor.styles.is_empty()
        && !has_art_variants(descriptor)
        && super::super::has_template_read_demand(linter)
}

/// The analysis this dispatch draws itself, when the template pass shared none
/// and an active script rule reads one.
pub(super) fn owned(
    linter: &Linter,
    descriptor: &SfcDescriptor<'_>,
    scripts: &ScriptSemantics<'_, '_>,
    filename: &str,
) -> Option<Croquis> {
    let macro_types = super::super::has_resolved_macro_type_demand(linter)
        && descriptor_may_have_typed_macros(descriptor);
    let template_reads = styles_need_reads(linter, descriptor);
    (macro_types || template_reads).then(|| {
        analyze_descriptor_for_lint(
            scripts.parses(),
            None,
            macro_types.then(|| MacroTypeResolution::new(linter.type_sources.as_ref(), filename)),
            ReadDemand {
                unused: false,
                template_reads,
            },
            false,
        )
    })
}

/// What the template side reads, from `analysis` when it recorded the relation.
pub(super) fn template_side<'a>(
    linter: &Linter,
    descriptor: &SfcDescriptor<'_>,
    analysis: Option<&'a Croquis>,
) -> TemplateSideReads<'a> {
    const UNDEFINED_REFS: &str = "vue/no-undefined-refs";
    if has_art_variants(descriptor) {
        return TemplateSideReads::Unknown;
    }
    if let Some(analysis) = analysis
        && let Some(reads) = analysis.template_binding_reads()
    {
        return TemplateSideReads::Analyzed {
            reads,
            bindings: &analysis.bindings,
            bindings_complete: !crate::rules::props_shape::has_unresolved_props(analysis),
            undefined_refs_reported: linter.registry.has_rule(UNDEFINED_REFS)
                && linter.is_rule_enabled(UNDEFINED_REFS),
        };
    }
    if descriptor.template.is_none() && descriptor.styles.is_empty() {
        TemplateSideReads::Nothing
    } else {
        TemplateSideReads::Unknown
    }
}
