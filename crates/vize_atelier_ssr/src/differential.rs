//! Two-lane comparison entry for the SSR corpus gate (`davinci-differential`).
//!
//! The published compile path selects one emitter. The corpus gate needs both
//! on the same input: the selected lane, the legacy AST walker pinned, and the
//! selector's verdict, so an admitted template that diverges from the legacy
//! bytes is a measurable failure rather than an unobserved fallback.

#![doc(hidden)]

use vize_atelier_core::CompilerError;
use vize_atelier_core::options::{CustomElementMatcher, TemplateSyntaxMode};
use vize_s0::Allocator;

use crate::compile::{SsrLane, compile_ssr_on_lane};
use crate::s4::{SsrS4Request, SsrS4Selection, select_ssr_lane};
use crate::{SsrCodegenResult, SsrCompilerExperimentalOptions, SsrCompilerOptions};

/// One compile's output on one lane.
#[derive(Debug)]
pub struct SsrLaneOutput {
    pub errors: std::vec::Vec<CompilerError>,
    pub result: SsrCodegenResult,
}

/// The selected lane beside the pinned legacy lane for one template.
#[derive(Debug)]
pub struct SsrLaneComparison {
    /// `s4` when the string plan emitted, `legacy.<reason>` for a selected
    /// legacy route, `rejected` for a broken plan invariant.
    pub lane: &'static str,
    pub selected: SsrLaneOutput,
    pub legacy: SsrLaneOutput,
}

/// Compile `source` on both SSR lanes under the same options.
#[must_use]
pub fn compare_ssr_lanes(
    source: &str,
    options: &SsrCompilerOptions,
    experimental: &SsrCompilerExperimentalOptions,
) -> SsrLaneComparison {
    let lane = {
        let allocator = Allocator::new();
        let selection = select_ssr_lane(
            &allocator,
            source,
            &SsrS4Request {
                options,
                experimental,
                template_syntax: TemplateSyntaxMode::Standard,
                has_custom_elements: false,
            },
        );
        match selection {
            SsrS4Selection::Emitted(_) => "s4",
            SsrS4Selection::Legacy(reason) => reason.counter_suffix(),
            SsrS4Selection::Rejected(_) => "rejected",
        }
    };
    let compile = |lane| {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr_on_lane(
            &allocator,
            source,
            options.clone(),
            TemplateSyntaxMode::Standard,
            CustomElementMatcher::default(),
            experimental.clone(),
            lane,
        );
        SsrLaneOutput { errors, result }
    };
    SsrLaneComparison {
        lane,
        selected: compile(SsrLane::Selected),
        legacy: compile(SsrLane::LegacyOnly),
    }
}
