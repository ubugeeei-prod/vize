//! The SSR S4 lane for callers that build S2 themselves (JSX/TSX render
//! roots): the same S3 verification, partition facts, string plan, and plan
//! emitter as the template path, with the transform's `prefix_identifiers`
//! off because JSX render functions close over their setup scope.

use vize_davinci::side_table::SideTable;
use vize_s0::Allocator;
use vize_s1_to_s2::TransformExpressions;
use vize_s2::op::Region;

use super::emit::PlanFacts;
use super::select::{S2Artifact, select_from_s2};
use super::{LegacyReason, SsrS4Selection, record_selection};
use crate::codegen::SsrCodegenResult;
use crate::options::{SsrCompilerExperimentalOptions, SsrCompilerOptions};

/// Emit the `ssrRender` module for a caller-built S2 render root from the S4
/// string plan. `None` means the plan does not own the root (an unadmitted
/// shape, or an artifact that broke an invariant) and the caller runs the
/// legacy walker instead.
///
/// The root carries no template-lowering side facts (merged text runs,
/// wrapper keys), so shapes that need them fall back to the walker.
pub fn compile_s2_to_ssr<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    root: &Region<'a>,
    options: &SsrCompilerOptions,
) -> Option<SsrCodegenResult> {
    let selection = select_s2_lane(allocator, source, root, options);
    record_selection(&selection);
    match selection {
        SsrS4Selection::Emitted(result) => Some(result),
        SsrS4Selection::Legacy(_) | SsrS4Selection::Rejected(_) => None,
    }
}

pub(super) fn select_s2_lane<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    root: &Region<'a>,
    options: &SsrCompilerOptions,
) -> SsrS4Selection {
    let texts = SideTable::new();
    let for_wrappers = SideTable::new();
    let wrappers = SideTable::new();
    let if_facts = SideTable::new();
    let artifact = S2Artifact {
        source,
        root,
        facts: PlanFacts {
            texts: &texts,
            for_wrappers: &for_wrappers,
            wrappers: &wrappers,
            if_facts: &if_facts,
        },
        diagnostics: 0,
    };
    let experimental = SsrCompilerExperimentalOptions::default();
    select_from_s2(allocator, &artifact, options, &experimental, || {
        if options.croquis.is_some() || options.binding_metadata.is_some() || options.inline {
            return Err(LegacyReason::Options);
        }
        Ok(TransformExpressions::unprefixed(source, false))
    })
}
