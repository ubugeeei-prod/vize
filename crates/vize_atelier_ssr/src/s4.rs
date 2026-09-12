//! Davinci S4 bridge for the SSR compile path.
//!
//! P3-8 keeps SSR on a thin S2->S4 route: the compile path reads the shared
//! S2->S3 partition facts, builds a string-plan witness, and only then falls
//! through to today's payload-rich SSR generator until the plan owns emission.

mod string_plan;

use core::sync::atomic::{AtomicU64, Ordering};

use vize_atelier_core::TemplateSyntaxMode;
use vize_s0::{Allocator, String, cstr, profile, profiler::global_profiler};
use vize_s1::SurfaceParseOptions;
use vize_s3::verify::verify;

pub use string_plan::{
    SsrPartitionSummary, SsrStringPlan, SsrStringPlanError, SsrStringPlanErrorKind,
    SsrStringPlanLowering, SsrStringSegment, SsrStringSegmentKind, lower_s2_to_string_plan,
};

/// Option subset that decides whether the S4 bridge can mirror this SSR compile.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SsrS4BridgeOptions {
    pub(crate) comments: bool,
    pub(crate) custom_renderer: bool,
    pub(crate) experimental_in_tag_comments: bool,
    pub(crate) experimental_patterned_template: bool,
    pub(crate) template_syntax: TemplateSyntaxMode,
    pub(crate) has_custom_elements: bool,
}

/// Result of attempting the SSR S4 bridge for one compile.
#[derive(Debug)]
pub(crate) enum SsrS4BridgeStatus {
    /// The input uses an option shape this S2->S4 slice does not model yet.
    Skipped,
    /// S1->S2->S3 partition facts were read and the string plan was valid.
    Accepted,
    /// The bridge was selected but rejected a stale or inconsistent artifact.
    Rejected(std::vec::Vec<String>),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg(test)]
pub(crate) struct SsrS4BridgeStats {
    pub(crate) attempts: u64,
    pub(crate) accepted: u64,
    pub(crate) skipped: u64,
    pub(crate) rejected: u64,
}

static ATTEMPTS: AtomicU64 = AtomicU64::new(0);
static ACCEPTED: AtomicU64 = AtomicU64::new(0);
static SKIPPED: AtomicU64 = AtomicU64::new(0);
static REJECTED: AtomicU64 = AtomicU64::new(0);

/// Lower `source` through S1->S2, read S2->S3 partition facts, and build the
/// SSR S4 string plan for supported compiles.
pub(crate) fn lower_source_for_ssr(
    allocator: &Allocator,
    source: &str,
    options: SsrS4BridgeOptions,
) -> SsrS4BridgeStatus {
    ATTEMPTS.fetch_add(1, Ordering::Relaxed);

    if !options.is_supported() {
        SKIPPED.fetch_add(1, Ordering::Relaxed);
        return SsrS4BridgeStatus::Skipped;
    }

    let status = profile!("atelier.ssr.template.s4_bridge", {
        let (tree, surface_errors) = vize_s1::parse_with_options(
            allocator,
            source,
            SurfaceParseOptions {
                experimental_in_tag_comments: options.experimental_in_tag_comments,
            },
        );
        let s2 = vize_s1_to_s2::lower(allocator, &tree, &surface_errors);
        let s3 = vize_s2_to_s3::lower(allocator, &s2.root);
        let violations = verify(&s3.program);
        if !violations.is_empty() {
            let mut diagnostics = std::vec::Vec::new();
            for violation in violations {
                diagnostics.push(cstr!(
                    "Davinci S4 verifier rejected SSR bridge input: {violation}"
                ));
            }
            return_rejected(diagnostics)
        } else if s3.partition.ops.len() != s3.program.ops.len() {
            return_rejected(std::vec![cstr!(
                "Davinci S4 verifier rejected SSR bridge input: partition facts {} did not match ops {}",
                s3.partition.ops.len(),
                s3.program.ops.len()
            )])
        } else {
            let lowered = lower_s2_to_string_plan(allocator, &s2.root, &s3.partition);
            if !lowered.errors.is_empty() {
                return_rejected(
                    lowered
                        .errors
                        .iter()
                        .map(|error| {
                            cstr!("Davinci S4 string-plan rejected SSR bridge input: {error:?}")
                        })
                        .collect(),
                )
            } else {
                record_bridge_counters(
                    lowered.plan.segments.len() as u64,
                    lowered.plan.partition.static_segments as u64,
                    lowered.plan.partition.dynamic_segments as u64,
                    s3.partition.ops.len() as u64,
                    s2.diagnostics.len() as u64,
                );
                SsrS4BridgeStatus::Accepted
            }
        }
    });

    if matches!(status, SsrS4BridgeStatus::Accepted) {
        ACCEPTED.fetch_add(1, Ordering::Relaxed);
    }

    status
}

