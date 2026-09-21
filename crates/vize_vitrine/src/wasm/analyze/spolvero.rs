//! The `analyzeSfc` Spolvero members: the stage-ladder feed (P2-18, C-2/C-5,
//! with P3-13's remarks)
//! and the ladder's step timings as a P0-11 profile document (C-3), both
//! from one ladder run in `vize_curator`.

/// A monotonic host clock in nanoseconds. The browser entry passes
/// `performance.now()`; native tests pass a deterministic counter.
pub(crate) type HostClock<'c> = &'c dyn Fn() -> u64;

/// `(spolvero, spolveroProfile)` for the SFC's template (no template: an
/// empty feed and an empty profile).
pub(super) fn spolvero_members(
    filename: &str,
    template: Option<&str>,
    remarks: Vec<vize_curator::inspector::SpolveroRemark>,
    clock: HostClock<'_>,
) -> (serde_json::Value, serde_json::Value) {
    let run =
        template.map(|template| vize_curator::inspector::ladder_run(filename, template, clock));
    let (pages, steps) = run.map(|run| (run.pages, run.steps)).unwrap_or_default();
    (
        vize_curator::inspector::spolvero_value_with_remarks("analyze-sfc", pages, remarks),
        vize_curator::inspector::ladder_profile("analyze-sfc", &steps),
    )
}
