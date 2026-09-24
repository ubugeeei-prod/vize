//! Lane-selection accounting for the DOM compile path (P3-17).
//!
//! Every DOM template compile that reaches codegen records exactly one
//! selection counter while the global profiler is enabled: `accepted` when
//! the S2 emitter produced the module, or one `legacy.<reason>` naming the
//! first gate that kept the compile on the legacy transform lane. The
//! production-reach gate (`vize_atelier_sfc/tests/davinci_production_reach.rs`)
//! reads these counters around real `compile_sfc` calls, so a reason added
//! here must keep that one-counter-per-compile law. Nothing is recorded while
//! the profiler is disabled (the P2-12b no-observer cost law).

use vize_s0::profiler::global_profiler;

/// The counter an S2-emitted DOM compile records.
pub(super) const ACCEPTED_COUNTER: &str = "davinci.s2_dom.accepted";

/// Why one DOM compile stayed on the legacy transform lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DomLegacyReason {
    /// The shipped parser reported a fatal error: the compile ends with
    /// diagnostics and no module, before either lane runs.
    ParseError,
    /// The entry point declined S2: the compatibility entries (module-mode
    /// static hoisting outside the sections route) and the legacy lane of
    /// the differential runners.
    Entry,
    /// S2 was attempted and refused the template (an unsupported surface or
    /// a diagnostic the shipped lane must report).
    EmitRefused,
    /// SSR options reached the DOM compiler.
    Ssr,
    /// `experimental_patterned_template` is on.
    PatternedTemplate,
    /// The experimental self-component codegen context is on.
    SelfComponent,
    /// A custom renderer target.
    CustomRenderer,
    /// The requested whitespace strategy is not implemented by S2 lowering.
    Whitespace,
    /// A Vue 2 / 2.7 dialect.
    Dialect,
    /// Quirks template syntax.
    TemplateSyntax,
    /// A Croquis summary is attached (the script-setup SFC path).
    Croquis,
    /// The source text may carry patterned-template directives.
    PatternedSource,
    /// The source text may carry an `@vize:` directive comment.
    DirectiveComment,
    /// The `davinci-differential` legacy lane was forced for this thread.
    #[cfg(feature = "davinci-differential")]
    Forced,
}

impl DomLegacyReason {
    /// The profiler counter this reason records.
    pub(super) const fn counter(self) -> &'static str {
        match self {
            Self::ParseError => "davinci.s2_dom.legacy.parse_error",
            Self::Entry => "davinci.s2_dom.legacy.entry",
            Self::EmitRefused => "davinci.s2_dom.legacy.emit_refused",
            Self::Ssr => "davinci.s2_dom.legacy.ssr",
            Self::PatternedTemplate => "davinci.s2_dom.legacy.patterned_template",
            Self::SelfComponent => "davinci.s2_dom.legacy.self_component",
            Self::CustomRenderer => "davinci.s2_dom.legacy.custom_renderer",
            Self::Whitespace => "davinci.s2_dom.legacy.whitespace",
            Self::Dialect => "davinci.s2_dom.legacy.dialect",
            Self::TemplateSyntax => "davinci.s2_dom.legacy.template_syntax",
            Self::Croquis => "davinci.s2_dom.legacy.croquis",
            Self::PatternedSource => "davinci.s2_dom.legacy.patterned_source",
            Self::DirectiveComment => "davinci.s2_dom.legacy.directive_comment",
            #[cfg(feature = "davinci-differential")]
            Self::Forced => "davinci.s2_dom.legacy.forced",
        }
    }
}

/// Record the lane one DOM compile took.
pub(super) fn record(outcome: Result<(), DomLegacyReason>) {
    let profiler = global_profiler();
    if !profiler.is_enabled() {
        return;
    }
    let counter = match outcome {
        Ok(()) => ACCEPTED_COUNTER,
        Err(reason) => reason.counter(),
    };
    profiler.record_counter_enabled(counter, 1);
}

/// The `davinci-differential` legacy override: while it is armed on a
/// thread, every DOM compile on that thread selects the legacy lane, so a
/// production entry point (`compile_sfc`) can be run on both lanes and its
/// complete output compared byte for byte. Compiled out of production.
#[cfg(feature = "davinci-differential")]
pub mod differential {
    use core::cell::Cell;

    std::thread_local! {
        static FORCE_LEGACY: Cell<bool> = const { Cell::new(false) };
    }

    /// Historical arm. A projectable Croquis summary is admitted in
    /// production, so this no longer changes the selector.
    pub fn with_croquis_projection<R>(f: impl FnOnce() -> R) -> R {
        f()
    }

    /// Run `f` with every DOM compile on this thread on the legacy lane.
    pub fn with_legacy_lane<R>(f: impl FnOnce() -> R) -> R {
        struct Reset(bool);
        impl Drop for Reset {
            fn drop(&mut self) {
                FORCE_LEGACY.with(|flag| flag.set(self.0));
            }
        }
        let _reset = Reset(FORCE_LEGACY.with(|flag| flag.replace(true)));
        f()
    }

    pub(in crate::compile) fn legacy_forced() -> bool {
        FORCE_LEGACY.with(Cell::get)
    }
}
