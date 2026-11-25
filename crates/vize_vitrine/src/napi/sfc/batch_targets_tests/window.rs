//! A multi-target call inside a host's type-resolution window joins it, as the
//! single-target batch calls do: a co-compiled build reads its imported type
//! modules once for the window, not once per call.

use super::super::{compile_sfc_batch_for_targets, compile_sfc_batch_with_results};
use super::{BUILDS, inputs, options, targets, typed_project};
use vize_atelier_sfc::{begin_type_resolution_batch, join_type_resolution_batch};

/// A call that begins a window of its own ends it while the host's is still
/// open, which moves the process to a new generation. A call that joined
/// leaves the host's generation current. Other tests of this process may open
/// and close windows of their own at the same moment, which also moves the
/// generation, so a joined call needs only one undisturbed attempt; a call
/// that does not join never has one.
fn leaves_the_host_generation(call: &dyn Fn()) -> bool {
    (0..8).any(|_| {
        let host = begin_type_resolution_batch();
        call();
        let after = join_type_resolution_batch();
        after.epoch() == host.epoch()
    })
}

#[test]
fn every_batch_call_joins_the_window_a_host_holds() {
    let project = tempfile::tempdir().expect("temporary project");
    let files = typed_project(project.path());
    let build = BUILDS[0];
    assert!(leaves_the_host_generation(&|| {
        compile_sfc_batch_for_targets(
            inputs(&files),
            Some(options(build, None)),
            Vec::from(targets(build)),
        )
        .expect("multi-target batch");
    }));
    assert!(leaves_the_host_generation(&|| {
        compile_sfc_batch_with_results(inputs(&files), Some(options(build, None))).expect("batch");
    }));
}