#[cfg(test)]
pub(crate) fn stats() -> SsrS4BridgeStats {
    SsrS4BridgeStats {
        attempts: ATTEMPTS.load(Ordering::Relaxed),
        accepted: ACCEPTED.load(Ordering::Relaxed),
        skipped: SKIPPED.load(Ordering::Relaxed),
        rejected: REJECTED.load(Ordering::Relaxed),
    }
}

impl SsrS4BridgeOptions {
    fn is_supported(self) -> bool {
        !self.comments
            && !self.custom_renderer
            && !self.experimental_patterned_template
            && self.template_syntax == TemplateSyntaxMode::Standard
            && !self.has_custom_elements
    }
}

fn return_rejected(diagnostics: std::vec::Vec<String>) -> SsrS4BridgeStatus {
    REJECTED.fetch_add(1, Ordering::Relaxed);
    SsrS4BridgeStatus::Rejected(diagnostics)
}

fn record_bridge_counters(
    segments: u64,
    static_segments: u64,
    dynamic_segments: u64,
    partition_facts: u64,
    diagnostics: u64,
) {
    let profiler = global_profiler();
    if !profiler.is_enabled() {
        return;
    }
    profiler.record_counter_enabled("davinci.s4_ssr.files", 1);
    profiler.record_counter_enabled("davinci.s4_ssr.segments", segments);
    profiler.record_counter_enabled("davinci.s4_ssr.static_segments", static_segments);
    profiler.record_counter_enabled("davinci.s4_ssr.dynamic_segments", dynamic_segments);
    profiler.record_counter_enabled("davinci.s4_ssr.partition_facts", partition_facts);
    profiler.record_counter_enabled("davinci.s4_ssr.s2_diagnostics", diagnostics);
}

#[cfg(test)]
mod tests {
    use super::{SsrS4BridgeOptions, SsrS4BridgeStatus, lower_source_for_ssr, stats};
    use vize_atelier_core::TemplateSyntaxMode;
    use vize_s0::Allocator;

    fn supported_options() -> SsrS4BridgeOptions {
        SsrS4BridgeOptions {
            comments: false,
            custom_renderer: false,
            experimental_in_tag_comments: false,
            experimental_patterned_template: false,
            template_syntax: TemplateSyntaxMode::Standard,
            has_custom_elements: false,
        }
    }

    #[test]
    fn supported_templates_lower_to_s4_string_plan() {
        let allocator = Allocator::new();
        let before = stats();
        let status = lower_source_for_ssr(
            &allocator,
            r#"<section><h1>{{ title }}</h1><p>Ready</p></section>"#,
            supported_options(),
        );
        let after = stats();

        assert!(matches!(status, SsrS4BridgeStatus::Accepted));
        assert!(after.accepted > before.accepted);
        assert_eq!(after.rejected, before.rejected);
    }

    #[test]
    fn unsupported_ssr_options_skip_the_bridge() {
        let allocator = Allocator::new();
        let before = stats();
        let status = lower_source_for_ssr(
            &allocator,
            r#"<div><!-- kept --></div>"#,
            SsrS4BridgeOptions {
                comments: true,
                ..supported_options()
            },
        );
        let after = stats();

        assert!(matches!(status, SsrS4BridgeStatus::Skipped));
        assert!(after.skipped > before.skipped);
    }
}
