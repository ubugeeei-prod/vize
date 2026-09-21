//! The S3 optimization pipeline: `annotate-placements`, then
//! `extract-placements`, driven by the pass manager.
//!
//! Running through `run_pipeline_remarked` is what attributes every
//! extraction remark to `s3.extract-placements`: the pass and stage come
//! from the running `PassEvent`, never from the pass body. Both passes are
//! barriers that write only the placement overlay (`Preserved::ALL`).

use vize_davinci::pass::{PassDesc, PassFailure, PassObserver, Pipeline, run_pipeline_remarked};

use crate::extract::{EXTRACT, Extraction, OptTier, extract};
use crate::op::Program;
use crate::placement::{ANNOTATE, annotate};

/// The stage name S3 pipelines print and remark under.
pub const S3_STAGE: &str = "s3";

/// The optimization passes, in run order.
pub const OPTIMIZE_PASSES: &[PassDesc] = &[ANNOTATE, EXTRACT];

/// The planned S3 optimization pipeline.
pub const OPTIMIZE: Pipeline = Pipeline::new(S3_STAGE, OPTIMIZE_PASSES);

/// Run [`OPTIMIZE`] over `program` at `tier`, reporting remarks to
/// `observer`.
///
/// # Errors
///
/// Returns the pass manager's failure. The catalogue below is closed over the
/// const pipeline, so a failure here is a compiler bug, not an input property.
pub fn optimize<O: PassObserver>(
    program: &mut Program<'_>,
    tier: OptTier,
    observer: &mut O,
) -> Result<Extraction, PassFailure> {
    let mut extraction = None;
    run_pipeline_remarked(&OPTIMIZE, observer, |event, remarks| {
        let name = event.desc().name;
        if name == ANNOTATE.name {
            annotate(program);
        } else if name == EXTRACT.name {
            let result = extract(program, tier);
            result.remark(remarks);
            extraction = Some(result);
        } else {
            return Err(PassFailure::new(
                "s3 optimization pass has no registered body",
            ));
        }
        Ok(())
    })?;
    extraction.ok_or(PassFailure::new(
        "s3 optimization pipeline did not run extract-placements",
    ))
}
