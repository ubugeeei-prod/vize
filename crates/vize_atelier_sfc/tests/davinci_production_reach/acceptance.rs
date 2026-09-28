//! Whole-product accounting for the current compiler fixture sweep.

use super::shapes::Shape;
use crate::Sweep;

/// Every template compile in this sweep consumes the descriptor produced by
/// `vize_croquis::sfc::parse_sfc` in `sweep`. Backend selection and parity do
/// not turn those legacy facts into native-only compiler execution.
fn report_native_only(scope: &str, shape: Shape, sweep: &Sweep) -> String {
    let unverified = sweep.unreadable + sweep.parse_errors + sweep.without_template;
    assert!(unverified <= sweep.files, "corpus accounting exceeds planned inputs");
    let attempted = sweep.files - unverified;
    format!(
        "davinci native-only acceptance: scope={scope} product=compiler target={} planned={} native_only=0 legacy_backed={} unverified={} permille=0 contribution=sfc_parse:legacy:legacy",
        shape.id(),
        sweep.files,
        attempted,
        unverified,
    )
}

/// Keep emitter reach and whole-product acceptance visibly separate for each
/// shipping target over exactly the same planned corpus.
pub fn report(scope: &str, sweep: &Sweep) {
    eprintln!(
        "davinci production reach: scope={scope} files={} unreadable={} parse_errors={} without_template={}",
        sweep.files, sweep.unreadable, sweep.parse_errors, sweep.without_template
    );
    for (shape, tally) in sweep.tallies.values() {
        eprintln!("{}", tally.line(*shape));
        eprintln!("{}", report_native_only(scope, *shape, sweep));
    }
}
