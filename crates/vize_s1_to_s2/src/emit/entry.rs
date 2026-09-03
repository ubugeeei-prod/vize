//! One-shot entry points: parse → lower → S2 transform → emit, and the
//! two-half render module the dual-run compares.

use vize_s0::{Allocator, String};

use crate::lower::LegacyCaps;

use super::budget::emit_dom_source_observed_with_options;
use super::options::DomEmitOptions;
use super::{EmitError, emit_dom_source_with_caps_observed};

/// One DOM render module, split the way the shipped codegen splits it
/// (`CodegenResult::{preamble, code}`) so a dual-run can compare each
/// half and the concatenated form the DOM snapshots use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomEmit {
    /// Helper destructure (`const { … } = Vue\n`).
    pub preamble: String,
    /// The `function render(…)` body, no trailing newline after `}`.
    pub code: String,
}

impl DomEmit {
    /// `preamble + "\\n" + code` — the same concatenation
    /// `vize_atelier_dom` snapshots pin.
    #[must_use]
    pub fn assembled(&self) -> String {
        let mut out = self.preamble.clone();
        out.push('\n');
        out.push_str(self.code.as_str());
        out
    }
}

/// Parse → lower → S2 transform → emit. The comparator's one-shot entry
/// so atelier_dom tests do not re-derive the pipeline.
pub fn emit_dom_source<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> Result<DomEmit, EmitError> {
    emit_dom_source_with_caps(allocator, source, LegacyCaps::VUE3)
}

/// [`emit_dom_source`] under an explicit Vue dialect capability set.
pub fn emit_dom_source_with_caps<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    caps: LegacyCaps,
) -> Result<DomEmit, EmitError> {
    emit_dom_source_with_caps_observed(allocator, source, caps).map(|observed| observed.emit)
}

/// [`emit_dom_source_with_caps`] under explicit [`DomEmitOptions`].
pub fn emit_dom_source_with_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    caps: LegacyCaps,
    options: &DomEmitOptions<'_>,
) -> Result<DomEmit, EmitError> {
    emit_dom_source_observed_with_options(allocator, source, caps, options)
        .map(|observed| observed.emit)
}
