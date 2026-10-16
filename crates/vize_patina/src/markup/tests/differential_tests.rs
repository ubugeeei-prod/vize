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

    /// Exercise the separate authored-tree view with the same full element,
    /// attribute, opening-range, binding and directive recorder as the battery.
    fn authored_trace(
        document: &crate::markup::MarkupDocument<'_>,
        source: &str,
    ) -> Vec<vize_l0::String> {
        use crate::markup::{MarkupContext, MarkupRule, differential::TraceRecorder};
        let allocator = vize_l0::Allocator::new();
        let mut lint = crate::context::LintContext::new(&allocator, source, "raw.vue");
        let mut context = MarkupContext::new(&mut lint, document);
        let recorder = TraceRecorder::default();
        document.walk_authored_tree(
            &mut |element| {
                recorder.enter_element(&mut context, &element);
                element.walk_bindings(&mut |binding| {
                    recorder.enter_binding(&mut context, &element, &binding);
                });
                element.walk_directives(&mut |directive| {
                    recorder.enter_directive(&mut context, &element, &directive);
                });
            },
            &mut |_| {},
        );
        recorder.into_lines()
    }

    #[test]
    fn v_pre_directive_names_freeze_only_their_own_subtree() {
        use crate::markup::differential::{TemplateComparison, compare_template};
        let mut failures = Vec::new();
        for source in [
            r#"<div class="a" v-pre:arg.mod="foo" id="b"><span v-if="x" /></div>"#,
            r#"<div v-pre:arg :title="x">{{ raw }}<span v-for="x in xs" /></div>"#,
            r#"<div :title="x" v-pre.mod>{{ raw }}<template v-if="x">raw</template></div>"#,
            r#"<div v-pre><template v-if="x">{{ raw }}</template></div>"#,
            r#"<div v-pre.mod><template v-slot:foo v-for="x in xs">raw</template></div>"#,
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
                failures.push(format!("semantic {source}: {compared:?}"));
            }
            let allocator = vize_l0::Allocator::new();
            let (root, _) = vize_armature::Parser::new(&allocator, source).parse();
            let lowered = crate::markup::L2Template::lower(&allocator, source);
            let markup = lowered.markup();
            let relief = crate::markup::MarkupDocument::new(&root, crate::ir::TemplateSyntax::Vue);
            let l2 =
                crate::markup::MarkupDocument::from_l2(&markup, crate::ir::TemplateSyntax::Vue);
            let actual = authored_trace(&l2, source);
            let expected = authored_trace(&relief, source);
            if actual != expected {
                failures.push(format!(
                    "authored {source}: expected={expected:#?} actual={actual:#?}"
                ));
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
