//! S3 bridge for the Vapor compile path.
//!
//! P3-6 moves Vapor toward the canonical S2->S3 backend route. The first
//! production slice keeps today's payload-rich Vapor IR generator in place, but
//! every supported Vapor compile now builds and verifies the S3 program beside
//! it. That makes the S3 contract observable in the real path before payload
//! ownership moves out of the legacy lowering.

use core::sync::atomic::{AtomicU64, Ordering};

use vize_atelier_core::TemplateSyntaxMode;
use vize_carton::{Allocator, String, cstr, profile, profiler::global_profiler};
use vize_s1::SurfaceParseOptions;
use vize_s3::verify::verify;

/// Option subset that decides whether the S3 bridge can mirror this compile.
#[derive(Debug, Clone, Copy)]
pub(crate) struct VaporS3BridgeOptions {
    pub(crate) ssr: bool,
    pub(crate) custom_renderer: bool,
    pub(crate) experimental_in_tag_comments: bool,
    pub(crate) experimental_patterned_template: bool,
    pub(crate) template_syntax: TemplateSyntaxMode,
    pub(crate) has_custom_elements: bool,
}

/// Result of attempting the S3 bridge for one Vapor compile.
#[derive(Debug)]
pub(crate) enum VaporS3BridgeStatus {
    /// The input uses an option shape S3 does not model yet.
    Skipped,
    /// S1->S2->S3 built and verified.
    Accepted,
    /// S3 was selected but failed an invariant.
    Rejected(std::vec::Vec<String>),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg(test)]
pub(crate) struct VaporS3BridgeStats {
    pub(crate) attempts: u64,
    pub(crate) accepted: u64,
    pub(crate) skipped: u64,
    pub(crate) rejected: u64,
}

static ATTEMPTS: AtomicU64 = AtomicU64::new(0);
static ACCEPTED: AtomicU64 = AtomicU64::new(0);
static SKIPPED: AtomicU64 = AtomicU64::new(0);
static REJECTED: AtomicU64 = AtomicU64::new(0);

/// Lower `source` through S1->S2->S3 for supported Vapor compiles.
pub(crate) fn lower_source_for_vapor(
    allocator: &Allocator,
    source: &str,
    options: VaporS3BridgeOptions,
) -> VaporS3BridgeStatus {
    ATTEMPTS.fetch_add(1, Ordering::Relaxed);

    if !options.is_supported() {
        SKIPPED.fetch_add(1, Ordering::Relaxed);
        return VaporS3BridgeStatus::Skipped;
    }

    let status = profile!("atelier.vapor.template.s3_bridge", {
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
                    "Davinci S3 verifier rejected Vapor bridge: {violation}"
                ));
            }
            return_rejected(diagnostics)
        } else if s3.partition.ops.len() != s3.program.ops.len() {
            return_rejected(std::vec![cstr!(
                "Davinci S3 verifier rejected Vapor bridge: partition facts {} did not match ops {}",
                s3.partition.ops.len(),
                s3.program.ops.len()
            )])
        } else {
            let dynamic_ops = s3
                .partition
                .ops
                .iter()
                .filter(|fact| fact.kind.is_dynamic())
                .count();
            record_bridge_counters(
                s3.program.ops.len() as u64,
                dynamic_ops as u64,
                s3.program.regions.len() as u64,
                s3.program.effects.len() as u64,
                s2.diagnostics.len() as u64,
            );
            VaporS3BridgeStatus::Accepted
        }
    });

    if matches!(status, VaporS3BridgeStatus::Accepted) {
        ACCEPTED.fetch_add(1, Ordering::Relaxed);
    }

    status
}

#[cfg(test)]
pub(crate) fn stats() -> VaporS3BridgeStats {
    VaporS3BridgeStats {
        attempts: ATTEMPTS.load(Ordering::Relaxed),
        accepted: ACCEPTED.load(Ordering::Relaxed),
        skipped: SKIPPED.load(Ordering::Relaxed),
        rejected: REJECTED.load(Ordering::Relaxed),
    }
}

impl VaporS3BridgeOptions {
    fn is_supported(self) -> bool {
        !self.ssr
            && !self.custom_renderer
            && !self.experimental_patterned_template
            && self.template_syntax == TemplateSyntaxMode::Standard
            && !self.has_custom_elements
    }
}

fn return_rejected(diagnostics: std::vec::Vec<String>) -> VaporS3BridgeStatus {
    REJECTED.fetch_add(1, Ordering::Relaxed);
    VaporS3BridgeStatus::Rejected(diagnostics)
}

fn record_bridge_counters(
    ops: u64,
    dynamic_ops: u64,
    regions: u64,
    effects: u64,
    diagnostics: u64,
) {
    let profiler = global_profiler();
    if !profiler.is_enabled() {
        return;
    }
    profiler.record_counter_enabled("davinci.s3_vapor.files", 1);
    profiler.record_counter_enabled("davinci.s3_vapor.ops", ops);
    profiler.record_counter_enabled("davinci.s3_vapor.dynamic_ops", dynamic_ops);
    profiler.record_counter_enabled("davinci.s3_vapor.regions", regions);
    profiler.record_counter_enabled("davinci.s3_vapor.effects", effects);
    profiler.record_counter_enabled("davinci.s3_vapor.s2_diagnostics", diagnostics);
}

#[cfg(test)]
mod tests {
    use super::{VaporS3BridgeOptions, VaporS3BridgeStatus, lower_source_for_vapor, stats};
    use vize_atelier_core::TemplateSyntaxMode;
    use vize_carton::Allocator;

    fn supported_options() -> VaporS3BridgeOptions {
        VaporS3BridgeOptions {
            ssr: false,
            custom_renderer: false,
            experimental_in_tag_comments: false,
            experimental_patterned_template: false,
            template_syntax: TemplateSyntaxMode::Standard,
            has_custom_elements: false,
        }
    }

    #[test]
    fn supported_templates_lower_to_verified_s3() {
        let allocator = Allocator::new();
        let before = stats();
        let status = lower_source_for_vapor(
            &allocator,
            r#"<button :disabled="locked" @click="save">{{ label }}</button>"#,
            supported_options(),
        );
        let after = stats();

        assert!(matches!(status, VaporS3BridgeStatus::Accepted));
        assert!(after.accepted > before.accepted);
        assert_eq!(after.rejected, before.rejected);
    }

    #[test]
    fn unsupported_vapor_options_skip_the_bridge() {
        let allocator = Allocator::new();
        let before = stats();
        let status = lower_source_for_vapor(
            &allocator,
            r#"<x-thing />"#,
            VaporS3BridgeOptions {
                has_custom_elements: true,
                ..supported_options()
            },
        );
        let after = stats();

        assert!(matches!(status, VaporS3BridgeStatus::Skipped));
        assert!(after.skipped > before.skipped);
    }
}
