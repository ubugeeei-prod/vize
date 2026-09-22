//! Davinci S4 lane for the SSR compile path.
//!
//! P3-8 keeps SSR on a thin S2->S4 route: the compile path reads the shared
//! S2->S3 partition facts, builds the SSR string plan, and emits the
//! `ssrRender` module from that plan for the admitted surface. Every other
//! input selects the legacy AST walker explicitly under a typed reason, and a
//! stale or inconsistent artifact is rejected rather than guessed around.

mod bindings;
mod emit;
mod s2_input;
mod select;
mod string_plan;

pub use s2_input::compile_s2_to_ssr;

use vize_atelier_core::TemplateSyntaxMode;
use vize_s0::config::VueVersion;
use vize_s0::{Allocator, String, profile, profiler::global_profiler};
use vize_s1::SurfaceParseOptions;
use vize_s1_to_s2::TransformExpressions;

use crate::codegen::SsrCodegenResult;
use crate::options::{SsrCompilerExperimentalOptions, SsrCompilerOptions};

/// Everything the SSR S4 lane needs to decide and emit one compile.
pub(crate) struct SsrS4Request<'o> {
    pub(crate) options: &'o SsrCompilerOptions,
    pub(crate) experimental: &'o SsrCompilerExperimentalOptions,
    pub(crate) template_syntax: TemplateSyntaxMode,
    pub(crate) has_custom_elements: bool,
}

/// A selected legacy route is distinct from a failed compiler invariant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LegacyReason {
    /// The option surface is outside the S4 lane.
    Options,
    /// A Croquis summary asks for script-aware expression rewrites the
    /// shared transform door does not publish yet (production
    /// `<script setup>` SFCs; the DOM S2 lane refuses them the same way).
    Croquis,
    /// S2 recorded diagnostics or a lowering rule the lane does not model.
    SurfaceSemantics,
    /// An S2 op kind the plan emitter does not own yet.
    Operation,
    /// An element whose content model or runtime helpers are not admitted.
    Element,
    /// An attached binding the plan emitter does not own yet.
    Binding,
    /// An expression without a reproducible transform rewrite.
    ExpressionOrEncoding,
    /// A structural shape outside the admitted surface.
    Structure,
}

impl LegacyReason {
    /// The counter name without the `davinci.s4_ssr.` namespace.
    #[cfg(feature = "davinci-differential")]
    pub(crate) fn counter_suffix(self) -> &'static str {
        let counter = self.counter();
        counter.strip_prefix("davinci.s4_ssr.").unwrap_or(counter)
    }

    const fn counter(self) -> &'static str {
        match self {
            Self::Options => "davinci.s4_ssr.legacy.options",
            Self::Croquis => "davinci.s4_ssr.legacy.croquis",
            Self::SurfaceSemantics => "davinci.s4_ssr.legacy.surface_semantics",
            Self::Operation => "davinci.s4_ssr.legacy.operation",
            Self::Element => "davinci.s4_ssr.legacy.element",
            Self::Binding => "davinci.s4_ssr.legacy.binding",
            Self::ExpressionOrEncoding => "davinci.s4_ssr.legacy.expression_or_encoding",
            Self::Structure => "davinci.s4_ssr.legacy.structure",
        }
    }
}

