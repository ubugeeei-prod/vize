//! Production S3 admission and ownership for the native Vapor slice.
//!
//! Acceptance carries the complete checked backend payload. Unsupported inputs
//! select the retained legacy lane explicitly; corrupt invariants never emit.

mod native;
mod text;

use vize_atelier_core::TemplateSyntaxMode;
use vize_carton::{Allocator, String, cstr, profile, profiler::global_profiler};
use vize_s1::SurfaceParseOptions;
use vize_s2_to_s3::Lowered;
use vize_s3::verify::verify;

use native::NativeArtifact;

#[derive(Debug, Clone, Copy)]
pub(crate) struct VaporS3BridgeOptions {
    pub(crate) ssr: bool,
    pub(crate) custom_renderer: bool,
    pub(crate) experimental_in_tag_comments: bool,
    pub(crate) experimental_patterned_template: bool,
    pub(crate) template_syntax: TemplateSyntaxMode,
    pub(crate) has_custom_elements: bool,
    pub(crate) has_binding_metadata: bool,
    pub(crate) inline: bool,
}

/// A selected legacy route is distinct from a failed compiler invariant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LegacyReason {
    Options,
    SurfaceSemantics,
    Operation,
    Element,
    Binding,
    ExpressionOrEncoding,
    Structure,
}

impl LegacyReason {
    fn counter(self) -> &'static str {
        match self {
            Self::Options => "davinci.s3_vapor.legacy.options",
            Self::SurfaceSemantics => "davinci.s3_vapor.legacy.surface_semantics",
            Self::Operation => "davinci.s3_vapor.legacy.operation",
            Self::Element => "davinci.s3_vapor.legacy.element",
            Self::Binding => "davinci.s3_vapor.legacy.binding",
            Self::ExpressionOrEncoding => "davinci.s3_vapor.legacy.expression_or_encoding",
            Self::Structure => "davinci.s3_vapor.legacy.structure",
        }
    }
}

#[derive(Debug)]
pub(super) enum AdmissionFailure {
    Unsupported(LegacyReason),
    Invalid(&'static str),
}

impl From<LegacyReason> for AdmissionFailure {
    fn from(reason: LegacyReason) -> Self {
        Self::Unsupported(reason)
    }
}

#[derive(Debug)]
pub(crate) enum VaporS3BridgeStatus<'a> {
    Legacy(LegacyReason),
    Accepted(VaporS3Artifact<'a>),
    Rejected(std::vec::Vec<String>),
}

/// Private fields prevent callers from confusing a verified generic graph with
/// the narrower executable backend contract. There is no boolean acceptance.
#[derive(Debug)]
pub(crate) struct VaporS3Artifact<'a>(NativeArtifact<'a>);

impl<'a> VaporS3Artifact<'a> {
    pub(crate) fn into_ir(
        self,
        allocator: &'a Allocator,
        source: &'a str,
        scope_id: Option<&str>,
    ) -> crate::ir::RootIRNode<'a> {
        self.0.into_ir(allocator, source, scope_id)
    }
}

pub(crate) fn lower_source_for_vapor<'a>(
    allocator: &'a Allocator,
    source: &str,
    options: VaporS3BridgeOptions,
) -> VaporS3BridgeStatus<'a> {
    if options.ssr
        || options.custom_renderer
        || options.experimental_patterned_template
        || options.template_syntax != TemplateSyntaxMode::Standard
        || options.has_custom_elements
        || options.has_binding_metadata
        || options.inline
    {
        return VaporS3BridgeStatus::Legacy(LegacyReason::Options);
    }
    profile!("atelier.vapor.template.s3_bridge", {
        // Earlier-stage storage cannot accidentally become an emitter input.
        // S3 copies its payloads into the output arena before this scope ends.
        let scratch = Allocator::new();
        let (tree, errors) = vize_s1::parse_with_options(
            &scratch,
            source,
            SurfaceParseOptions {
                experimental_in_tag_comments: options.experimental_in_tag_comments,
            },
        );
        let s2 = vize_s1_to_s2::lower(&scratch, &tree, &errors);
        if !s2.diagnostics.is_empty()
            || s2.provenance.iter().any(|record| {
                !record.rule.starts_with("lower.")
                    && !record.rule.starts_with("condense.")
                    && record.rule != "drop.comment"
            })
        {
            return VaporS3BridgeStatus::Legacy(LegacyReason::SurfaceSemantics);
        }
        let mut s3 = vize_s2_to_s3::lower(allocator, &s2.root);
        if let Err(failure) = text::capture(allocator, &s2, &mut s3) {
            return match failure {
                AdmissionFailure::Unsupported(reason) => VaporS3BridgeStatus::Legacy(reason),
                AdmissionFailure::Invalid(message) => {
                    VaporS3BridgeStatus::Rejected(std::vec![cstr!(
                        "Davinci S3 verifier rejected Vapor artifact: {message}"
                    )])
                }
            };
        }
        admit(s3)
    })
}

fn admit(s3: Lowered<'_>) -> VaporS3BridgeStatus<'_> {
    let violations = verify(&s3.program);
    if !violations.is_empty() {
        return VaporS3BridgeStatus::Rejected(
            violations
                .into_iter()
                .map(|violation| cstr!("Davinci S3 verifier rejected Vapor artifact: {violation}"))
                .collect(),
        );
    }
    // Count equality alone admits duplicate, reordered, stale, or incorrectly
    // classified facts. The producing pass exports one aligned fact per op.
    if s3.partition.ops.len() != s3.program.ops.len()
        || s3
            .partition
            .ops
            .iter()
            .zip(&s3.program.ops)
            .any(|(fact, op)| {
                fact.op != op.id
                    || fact.span != op.span
                    || fact.kind.is_dynamic() != op.effect.is_some()
            })
    {
        return VaporS3BridgeStatus::Rejected(std::vec![String::from(
            "Davinci S3 verifier rejected Vapor artifact: partition identity/classification mismatch",
        )]);
    }
    match NativeArtifact::admit(&s3) {
        Ok(artifact) => VaporS3BridgeStatus::Accepted(VaporS3Artifact(artifact)),
        Err(AdmissionFailure::Unsupported(reason)) => VaporS3BridgeStatus::Legacy(reason),
        Err(AdmissionFailure::Invalid(message)) => VaporS3BridgeStatus::Rejected(std::vec![cstr!(
            "Davinci S3 verifier rejected Vapor artifact: {message}"
        )]),
    }
}

pub(crate) fn record_selection(status: &VaporS3BridgeStatus<'_>) {
    let profiler = global_profiler();
    if !profiler.is_enabled() {
        return;
    }
    let counter = match status {
        VaporS3BridgeStatus::Accepted(_) => "davinci.s3_vapor.accepted",
        VaporS3BridgeStatus::Legacy(reason) => reason.counter(),
        VaporS3BridgeStatus::Rejected(_) => "davinci.s3_vapor.rejected",
    };
    profiler.record_counter_enabled(counter, 1);
}

#[cfg(test)]
mod tests;
