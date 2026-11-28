// DO NOT EDIT. Generated from crates/vize_atelier_dom/src/compile/selection.rs by `node tools/commands/generate/composed-copies.ts`.
// Edit the source and run the command again.
//! Lane-selection accounting for the DOM compile path (P3-17).
//!
//! Every DOM template compile that reaches codegen records exactly one
//! selection counter while the active profiler is enabled: `accepted` when
//! the L2 emitter produced the module, or one `legacy.<reason>` naming the
//! first gate that kept the compile on the legacy transform lane. The
//! production-reach gate (`vize_atelier_sfc/tests/davinci_production_reach.rs`)
//! reads these counters around real `compile_sfc` calls, so a reason added
//! here must keep that one-counter-per-compile law. Nothing is recorded while
//! the profiler is disabled (the P2-12b no-observer cost law).

use vize_l0::profiler::with_active_profiler;

/// The counter an L2-emitted DOM compile records.
pub(super) const ACCEPTED_COUNTER: &str = "l2.dom.accepted";

/// Why one DOM compile stayed on the legacy transform lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DomLegacyReason {
    /// The shipped parser reported a fatal error: the compile ends with
    /// diagnostics and no module, before either lane runs.
    ParseError,
    /// The entry point declined L2: the compatibility entries (module-mode
    /// static hoisting outside the sections route) and the legacy lane of
    /// the differential runners.
    Entry,
    /// L2 was attempted and refused the template (an unsupported surface or
    /// a diagnostic the shipped lane must report).
    EmitRefused,
    /// L2 emitted a module, but the compatibility source-map generator
    /// disagreed on code or section boundaries and supplied the result.
    SourceMapMismatch,
    /// SSR options reached the DOM compiler.
    Ssr,
    /// `experimental_patterned_template` is on.
    PatternedTemplate,
    /// The experimental self-component codegen context is on.
    SelfComponent,
    /// A custom renderer target.
    CustomRenderer,
    /// The requested whitespace strategy is not implemented by L2 lowering.
    Whitespace,
    /// Script tables register a component the binding metadata lacks.
    Croquis,
    /// The source text may carry patterned-template directives.
    PatternedSource,
    /// The source text may carry an `@vize:` directive comment.
    DirectiveComment,
    /// The source text may bind a directive a `vue.plugins` entry declares.
    DeclaredDirective,
    /// The `legacy-differential` legacy lane was forced for this thread.
    #[cfg(feature = "legacy-differential")]
    Forced,
}

impl DomLegacyReason {
    /// A stable target-neutral capture reason, without profiler key prefixes.
    pub(super) const fn id(self) -> &'static str {
        match self {
            Self::ParseError => "parse-error",
            Self::Entry => "entry",
            Self::EmitRefused => "emit-refused",
            Self::SourceMapMismatch => "source-map-mismatch",
            Self::Ssr => "ssr",
            Self::PatternedTemplate => "patterned-template",
            Self::SelfComponent => "self-component",
            Self::CustomRenderer => "custom-renderer",
            Self::Whitespace => "whitespace",
            Self::Croquis => "analysis-context",
            Self::PatternedSource => "patterned-source",
            Self::DirectiveComment => "directive-comment",
            Self::DeclaredDirective => "declared-directive",
            #[cfg(feature = "legacy-differential")]
            Self::Forced => "forced",
        }
    }

    /// The profiler counter this reason records.
    pub(super) const fn counter(self) -> &'static str {
        match self {
            Self::ParseError => "l2.dom.legacy.parse_error",
            Self::Entry => "l2.dom.legacy.entry",
            Self::EmitRefused => "l2.dom.legacy.emit_refused",
            Self::SourceMapMismatch => "l2.dom.legacy.source_map_mismatch",
            Self::Ssr => "l2.dom.legacy.ssr",
            Self::PatternedTemplate => "l2.dom.legacy.patterned_template",
            Self::SelfComponent => "l2.dom.legacy.self_component",
            Self::CustomRenderer => "l2.dom.legacy.custom_renderer",
            Self::Whitespace => "l2.dom.legacy.whitespace",
            Self::Croquis => "l2.dom.legacy.croquis",
            Self::PatternedSource => "l2.dom.legacy.patterned_source",
            Self::DirectiveComment => "l2.dom.legacy.directive_comment",
            Self::DeclaredDirective => "l2.dom.legacy.declared_directive",
            #[cfg(feature = "legacy-differential")]
            Self::Forced => "l2.dom.legacy.forced",
        }
    }
}

/// Record the lane one DOM compile took.
pub(super) fn record(outcome: Result<(), DomLegacyReason>) {
    with_active_profiler(|profiler| {
        if !profiler.is_enabled() {
            return;
        }
        let counter = match outcome {
            Ok(()) => ACCEPTED_COUNTER,
            Err(reason) => reason.counter(),
        };
        profiler.record_counter_enabled(counter, 1);
    });
}

/// Keep L2's typed refusal for the differential census. Production builds
/// compile this to the identity.
#[inline]
pub(super) fn noting_refusal<T>(
    emitted: Result<T, vize_l4::targets::dom::artifact::EmitError>,
) -> Result<T, vize_l4::targets::dom::artifact::EmitError> {
    #[cfg(feature = "legacy-differential")]
    if let Err(error) = &emitted {
        differential::note_emit_refusal(*error);
    }
    emitted
}

/// The `legacy-differential` legacy override: while it is armed on a
/// thread, every DOM compile on that thread selects the legacy lane, so a
/// production entry point (`compile_sfc`) can be run on both lanes and its
/// complete output compared byte for byte. Compiled out of production.
#[cfg(feature = "legacy-differential")]
pub use crate::differential;
