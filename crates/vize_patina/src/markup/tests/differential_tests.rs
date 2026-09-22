//! Plain-suite witness for the P4-7a TS-25 markup lane: the committed battery
//! runs in every `cargo test`, and its census is pinned to the same numbers
//! the `davinci-differential` corpus entry pins, so a cfg regression in
//! either lane fails loudly.

#[cfg(test)]
mod differential_battery {
    use crate::markup::differential::{JSX, PINNED_BATTERY_CENSUS, compare_jsx, run_battery};

    #[test]
    fn facade_projections_agree_over_the_committed_battery() {
        assert_eq!(run_battery(), PINNED_BATTERY_CENSUS);
    }

    /// The JSX roots the P2-16 projection refuses are named, so the refused
    /// count in the census cannot hide a newly refused construct. The custom
    /// directive module (`v-custom={c}`) is admitted: main projects it.
    #[test]
    fn refused_jsx_roots_are_the_named_ones() {
        let refused: std::vec::Vec<&str> = JSX
            .iter()
            .filter(|(_, lang, source)| {
                compare_jsx(source, *lang).is_ok_and(|comparison| comparison.refused > 0)
            })
            .map(|(name, ..)| *name)
            .collect();
        assert_eq!(refused, [] as [&str; 0]);
    }
}
