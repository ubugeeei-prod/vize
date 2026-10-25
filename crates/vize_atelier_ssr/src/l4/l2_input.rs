//! The SSR L4 lane for callers that build L2 themselves (JSX/TSX render
//! roots): the same L3 verification, partition facts, string plan, and plan
//! emitter as the template path, with the transform's `prefix_identifiers`
//! off because JSX render functions close over their setup scope.

use vize_l0::side_table::SideTable;
use vize_l0::{Allocator, Span};
use vize_l1_to_l2::TransformExpressions;
use vize_l2::op::Region;

use super::emit::PlanFacts;
use super::select::{L2Artifact, select_from_l2_plain};
use super::{LegacyReason, SsrL4Selection, record_selection};
use crate::codegen::SsrCodegenResult;
use crate::options::{SsrCompilerExperimentalOptions, SsrCompilerOptions};

/// Emit the `ssrRender` module for a caller-built L2 render root from the L4
/// string plan. `None` means the plan does not own the root (an unadmitted
/// shape, or an artifact that broke an invariant) and the caller runs the
/// legacy walker instead.
///
/// The root carries no template-lowering side facts (merged text runs,
/// wrapper keys), so shapes that need them fall back to the walker.
pub fn compile_l2_to_ssr<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    root: &Region<'a>,
    options: &SsrCompilerOptions,
) -> Option<SsrCodegenResult> {
    emit_l2(
        allocator,
        source,
        root,
        options,
        &SsrCompilerExperimentalOptions::default(),
        None,
    )
}

/// Emit a caller-built L2 root with the shared SSR emitter's genuine source
/// origins. The options remain separate from the published exhaustive model.
/// `root_origin` is the actual lowered root's location in `source`, not the
/// complete module's range. It grounds the synthetic render entry in its
/// authored construct rather than fabricating a module-start mapping.
#[doc(hidden)]
pub fn compile_l2_to_ssr_with_experimental_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    root: &Region<'a>,
    root_origin: Span,
    options: &SsrCompilerOptions,
    experimental: &SsrCompilerExperimentalOptions,
) -> Option<SsrCodegenResult> {
    if root_origin.start >= root_origin.end
        || root_origin.end as usize > source.len()
        || !source.is_char_boundary(root_origin.start as usize)
        || !source.is_char_boundary(root_origin.end as usize)
    {
        return None;
    }
    emit_l2(
        allocator,
        source,
        root,
        options,
        experimental,
        Some(root_origin),
    )
}

fn emit_l2<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    root: &Region<'a>,
    options: &SsrCompilerOptions,
    experimental: &SsrCompilerExperimentalOptions,
    root_origin: Option<Span>,
) -> Option<SsrCodegenResult> {
    let selection = vize_l0::profile!(
        "atelier.ssr.caller_l2.select_and_emit",
        select_l2_lane_with_experimental_options(
            allocator,
            source,
            root,
            options,
            experimental,
            root_origin
        )
    );
    record_selection(&selection);
    match selection {
        SsrL4Selection::Emitted(result) => Some(result),
        SsrL4Selection::Legacy(_) | SsrL4Selection::Rejected(_) => None,
    }
}

#[cfg(test)]
pub(super) fn select_l2_lane<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    root: &Region<'a>,
    options: &SsrCompilerOptions,
) -> SsrL4Selection {
    select_l2_lane_with_experimental_options(
        allocator,
        source,
        root,
        options,
        &SsrCompilerExperimentalOptions::default(),
        None,
    )
}

fn select_l2_lane_with_experimental_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    root: &Region<'a>,
    options: &SsrCompilerOptions,
    experimental: &SsrCompilerExperimentalOptions,
    root_origin: Option<Span>,
) -> SsrL4Selection {
    let texts = SideTable::new();
    let for_wrappers = SideTable::new();
    let wrappers = SideTable::new();
    let if_facts = SideTable::new();
    let artifact = L2Artifact {
        source,
        root,
        root_origin,
        facts: PlanFacts {
            texts: &texts,
            for_wrappers: &for_wrappers,
            wrappers: &wrappers,
            if_facts: &if_facts,
            frozen_slots: None,
        },
        diagnostics: 0,
    };
    select_from_l2_plain(
        allocator,
        &artifact,
        options,
        experimental,
        crate::SsrSfcContext::new(true),
        || {
            if options.croquis.is_some() || options.binding_metadata.is_some() || options.inline {
                return Err(LegacyReason::Options);
            }
            Ok(TransformExpressions::unprefixed(source, false))
        },
    )
}
