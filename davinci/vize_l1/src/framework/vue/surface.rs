//! Shared Vue scope recovery; each version supplies its syntax policy.

pub(crate) mod sink;

#[derive(Clone, Copy)]
pub(crate) enum LexicalScope {
    Recovered,
    Authored,
}

use crate::markup::Sink;
use crate::markup::htmlish::event::Recorder;
use crate::markup::htmlish::surface::TagKindFact;
use vize_l0::Vec;

#[derive(Default)]
pub(crate) struct HeaderPolicy {
    pub pre: bool,
    pub structural_template: bool,
}

pub(crate) trait SurfacePolicy {
    type Boundary;
    const RECORD_TAG_FACTS: bool = false;
    const LEXICAL_SCOPE: LexicalScope = LexicalScope::Recovered;

    fn pre(raw: &str, offset: u32, source: &str) -> Result<bool, Self::Boundary>;

    fn header_facts(raw: &str, offset: u32, source: &str) -> Result<HeaderPolicy, Self::Boundary> {
        Self::pre(raw, offset, source).map(|pre| HeaderPolicy {
            pre,
            structural_template: false,
        })
    }

    fn tag_facts(
        _tag: &str,
        _structural_template: bool,
        _verbatim: bool,
        _exact_pre: bool,
        _inherited: bool,
    ) -> Option<TagKindFact> {
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
