//! Shared Vue scope recovery; each version supplies its syntax policy.

pub(crate) mod sink;

use crate::event::Recorder;
use crate::markup::Sink;
use crate::surface::LintTagFact;
use vize_l0::Vec;

#[derive(Default)]
pub(crate) struct HeaderPolicy {
    pub pre: bool,
    pub structural_template: bool,
}

pub(crate) trait SurfacePolicy {
    type Boundary;
    const LINT_TAGS: bool = false;

    fn pre(raw: &str, offset: u32, source: &str) -> Result<bool, Self::Boundary>;

    fn lint_header(raw: &str, offset: u32, source: &str) -> Result<HeaderPolicy, Self::Boundary> {
        Self::pre(raw, offset, source).map(|pre| HeaderPolicy {
            pre,
            structural_template: false,
        })
    }

    fn lint_tag(
        _tag: &str,
        _structural_template: bool,
        _verbatim: bool,
        _exact_pre: bool,
        _inherited: bool,
    ) -> Option<LintTagFact> {
        None
    }

    fn interpolation<'a>(
        _source: &'a str,
        recorder: &mut Recorder<'a, '_>,
        _unsupported: &mut Vec<'a, Self::Boundary>,
        start: usize,
        end: usize,
        _raw: bool,
    ) {
        recorder.on_interpolation(start, end);
    }
}
