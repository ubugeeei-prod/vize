//! The L2 -> L3 -> string plan -> emission half of the SSR L4 lane, shared by
//! the template compile path and callers that build L2 themselves (JSX).

use vize_l0::{Allocator, String, cstr};
use vize_l1_to_l2::TransformExpressions;
use vize_l2::op::Region;
use vize_l3::verify::verify;

use super::emit::{self, PlanFacts};
use super::string_plan::lower_l2_to_string_plan;
use super::{AdmissionFailure, LegacyReason, SsrL4Selection, record_bridge_counters};
use crate::codegen::SsrCodegenContext;
use crate::options::{SsrCompilerExperimentalOptions, SsrCompilerOptions};

/// One L2 artifact and the side facts the emitter reads beside it.
pub(super) struct L2Artifact<'s, 'a> {
    pub(super) source: &'a str,
    pub(super) root: &'s Region<'a>,
    pub(super) facts: PlanFacts<'s>,
    pub(super) diagnostics: u64,
}

/// Lower `artifact` to L3, verify it, build the string plan from the shared
/// partition facts, and emit from it when `admit` hands back the expression
/// rewriter (its `Err` names the legacy reason instead).
pub(super) fn select_from_l2<'a, 'e>(
    allocator: &'a Allocator,
    artifact: &L2Artifact<'_, 'a>,
    options: &SsrCompilerOptions,
    experimental: &SsrCompilerExperimentalOptions,
    admit: impl FnOnce() -> Result<TransformExpressions<'e>, LegacyReason>,
) -> SsrL4Selection {
    let s3 = vize_l2_to_l3::lower(allocator, artifact.root);
    let violations = verify(&s3.program);
    if !violations.is_empty() {
        return SsrL4Selection::Rejected(
            violations
                .into_iter()
                .map(|violation| {
                    cstr!("Davinci L4 verifier rejected SSR bridge input: {violation}")
                })
                .collect(),
        );
    }
    if s3.partition.ops.len() != s3.program.ops.len() {
        return SsrL4Selection::Rejected(std::vec![cstr!(
            "Davinci L4 verifier rejected SSR bridge input: partition facts {} did not match ops {}",
            s3.partition.ops.len(),
            s3.program.ops.len()
        )]);
    }
    let lowered = lower_l2_to_string_plan(allocator, artifact.root, &s3.partition);
    if !lowered.errors.is_empty() {
        return SsrL4Selection::Rejected(
            lowered
                .errors
                .iter()
                .map(|error| cstr!("Davinci L4 string-plan rejected SSR bridge input: {error:?}"))
                .collect::<std::vec::Vec<String>>(),
        );
    }
    record_bridge_counters(
        lowered.plan.segments.len() as u64,
        lowered.plan.partition.static_segments as u64,
        lowered.plan.partition.dynamic_segments as u64,
        s3.partition.ops.len() as u64,
        artifact.diagnostics,
    );

    let mut exprs = match admit() {
        Ok(exprs) => exprs,
        Err(reason) => return SsrL4Selection::Legacy(reason),
    };
    let mut ctx = SsrCodegenContext::new_with_experimental_options(
        allocator,
        options,
        artifact.source,
        experimental.clone(),
    );
    // The L2 program covers the whole template source, which starts at 0.
    ctx.begin_render(0);
    match emit::emit_plan(&mut ctx, &lowered.plan, &artifact.facts, &mut exprs) {
        Ok(()) => SsrL4Selection::Emitted(ctx.finish_render()),
        Err(AdmissionFailure::Unsupported(reason)) => SsrL4Selection::Legacy(reason),
        Err(AdmissionFailure::Invalid(message)) => SsrL4Selection::Rejected(std::vec![cstr!(
            "Davinci L4 string-plan emitter rejected SSR artifact: {message}"
        )]),
    }
}
