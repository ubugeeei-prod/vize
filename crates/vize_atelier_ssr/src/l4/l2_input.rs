//! The SSR L4 lane for callers that build L2 themselves (JSX/TSX render
//! roots): the same L3 verification, partition facts, string plan, and plan
//! emitter as the template path, with the transform's `prefix_identifiers`
//! off because JSX render functions close over their setup scope.

use vize_davinci::side_table::SideTable;
use vize_l0::Allocator;
use vize_l1_to_l2::TransformExpressions;
use vize_l2::op::Region;

use super::emit::PlanFacts;
use super::select::{L2Artifact, select_from_l2};
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
    let selection = select_l2_lane(allocator, source, root, options);
    record_selection(&selection);
    match selection {
        SsrL4Selection::Emitted(result) => Some(result),
        SsrL4Selection::Legacy(_) | SsrL4Selection::Rejected(_) => None,
    }
}

pub(super) fn select_l2_lane<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    root: &Region<'a>,
    options: &SsrCompilerOptions,
) -> SsrL4Selection {
    let texts = SideTable::new();
    let for_wrappers = SideTable::new();
    let wrappers = SideTable::new();
    let if_facts = SideTable::new();
    let artifact = L2Artifact {
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
    select_from_l2(allocator, &artifact, options, &experimental, || {
        if options.croquis.is_some() || options.binding_metadata.is_some() || options.inline {
            return Err(LegacyReason::Options);
        }
        Ok(TransformExpressions::unprefixed(source, false))
    })
}
