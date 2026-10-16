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

    #[test]
    fn v_pre_directive_names_freeze_only_their_own_subtree() {
        use crate::markup::differential::{TemplateComparison, compare_template};
        let mut failures = Vec::new();
        for source in [
            r#"<div class="a" v-pre:arg.mod="foo" id="b"><span v-if="x" /></div>"#,
            r#"<div v-pre:arg :title="x">{{ raw }}<span v-for="x in xs" /></div>"#,
            r#"<div :title="x" v-pre.mod>{{ raw }}<template v-if="x">raw</template></div>"#,
            r#"<div v-pre:[key].mod><span v-pre:inner.mod="x" v-if="y">{{ raw }}</span></div>"#,
            r#"<div v-pre><span v-pre:arg.mod="x" v-if="y">{{ raw }}</span></div>"#,
            r#"<div v-pre.mod><span v-pre>{{ raw }}</span></div><p v-if="ok">{{ live }}</p>"#,
            r#"<div v-pre:arg.mod="foo" v-pre :title="x">{{ raw }}</div>"#,
            r#"<div v-pre v-pre:arg.mod="foo" :title="x">{{ raw }}</div>"#,
            r#"<div v-prex v-preview.mod v-prefix:arg><span v-if="x">{{ live }}</span></div>"#,
            r#"<div V-PRE v-Pre.mod :v-pre="x" @pre="x"><span v-if="x">{{ live }}</span></div>"#,
            r#"<!-- <div v-pre.mod> --><div title="v-pre:arg.mod"><span v-if="x">{{ live }}</span></div>"#,
            "<section>🦀<div\n v-pre:arg.mod='foo'\n :title='x'>{{ raw }}</div><p v-if='ok'/></section>",
        ] {
            let compared = compare_template(source);
            if !matches!(compared, Ok(TemplateComparison::Compared(lines)) if lines > 0) {
                failures.push((source, compared));
            }
        }
        assert!(failures.is_empty(), "{failures:#?}");
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
