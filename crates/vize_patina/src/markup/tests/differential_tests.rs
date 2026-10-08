//! Plain-suite witness for the P4-7a TS-25 markup lane: the committed battery
//! runs in every `cargo test`, and its census is pinned to the same numbers
//! the `legacy-differential` corpus entry pins, so a cfg regression in
//! either lane fails loudly.

#[cfg(test)]
mod differential_battery {
    use crate::markup::differential::{JSX, PINNED_BATTERY_CENSUS, compare_jsx, run_battery};

    #[test]
    fn facade_projections_agree_over_the_committed_battery() {
        let census = run_battery().unwrap_or_else(|divergence| panic!("{divergence}"));
        assert_eq!(census, PINNED_BATTERY_CENSUS);
    }

    #[test]
    fn compiler_branch_gap_drop_preserves_the_authored_lint_view() {
        use crate::markup::differential::{TemplateComparison, compare_template};
        for source in [
            include_str!(
                "../../../../../tests/_fixtures/differential/compiler/conditional-branch-whitespace/inline.template.txt"
            ),
            "<p><b v-if=\"a\">A</b>  <!-- gap --> \t<i v-else>B</i> tail</p>",
            "<pre><p><b v-if=\"a\">A</b>\n\t<i v-else>B</i></p></pre>",
            "<p><b v-if=\"a\">A</b> \u{a0}\t<i v-else>B</i></p>",
        ] {
            assert!(
                matches!(
                    compare_template(source),
                    Ok(TemplateComparison::Compared(lines)) if lines > 0
                ),
                "{source}"
            );
        }
    }

    /// The JSX roots the P2-16 projection refuses are named, so the refused
    /// count in the census cannot hide a newly refused construct. The custom
    /// directive module (`v-custom={c}`) is admitted: main projects it. The
    /// falsy-value `&&` scope (#6887) is not modelled by L2 yet.
    #[test]
    fn refused_jsx_roots_are_the_named_ones() {
        let refused: std::vec::Vec<&str> = JSX
            .iter()
            .filter(|(_, lang, source)| {
                compare_jsx(source, *lang).is_ok_and(|comparison| comparison.refused > 0)
            })
            .map(|(name, ..)| *name)
            .collect();
        assert_eq!(refused, ["conditional-value"]);
    }
}
