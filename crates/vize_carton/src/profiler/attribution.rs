//! Optional structured source-level attribution for profile spans.
//!
//! Today's span identity is one dotted `&'static str`
//! (`"atelier.dom.template.parse"`). Attribution extends that identity with
//! optional `{stage, pass, file_id, block, span}` coordinates so the Davinci
//! pass manager can report pass × stage × block × span costs. Every field is
//! optional and the empty attribution is exactly the historical dotted key, so
//! existing `profile!` call sites keep their identity and behavior.
//!
//! Attribution values are `Copy` and built from static strings and integer
//! ids only: constructing one on the hot path never allocates. A span records
//! under exactly one bucket — its dotted key plus its attribution — so an
//! attributed sample is never double counted in the unattributed bucket with
//! the same dotted key.

/// Half-open byte range of authored source attributed to a span.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpanRange {
    /// Inclusive start byte offset in the attributed source file.
    pub start: u32,
    /// Exclusive end byte offset in the attributed source file.
    pub end: u32,
}

/// Optional structured coordinates attached to a profile span key.
///
/// [`SpanAttribution::EMPTY`] (all fields `None`) is the identity of every
/// pre-attribution span. Builders are `const fn` over `&'static str` and
/// integer ids, so a fully attributed key is assembled without allocation:
///
/// ```
/// use vize_carton::profiler::SpanAttribution;
///
/// const ATTRIBUTION: SpanAttribution = SpanAttribution::new()
///     .with_stage("s1")
///     .with_pass("hoist_static")
///     .with_block("template");
/// assert_eq!(ATTRIBUTION.stage, Some("s1"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct SpanAttribution {
    /// Pipeline stage that ran the work (for example `"s1"`).
    pub stage: Option<&'static str>,
    /// Pass name within the stage (for example `"hoist_static"`).
    pub pass: Option<&'static str>,
    /// Producer-scoped source file id; the producer owns the id-to-path map.
    pub file_id: Option<u32>,
    /// SFC block kind the work applies to (for example `"template"`).
    pub block: Option<&'static str>,
    /// Byte range of the attributed source construct.
    pub span: Option<SpanRange>,
}

impl SpanAttribution {
    /// The empty attribution: identical to a plain dotted span key.
    pub const EMPTY: Self = Self {
        stage: None,
        pass: None,
        file_id: None,
        block: None,
        span: None,
    };

    /// Create an empty attribution to extend with `with_*` builders.
    #[must_use]
    pub const fn new() -> Self {
        Self::EMPTY
    }

    /// Whether every field is unset (the plain dotted-key identity).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.stage.is_none()
            && self.pass.is_none()
            && self.file_id.is_none()
            && self.block.is_none()
            && self.span.is_none()
    }

    /// Attach the pipeline stage name.
    #[must_use]
    pub const fn with_stage(mut self, stage: &'static str) -> Self {
        self.stage = Some(stage);
        self
    }

    /// Attach the pass name.
    #[must_use]
    pub const fn with_pass(mut self, pass: &'static str) -> Self {
        self.pass = Some(pass);
        self
    }

    /// Attach the producer-scoped source file id.
    #[must_use]
    pub const fn with_file_id(mut self, file_id: u32) -> Self {
        self.file_id = Some(file_id);
        self
    }

    /// Attach the SFC block kind.
    #[must_use]
    pub const fn with_block(mut self, block: &'static str) -> Self {
        self.block = Some(block);
        self
    }

    /// Attach the attributed source byte range (half-open).
    #[must_use]
    pub const fn with_span(mut self, start: u32, end: u32) -> Self {
        self.span = Some(SpanRange { start, end });
        self
    }
}

/// Full identity of an attributed span sample: dotted key plus attribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) struct AttributedSpanKey {
    pub(super) name: &'static str,
    pub(super) attribution: SpanAttribution,
}