#[derive(Debug)]
pub(crate) enum AdmissionFailure {
    Unsupported(LegacyReason),
    Invalid(&'static str),
}

impl From<LegacyReason> for AdmissionFailure {
    fn from(reason: LegacyReason) -> Self {
        Self::Unsupported(reason)
    }
}

/// The lane one SSR compile takes.
#[derive(Debug)]
pub(crate) enum SsrS4Selection {
    /// The module was emitted from the S4 string plan.
    Emitted(SsrCodegenResult),
    /// The legacy AST walker owns this compile.
    Legacy(LegacyReason),
    /// The artifact broke an invariant; the legacy walker emits and the
    /// diagnostics surface the broken fact.
    Rejected(std::vec::Vec<String>),
}

/// S2 lowering rules whose output the plan emitter reproduces byte-for-byte.
const ADMITTED_RULES: &[&str] = &[
    "lower.element",
    "lower.text",
    "lower.text-run",
    "lower.text-fact",
    "lower.interpolation",
    "lower.compound",
    "lower.bind",
    "lower.on",
    "lower.if",
    "lower.if-branch-key",
    "lower.branch-wrapper-key",
    "lower.for",
    "lower.for-fact",
    "lower.model",
    "lower.vue-show",
    "lower.vue-html",
    "lower.vue-text",
    "lower.component",
    "lower.slot",
    "lower.slot-content",
    "lower.vue-directive",
    "lower.vue-once",
    "lower.vue-memo",
    "lower.vue-cloak",
    "normalize.bind.same-name",
    "condense.whitespace",
    "condense.drop-whitespace",
    "drop.comment",
    "drop.branch-gap",
    // HTML table construction: a direct row under `<table>` gains a `tbody`.
    "lower.table.implicit-tbody",
];

/// Lower `source` through S1->S2->S3, build the SSR string plan from the
/// shared partition facts, and emit from it when the surface is admitted.
pub(crate) fn select_ssr_lane(
    allocator: &Allocator,
    source: &str,
    request: &SsrS4Request<'_>,
) -> SsrS4Selection {
    let selection = if bridge_supported(request) {
        profile!(
            "atelier.ssr.template.s4_bridge",
            lower_and_emit(allocator, source, request)
        )
    } else {
        SsrS4Selection::Legacy(LegacyReason::Options)
    };
    record_selection(&selection);
    selection
}

fn lower_and_emit(
    allocator: &Allocator,
    source: &str,
    request: &SsrS4Request<'_>,
) -> SsrS4Selection {
    let (tree, surface_errors) = vize_s1::parse_with_options(
        allocator,
        source,
        SurfaceParseOptions {
            experimental_in_tag_comments: request.options.experimental_in_tag_comments,
        },
    );
    let s2 = vize_s1_to_s2::lower(allocator, &tree, &surface_errors);
    let artifact = select::S2Artifact {
        source,
        root: &s2.root,
        facts: emit::PlanFacts {
            texts: &s2.texts,
            for_wrappers: &s2.for_wrappers,
            wrappers: &s2.wrappers,
            if_facts: &s2.if_facts,
        },
        diagnostics: s2.diagnostics.len() as u64,
    };
    let table = request
        .options
        .binding_metadata
        .as_ref()
        .map(bindings::binding_table);
    select::select_from_s2(
        allocator,
        &artifact,
        request.options,
        request.experimental,
        || {
            if request.options.croquis.is_some() {
                return Err(LegacyReason::Croquis);
            }
            if !emission_supported(request) {
                return Err(LegacyReason::Options);
            }
            if !s2.diagnostics.is_empty()
                || s2.provenance.iter().any(|record| {
                    !ADMITTED_RULES.contains(&record.rule.as_str()) || drops_directive(record)
                })
            {
                return Err(LegacyReason::SurfaceSemantics);
            }
            Ok(TransformExpressions::new(
                source,
                table.as_ref(),
                request.options.is_ts,
                request.options.inline,
            ))
        },
    )
}

/// The legacy parser keeps `@vize:` directive comments with `comments` off
/// and its SSR walker renders them, while S2 drops every comment.
fn drops_directive(record: &vize_s2::provenance::ProvenanceRecord) -> bool {
    if !matches!(record.rule.as_str(), "drop.comment" | "drop.branch-gap") {
        return false;
    }
    let text = record.before.as_str();
    let text = text.strip_prefix("<!--").unwrap_or(text);
    let text = text.strip_suffix("-->").unwrap_or(text);
    vize_s0::directive::parse_vize_directive(text, 1, 0).is_some()
}

/// Options under which S1 and S2 see the same template the SSR parser sees.
fn bridge_supported(request: &SsrS4Request<'_>) -> bool {
    let options = request.options;
    !options.comments
        && !options.custom_renderer
        && !options.experimental_patterned_template
        && request.template_syntax == TemplateSyntaxMode::Standard
        && !request.has_custom_elements
}

/// Options whose expression and module semantics the plan emitter owns.
/// Inline render closures, Vue 2 dialect sugar, and in-tag comments stay
/// with the legacy walker, like the S2 DOM lane; a Croquis summary is its
/// own reason ([`LegacyReason::Croquis`]) because it gates production reach.
fn emission_supported(request: &SsrS4Request<'_>) -> bool {
    let options = request.options;
    !options.inline && options.dialect == VueVersion::V3 && !options.experimental_in_tag_comments
}

pub(super) fn record_selection(selection: &SsrS4Selection) {
    let profiler = global_profiler();
    if !profiler.is_enabled() {
        return;
    }
    let counter = match selection {
        SsrS4Selection::Emitted(_) => "davinci.s4_ssr.accepted",
        SsrS4Selection::Legacy(reason) => reason.counter(),
        SsrS4Selection::Rejected(_) => "davinci.s4_ssr.rejected",
    };
    profiler.record_counter_enabled(counter, 1);
}

pub(super) fn record_bridge_counters(
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
mod differential_tests;
#[cfg(test)]
mod tests;
