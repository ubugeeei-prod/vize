//! Shared Vue scope recovery; each version supplies its syntax policy.

pub(crate) mod sink;

use crate::event::Recorder;
use crate::markup::Sink;
use vize_l0::Vec;

pub(crate) trait SurfacePolicy {
    type Boundary;

    fn pre(raw: &str, offset: u32, source: &str) -> Result<bool, Self::Boundary>;

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
