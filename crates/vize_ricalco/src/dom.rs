//! The in-phase DOM strangler flag (P2-11, charter #26).
//!
//! Named here so the phase-2 exit gate's deletion grep has one home,
//! matching [`super::pass::TRANSFORM_LANE_FLAG`]. This crate is `no_std`
//! and reads no environment; the dual-run comparator (atelier_dom test
//! space, later in the series) is what *reads* the flag. Value `legacy`
//! leaves the shipped DOM lane alone.

/// The env flag P2-11 runs behind: value `legacy` disarms the S2 DOM
/// dual-run. Deleted, with the lane it guards, at the phase-2 exit gate.
pub const DOM_LANE_FLAG: &str = "VIZE_DAVINCI_DOM";

#[cfg(test)]
mod tests {
    use super::DOM_LANE_FLAG;

    #[test]
    fn the_dom_lane_flag_has_its_recorded_name() {
        assert_eq!(DOM_LANE_FLAG, "VIZE_DAVINCI_DOM");
    }
}
