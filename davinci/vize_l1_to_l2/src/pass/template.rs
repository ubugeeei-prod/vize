//! Co-request the compiler's static analysis and the linter's template
//! complexity analysis over a single lowered owner and its original spans.
use super::{L2_STAGE, cfg, hoist};
use crate::Lowered;
use vize_l0::pass::{PassFailure, PassObserver, Pipeline, run_grouped_pipeline};
use vize_l2::{analysis::template, provenance::ProvenanceSink};

/// Both L2 query results from the shared traversal.
pub use vize_l2::analysis::template::TemplateFacts as TemplateAnalysis;

/// The two registered analyses use the original stage, pass names and masks.
pub const PLAN: Pipeline = Pipeline::new(L2_STAGE, &[hoist::DESC, cfg::DESC]);
const _: () = {
    assert!(PLAN.group_count() == 1);
    assert!(hoist::DESC.fusability.is_fusable() && cfg::DESC.fusability.is_fusable());
};

/// Execute one physical L2 traversal when both products have been requested.
/// Per-pass instrumentation retains the catalogue order. Static remarks and
/// provenance come from their originating pass. The complete walk is timed
/// from group entry to group exit. Lowering diagnostics stay with the owner.
pub fn analyze<P: ProvenanceSink, O: PassObserver>(
    lowered: &mut Lowered<'_, P>,
    observer: &mut O,
) -> Result<TemplateAnalysis, PassFailure> {
    let mut output = None;
    run_grouped_pipeline(&PLAN, observer, |event, remarks| {
        if event.is_group_entry() {
            output = Some(template::run(
                &lowered.artifact.root,
                lowered.op_count,
                &mut lowered.artifact.provenance,
                remarks,
            ));
        }
        Ok(())
    })?;
    output.ok_or(PassFailure::new("template analysis group did not run"))
}
