//! The S2 -> S3 -> string plan -> emission half of the SSR S4 lane, shared by
//! the template compile path and callers that build S2 themselves (JSX).

use vize_s0::{Allocator, String, cstr};
use vize_s1_to_s2::TransformExpressions;
use vize_s2::op::Region;
use vize_s3::verify::verify;

use super::emit::{self, PlanFacts};
use super::string_plan::lower_s2_to_string_plan;
use super::{AdmissionFailure, LegacyReason, SsrS4Selection, record_bridge_counters};
use crate::codegen::SsrCodegenContext;
use crate::options::{SsrCompilerExperimentalOptions, SsrCompilerOptions};

/// One S2 artifact and the side facts the emitter reads beside it.
pub(super) struct S2Artifact<'s, 'a> {
    pub(super) source: &'a str,
    pub(super) root: &'s Region<'a>,
    pub(super) facts: PlanFacts<'s>,
    pub(super) diagnostics: u64,
}

/// Lower `artifact` to S3, verify it, build the string plan from the shared
/// partition facts, and emit from it when `admit` hands back the expression
/// rewriter (its `Err` names the legacy reason instead).
pub(super) fn select_from_s2<'a, 'e>(
    allocator: &'a Allocator,
    artifact: &S2Artifact<'_, 'a>,
    options: &SsrCompilerOptions,
    experimental: &SsrCompilerExperimentalOptions,
    admit: impl FnOnce() -> Result<TransformExpressions<'e>, LegacyReason>,
) -> SsrS4Selection {
    let s3 = vize_s2_to_s3::lower(allocator, artifact.root);
    let violations = verify(&s3.program);
    if !violations.is_empty() {
        return SsrS4Selection::Rejected(
            violations
                .into_iter()
                .map(|violation| {
                    cstr!("Davinci S4 verifier rejected SSR bridge input: {violation}")
                })
                .collect(),
        );
    }
    if s3.partition.ops.len() != s3.program.ops.len() {
        return SsrS4Selection::Rejected(std::vec![cstr!(
            "Davinci S4 verifier rejected SSR bridge input: partition facts {} did not match ops {}",
            s3.partition.ops.len(),
            s3.program.ops.len()
        )]);
    }
    let lowered = lower_s2_to_string_plan(allocator, artifact.root, &s3.partition);
    if !lowered.errors.is_empty() {
        return SsrS4Selection::Rejected(
            lowered
                .errors
                .iter()
                .map(|error| cstr!("Davinci S4 string-plan rejected SSR bridge input: {error:?}"))
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
        Err(reason) => return SsrS4Selection::Legacy(reason),
    };
    let mut ctx = SsrCodegenContext::new_with_experimental_options(
        allocator,
        options,
        artifact.source,
        experimental.clone(),
    );
    // The S2 program covers the whole template source, which starts at 0.
    ctx.begin_render(0);
    match emit::emit_plan(&mut ctx, &lowered.plan, &artifact.facts, &mut exprs) {
        Ok(()) => SsrS4Selection::Emitted(ctx.finish_render()),
        Err(AdmissionFailure::Unsupported(reason)) => SsrS4Selection::Legacy(reason),
        Err(AdmissionFailure::Invalid(message)) => SsrS4Selection::Rejected(std::vec![cstr!(
            "Davinci S4 string-plan emitter rejected SSR artifact: {message}"
        )]),
    }
}
