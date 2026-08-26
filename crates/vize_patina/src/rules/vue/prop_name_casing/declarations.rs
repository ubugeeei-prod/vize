//! Prop declarations and the SFC-absolute range each one occupies.
//!
//! Croquis reports macro ranges against the *analysis* view of the script,
//! which concatenates a sibling plain `<script>` in front of `<script setup>`.
//! A diagnostic has to address the file, so each range is shifted back out of
//! that combined view and then into the declaring block's position in the SFC —
//! the same normalisation [`super::super::no_unused_properties`] performs.

use crate::context::LintContext;

/// A declared prop and where it is written.
pub(super) struct Declaration {
    pub(super) name: vize_s0::CompactString,
    pub(super) start: u32,
    pub(super) end: u32,
}

/// Every prop `defineProps` declares, with SFC-absolute ranges.
pub(super) fn declarations(ctx: &LintContext<'_>) -> Vec<Declaration> {
    let Some(descriptor) = ctx.sfc_descriptor() else {
        return Vec::new();
    };
    let script_setup = descriptor.script_setup.as_ref();
    let plain_script = descriptor.script.as_ref();
    let Some(declaring_block) = script_setup.or(plain_script) else {
        return Vec::new();
    };
    let declaring_offset = declaring_block.loc.start as u32;
    let setup_shift = match (plain_script, script_setup) {
        (Some(plain), Some(_)) => plain.content.len() as u32 + 1,
        _ => 0,
    };

    let Some(analysis) = ctx.analysis() else {
        return Vec::new();
    };
    let Some(call) = analysis.macros.define_props() else {
        return Vec::new();
    };
    analysis
        .macros
        .props()
        .iter()
        .map(|prop| {
            let (start, end) = analysis
                .macros
                .prop_declaration(prop.name.as_str())
                .unwrap_or((call.start, call.end));
            Declaration {
                name: prop.name.clone(),
                start: declaring_offset + start.saturating_sub(setup_shift),
                end: declaring_offset + end.saturating_sub(setup_shift),
            }
        })
        .collect()
}
