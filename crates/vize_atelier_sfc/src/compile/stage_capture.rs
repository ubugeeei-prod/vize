//! Host-only metadata for the stages that produced an SFC module.

use vize_l0::{
    String,
    dump::capture::{CaptureOutcome, CaptureSink, StageCapture},
};

pub(super) fn configure(
    capture: Option<&mut StageCapture>,
    is_vapor: bool,
    is_ssr: bool,
    source_map: bool,
) {
    let Some(capture) = capture else { return };
    capture.target = String::from(if is_vapor {
        "vapor"
    } else if is_ssr {
        "ssr"
    } else {
        "dom"
    });
    capture.option("ssr", if is_ssr { "true" } else { "false" });
    capture.option("vapor", if is_vapor { "true" } else { "false" });
    capture.option("sourceMap", if source_map { "true" } else { "false" });
}

pub(super) fn unavailable(capture: Option<&mut StageCapture>, reason: &'static str) {
    if let Some(capture) = capture {
        capture.finish(|| CaptureOutcome::Unavailable(String::from(reason)));
    }
}
