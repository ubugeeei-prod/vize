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
    /// count in the census cannot hide a newly refused construct. Today that
    /// is one module: a custom JSX directive (`v-custom={c}`) is the
    /// projection's explicit `S2Refusal::Directive`, which P4-7b must close
    /// before the JSX lane can run on S2 alone.
    #[test]
    fn refused_jsx_roots_are_the_named_ones() {
        let refused: std::vec::Vec<&str> = JSX
            .iter()
            .filter(|(_, lang, source)| {
                compare_jsx(source, *lang).is_ok_and(|comparison| comparison.refused > 0)
            })
            .map(|(name, ..)| *name)
            .collect();
        assert_eq!(refused, ["directives"]);
    }

    /// The P4-7b switch oracle over the committed planes and every rule's
    /// fixture SFC: each markup body reproduces its legacy hooks exactly on
    /// the Relief facade (the production lane) and on the S2 facade (outside
    /// the restructured bucket). The census pins that the oracle compared
    /// real diagnostics, not an empty set.
    #[test]
    fn markup_bodies_reproduce_their_legacy_hooks() {
        use crate::markup::differential::{
            SwitchReport, markup_registry, rule_fixture_sfcs, rule_lanes, sfc_rule_lanes,
            template_planes,
        };
        let registry = markup_registry();
        let mut census = [0usize; 4];
        let mut tally = |name: &str, report: SwitchReport| {
            if let Some(divergence) = report.divergences.first() {
                panic!("{name}: {divergence:#?}");
            }
            census[0] += 1;
            census[1] += usize::from(report.restructured);
            census[2] += report.agreed;
            census[3] += report.s2_agreed;
        };
        for plane in template_planes() {
            for (name, source) in plane {
                tally(&name, rule_lanes(&registry, &source));
            }
        }
        for (name, sfc) in rule_fixture_sfcs() {
            tally(&name, sfc_rule_lanes(&registry, &sfc));
        }
        // The 40 markup rules less `vue/permitted-contents`, which keeps its
        // template hooks (`Rule::markup_on_templates`).
        assert_eq!(registry.rules().len(), 39);
        // [templates, restructured, relief-agreed, s2-agreed diagnostics]
        assert_eq!(census, [1068, 0, 825, 825]);
    }
}
